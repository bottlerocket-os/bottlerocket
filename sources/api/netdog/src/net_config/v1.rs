//! The `v1` module contains the first version of the network configuration and implements the
//! appropriate traits.

use super::{error, Dhcp4ConfigV1, Dhcp6ConfigV1, Error, Interfaces, Result, Validate};
use crate::{
    interface_name::InterfaceName,
    net_config::{Dhcp4OptionsV1, Dhcp6OptionsV1},
    wicked::{WickedControl, WickedDhcp4, WickedDhcp6, WickedInterface},
};
use indexmap::indexmap;
use indexmap::IndexMap;
use serde::Deserialize;
use snafu::{ensure, OptionExt, ResultExt};
use std::{collections::HashSet, str::FromStr};
use std::{convert::TryInto, ops::Deref};

#[derive(Debug, Deserialize)]
pub(crate) struct NetConfigV1 {
    // Use an IndexMap to preserve the order of the devices defined in the net.toml.  The TOML
    // library supports this through a feature making use of IndexMap.  Order is important because
    // we use the first device in the list as the primary device if the `primary` key isn't set for
    // any of the devices.
    //
    // A custom type is used here that will ensure the validity of the interface name as according
    // to the criteria in the linux kernel.  See the `interface_name` module for additional details
    // on the validation performed.
    #[serde(flatten)]
    pub(crate) interfaces: IndexMap<InterfaceName, NetInterfaceV1>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NetInterfaceV1 {
    // Use this interface as the primary interface for the system
    pub(crate) primary: Option<bool>,
    pub(crate) dhcp4: Option<Dhcp4ConfigV1>,
    pub(crate) dhcp6: Option<Dhcp6ConfigV1>,
}

impl Interfaces for NetConfigV1 {
    fn primary_interface(&self) -> Option<String> {
        self.interfaces
            .iter()
            .find(|(_, v)| v.primary == Some(true))
            .or_else(|| self.interfaces.first())
            .map(|(n, _)| n.to_string())
    }

    fn has_interfaces(&self) -> bool {
        !self.interfaces.is_empty()
    }

    fn as_wicked_interfaces(&self) -> Vec<WickedInterface> {
        let mut wicked_interfaces = Vec::with_capacity(self.interfaces.len());
        for (name, config) in &self.interfaces {
            let wicked_dhcp4 = config.dhcp4.clone().map(WickedDhcp4::from);
            let wicked_dhcp6 = config.dhcp6.clone().map(WickedDhcp6::from);
            let wicked_interface = WickedInterface {
                name: name.clone(),
                control: WickedControl::default(),
                ipv4_dhcp: wicked_dhcp4,
                ipv6_dhcp: wicked_dhcp6,
            };
            wicked_interfaces.push(wicked_interface)
        }

        wicked_interfaces
    }
}

impl Validate for NetConfigV1 {
    fn validate(&self) -> Result<()> {
        let dhcp_misconfigured = self
            .interfaces
            .values()
            .any(|cfg| cfg.dhcp4.is_none() && cfg.dhcp6.is_none());
        ensure!(
            !dhcp_misconfigured,
            error::InvalidNetConfigSnafu {
                reason: "each interface must configure dhcp4 or dhcp6, or both",
            }
        );

        let primary_count = self
            .interfaces
            .values()
            .filter(|v| v.primary == Some(true))
            .count();
        ensure!(
            primary_count <= 1,
            error::InvalidNetConfigSnafu {
                reason: "multiple primary interfaces defined, expected 1"
            }
        );

        Ok(())
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// Allow a simple network configuration definition to be parsed from a string.  The expected input
/// string looks like: `interface-name:option1,option2`.  The colon is required.  Acceptable
/// options are "dhcp4", and "dhcp6".  For both options an additional sigil, "?", may be provided
/// to signify that the protocol is optional.  "Optional" in this context means that we will not
/// wait for a lease in order to consider the interface operational.
///
/// An full and sensible example could look like: `eno1:dhcp4,dhcp6?`.  This would create an
/// interface configuration for the interface named `eno1`, enable both dhcp4 and dhcp6, and
/// consider a dhcp6 lease optional.
impl FromStr for NetConfigV1 {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let (name, options) = s
            .split_once(":")
            .context(error::InvalidInterfaceDefSnafu { definition: s })?;

        if options.is_empty() || name.is_empty() {
            return error::InvalidInterfaceDefSnafu { definition: s }.fail();
        }

        let name = name.try_into().context(error::InvalidInterfaceNameSnafu)?;
        let mut interface_config = NetInterfaceV1 {
            primary: None,
            dhcp4: None,
            dhcp6: None,
        };

        // Keep track of the options we've parsed, and fail if an option is passed more than once,
        // for example "dhcp4,dhcp4?"
        let mut provided_options = HashSet::new();
        for option in options.split(',').collect::<Vec<&str>>() {
            if provided_options.contains(option) {
                return error::InvalidInterfaceDefSnafu { definition: s }.fail();
            }

            if option.starts_with("dhcp4") {
                provided_options.insert("dhcp4");
                interface_config.dhcp4 = Some(Dhcp4ConfigV1::from_str(option)?)
            } else if option.starts_with("dhcp6") {
                provided_options.insert("dhcp6");
                interface_config.dhcp6 = Some(Dhcp6ConfigV1::from_str(option)?)
            } else {
                return error::InvalidInterfaceOptionSnafu { given: option }.fail();
            }
        }

        let interfaces = indexmap! {name => interface_config};
        let net_config = NetConfigV1 { interfaces };
        Ok(net_config)
    }
}

/// Parse Dhcp4 configuration from a string.  See the `FromStr` impl for `NetConfig` for
/// additional details.
///
/// The expected input here is a string beginning with `dhcp4`.
impl FromStr for Dhcp4ConfigV1 {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        ensure!(
            s.starts_with("dhcp4"),
            error::CreateFromStrSnafu {
                what: "Dhcp4 options",
                given: s
            }
        );

        let mut optional = None;
        let maybe_sigils = s.trim_start_matches("dhcp4");
        if !maybe_sigils.is_empty() {
            let sigils = Sigils::from_str(maybe_sigils)?;
            for sigil in &*sigils {
                match sigil {
                    Sigil::Optional => {
                        optional = Some(true);
                    }
                }
            }
        }

        let dhcp4_options = Dhcp4OptionsV1 {
            enabled: true,
            optional,
            route_metric: None,
        };
        Ok(Dhcp4ConfigV1::WithOptions(dhcp4_options))
    }
}

/// Parse Dhcp6 configuration from a string.  See the `FromStr` impl for `NetConfig` for
/// additional details.
///
/// The expected input here is a string beginning with `dhcp6`.
impl FromStr for Dhcp6ConfigV1 {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        ensure!(
            s.starts_with("dhcp6"),
            error::CreateFromStrSnafu {
                what: "Dhcp6 options",
                given: s
            }
        );

        let mut optional = None;
        let maybe_sigils = s.trim_start_matches("dhcp6");
        if !maybe_sigils.is_empty() {
            let sigils = Sigils::from_str(maybe_sigils)?;
            for sigil in &*sigils {
                match sigil {
                    Sigil::Optional => {
                        optional = Some(true);
                    }
                }
            }
        }

        let dhcp6_options = Dhcp6OptionsV1 {
            enabled: true,
            optional,
        };
        Ok(Dhcp6ConfigV1::WithOptions(dhcp6_options))
    }
}

/// A wrapper around the possible sigils meant to configure dhcp4 and dhcp6 for an interface. These
/// sigils will be parsed as part of an interface directive string, e.g. "dhcp4?". Currently only
/// "Optional" is supported ("?").
#[derive(Debug)]
enum Sigil {
    Optional,
}

#[derive(Debug)]
struct Sigils(Vec<Sigil>);

// This is mostly for convenience to allow iterating over the contained Vec
impl Deref for Sigils {
    type Target = Vec<Sigil>;

    fn deref(&self) -> &Vec<Sigil> {
        &self.0
    }
}

impl FromStr for Sigils {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut sigils = Sigils(Vec::new());

        // `chars()` won't give us grapheme clusters, but we don't support any exotic sigils so
        // chars should be fine here
        let sigil_chars = s.chars();
        for sigil in sigil_chars {
            match sigil {
                '?' => sigils.0.push(Sigil::Optional),
                _ => {
                    return error::CreateFromStrSnafu {
                        what: "sigils",
                        given: sigil,
                    }
                    .fail()
                }
            }
        }

        Ok(sigils)
    }
}
