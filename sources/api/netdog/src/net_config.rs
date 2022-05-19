//! The net_config module contains the strucures needed to deserialize a `net.toml` file.  It also
//! includes contains the `FromStr` implementations to create a `NetConfig` from string, like from
//! the kernel command line.
//!
//! These structures are the user-facing options for configuring one or more network interfaces.
use crate::interface_name::InterfaceName;
use indexmap::{indexmap, IndexMap};
use serde::Deserialize;
use snafu::{ensure, OptionExt, ResultExt};
use std::collections::HashSet;
use std::convert::TryInto;
use std::fs;
use std::ops::Deref;
use std::path::Path;
use std::str::FromStr;

static DEFAULT_INTERFACE_PREFIX: &str = "netdog.default-interface=";

// TODO: support deserializing different versions of this configuration.
// Idea: write a deserializer that uses the `version` field and deserializes the rest of the config
// into an enum with variants for each version, i.e.
// enum NetConfig {
//     V1(NetInterfaceV1)
//     V2(NetInterfaceV2)
// }
#[derive(Debug, Deserialize)]
pub(crate) struct NetConfig {
    pub(crate) version: u8,
    // Use an IndexMap to preserve the order of the devices defined in the net.toml.  The TOML
    // library supports this through a feature making use of IndexMap.  Order is important because
    // we use the first device in the list as the primary device if the `primary` key isn't set for
    // any of the devices.
    //
    // A custom type is used here that will ensure the validity of the interface name as according
    // to the criteria in the linux kernel.  See the `interface_name` module for additional details
    // on the validation performed.
    #[serde(flatten)]
    pub(crate) interfaces: IndexMap<InterfaceName, NetInterface>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NetInterface {
    // Use this interface as the primary interface for the system
    pub(crate) primary: Option<bool>,
    pub(crate) dhcp4: Option<Dhcp4Config>,
    pub(crate) dhcp6: Option<Dhcp6Config>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum Dhcp4Config {
    DhcpEnabled(bool),
    WithOptions(Dhcp4Options),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Dhcp4Options {
    pub(crate) enabled: Option<bool>,
    pub(crate) optional: Option<bool>,
    #[serde(rename = "route-metric")]
    pub(crate) route_metric: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum Dhcp6Config {
    DhcpEnabled(bool),
    WithOptions(Dhcp6Options),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Dhcp6Options {
    pub(crate) enabled: Option<bool>,
    pub(crate) optional: Option<bool>,
}

impl NetConfig {
    /// Create a `NetConfig` from file
    pub(crate) fn from_path<P>(path: P) -> Result<Option<Self>>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let net_config_str =
            fs::read_to_string(path).context(error::NetConfigReadFailedSnafu { path })?;
        let net_config: NetConfig =
            toml::from_str(&net_config_str).context(error::NetConfigParseSnafu { path })?;

        ensure!(
            net_config.version == 1,
            error::InvalidNetConfigSnafu {
                reason: "invalid version"
            }
        );

        let dhcp_misconfigured = net_config
            .interfaces
            .values()
            .any(|cfg| cfg.dhcp4.is_none() && cfg.dhcp6.is_none());
        ensure!(
            !dhcp_misconfigured,
            error::InvalidNetConfigSnafu {
                reason: "each interface must configure dhcp4 or dhcp6, or both",
            }
        );

        let primary_count = net_config
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

        if net_config.interfaces.is_empty() {
            return Ok(None);
        }

        Ok(Some(net_config))
    }

    /// Create a `NetConfig` from string from the kernel command line
    pub(crate) fn from_command_line<P>(path: P) -> Result<Option<Self>>
    where
        P: AsRef<Path>,
    {
        let p = path.as_ref();
        let kernel_cmdline =
            fs::read_to_string(p).context(error::KernelCmdlineReadFailedSnafu { path: p })?;

        let mut maybe_interfaces = kernel_cmdline
            .split_whitespace()
            .filter(|s| s.starts_with(DEFAULT_INTERFACE_PREFIX));

        let default_interface = match maybe_interfaces.next() {
            Some(interface_str) => interface_str
                .trim_start_matches(DEFAULT_INTERFACE_PREFIX)
                .to_string(),
            None => return Ok(None),
        };

        ensure!(
            maybe_interfaces.next().is_none(),
            error::MultipleDefaultInterfacesSnafu
        );

        let net_config = NetConfig::from_str(&default_interface)?;
        Ok(Some(net_config))
    }

    /// Return the primary interface for the system.  If none of the interfaces are defined as
    /// `primary = true`, we use the first interface in the configuration file.  Returns `None` in
    /// the case no interfaces are defined.
    pub(crate) fn primary_interface(&self) -> Option<String> {
        self.interfaces
            .iter()
            .find(|(_, v)| v.primary == Some(true))
            .or_else(|| self.interfaces.first())
            .map(|(n, _)| n.to_string())
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
impl FromStr for NetConfig {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let (name, options) = s
            .split_once(":")
            .context(error::InvalidInterfaceDefSnafu { definition: s })?;

        if options.is_empty() || name.is_empty() {
            return error::InvalidInterfaceDefSnafu { definition: s }.fail();
        }

        let name = name.try_into().context(error::InvalidInterfaceNameSnafu)?;
        let mut interface_config = NetInterface {
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
                interface_config.dhcp4 = Some(Dhcp4Config::from_str(option)?)
            } else if option.starts_with("dhcp6") {
                provided_options.insert("dhcp6");
                interface_config.dhcp6 = Some(Dhcp6Config::from_str(option)?)
            } else {
                return error::InvalidInterfaceOptionSnafu { given: option }.fail();
            }
        }

        let interfaces = indexmap! {name => interface_config};
        let net_config = NetConfig {
            version: 1,
            interfaces,
        };
        Ok(net_config)
    }
}

/// Parse Dhcp4 configuration from a string.  See the `FromStr` impl for `NetConfig` for
/// additional details.
///
/// The expected input here is a string beginning with `dhcp4`.
impl FromStr for Dhcp4Config {
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

        let dhcp4_options = Dhcp4Options {
            enabled: Some(true),
            optional,
            route_metric: None,
        };
        Ok(Dhcp4Config::WithOptions(dhcp4_options))
    }
}

/// Parse Dhcp6 configuration from a string.  See the `FromStr` impl for `NetConfig` for
/// additional details.
///
/// The expected input here is a string beginning with `dhcp6`.
impl FromStr for Dhcp6Config {
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

        let dhcp6_options = Dhcp6Options {
            enabled: Some(true),
            optional,
        };
        Ok(Dhcp6Config::WithOptions(dhcp6_options))
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

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

mod error {
    use crate::interface_name;
    use snafu::Snafu;
    use std::io;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(crate)))]
    pub(crate) enum Error {
        #[snafu(display("Unable to create '{}', from '{}'", what, given))]
        CreateFromStr { what: String, given: String },

        #[snafu(display(
            "Invalid interface definition, expected 'name:option1,option2', got {}",
            definition
        ))]
        InvalidInterfaceDef { definition: String },

        #[snafu(display("Invalid interface name: {}", source))]
        InvalidInterfaceName { source: interface_name::Error },

        #[snafu(display(
            "Invalid interface option, expected 'dhcp4' or 'dhcp6', got '{}'",
            given
        ))]
        InvalidInterfaceOption { given: String },

        #[snafu(display("Invalid network configuration: {}", reason))]
        InvalidNetConfig { reason: String },

        #[snafu(display("Failed to read kernel command line from '{}': {}", path.display(), source))]
        KernelCmdlineReadFailed { path: PathBuf, source: io::Error },

        #[snafu(display(
            "Multiple default interfaces defined on kernel command line, expected 1",
        ))]
        MultipleDefaultInterfaces,

        #[snafu(display("Failed to read network config from '{}': {}", path.display(), source))]
        NetConfigReadFailed { path: PathBuf, source: io::Error },

        #[snafu(display("Failed to parse network config from '{}': {}", path.display(), source))]
        NetConfigParse {
            path: PathBuf,
            source: toml::de::Error,
        },
    }
}

pub(crate) use error::Error;
type Result<T> = std::result::Result<T, error::Error>;

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn test_data() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data")
    }

    fn cmdline() -> PathBuf {
        test_data().join("cmdline")
    }

    fn net_config() -> PathBuf {
        test_data().join("net_config")
    }

    #[test]
    fn ok_cmdline() {
        let cmdline = cmdline().join("ok");
        assert!(NetConfig::from_command_line(cmdline).unwrap().is_some());
    }

    #[test]
    fn multiple_interface_from_cmdline() {
        let cmdline = cmdline().join("multiple_interface");
        assert!(NetConfig::from_command_line(cmdline).is_err())
    }

    #[test]
    fn no_interfaces_cmdline() {
        let cmdline = cmdline().join("no_interfaces");
        assert!(NetConfig::from_command_line(cmdline).unwrap().is_none())
    }

    #[test]
    fn invalid_version() {
        let bad = net_config().join("bad_version.toml");
        assert!(NetConfig::from_path(bad).is_err())
    }

    #[test]
    fn ok_config() {
        let ok = net_config().join("net_config.toml");
        assert!(NetConfig::from_path(ok).is_ok())
    }

    #[test]
    fn invalid_interface_config() {
        let bad = net_config().join("invalid_interface_config.toml");
        assert!(NetConfig::from_path(bad).is_err())
    }

    #[test]
    fn invalid_dhcp4_config() {
        let bad = net_config().join("invalid_dhcp4_config.toml");
        assert!(NetConfig::from_path(bad).is_err())
    }

    #[test]
    fn invalid_dhcp6_config() {
        let bad = net_config().join("invalid_dhcp6_config.toml");
        assert!(NetConfig::from_path(bad).is_err())
    }

    #[test]
    fn invalid_dhcp_config() {
        let ok = net_config().join("invalid_dhcp_config.toml");
        assert!(NetConfig::from_path(ok).is_err())
    }

    #[test]
    fn no_interfaces() {
        let bad = net_config().join("no_interfaces.toml");
        assert!(NetConfig::from_path(bad).unwrap().is_none())
    }

    #[test]
    fn defined_primary_interface() {
        let ok_path = net_config().join("net_config.toml");
        let cfg = NetConfig::from_path(ok_path).unwrap().unwrap();

        let expected = "eno2";
        let actual = cfg.primary_interface().unwrap();
        assert_eq!(expected, actual)
    }

    #[test]
    fn undefined_primary_interface() {
        let ok_path = net_config().join("no_primary.toml");
        let cfg = NetConfig::from_path(ok_path).unwrap().unwrap();

        let expected = "eno3";
        let actual = cfg.primary_interface().unwrap();
        println!("{}", &actual);
        assert_eq!(expected, actual)
    }

    #[test]
    fn multiple_primary_interfaces() {
        let multiple = net_config().join("multiple_primary.toml");
        assert!(NetConfig::from_path(multiple).is_err())
    }

    #[test]
    fn ok_interface_from_str() {
        let ok = &[
            "eno1:dhcp4,dhcp6",
            "eno1:dhcp4,dhcp6?",
            "eno1:dhcp4?,dhcp6",
            "eno1:dhcp4?,dhcp6?",
            "eno1:dhcp6?,dhcp4?",
            "eno1:dhcp4",
            "eno1:dhcp4?",
            "eno1:dhcp6",
            "eno1:dhcp6?",
        ];
        for ok_str in ok {
            assert!(NetConfig::from_str(ok_str).is_ok())
        }
    }

    #[test]
    fn invalid_interface_from_str() {
        let bad = &[
            "",
            ":",
            "eno1:",
            ":dhcp4,dhcp6",
            "dhcp4",
            "eno1:dhc4",
            "eno1:dhcp",
            "eno1:dhcp4+",
            "eno1:dhcp?",
            "eno1:dhcp4?,dhcp4",
            "ENO1:DHCP4?,DhCp6",
        ];
        for bad_str in bad {
            assert!(NetConfig::from_str(bad_str).is_err())
        }
    }
}
