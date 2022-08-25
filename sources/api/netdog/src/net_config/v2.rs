//! The `v2` module contains the second version of the network configuration and implements the
//! appropriate traits.

use super::static_address::{RouteV1, StaticConfigV1};
use super::{error, Dhcp4ConfigV1, Dhcp6ConfigV1, Interfaces, Result, Validate};
use crate::interface_name::InterfaceName;
use crate::wicked::{WickedDhcp4, WickedDhcp6, WickedInterface, WickedRoutes, WickedStaticAddress};
use indexmap::IndexMap;
use ipnet::IpNet;
use serde::Deserialize;
use snafu::ensure;

#[derive(Debug, Deserialize)]
pub(crate) struct NetConfigV2 {
    #[serde(flatten)]
    pub(crate) interfaces: IndexMap<InterfaceName, NetInterfaceV2>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NetInterfaceV2 {
    // Use this interface as the primary interface for the system
    pub(crate) primary: Option<bool>,
    pub(crate) dhcp4: Option<Dhcp4ConfigV1>,
    pub(crate) dhcp6: Option<Dhcp6ConfigV1>,
    pub(crate) static4: Option<StaticConfigV1>,
    pub(crate) static6: Option<StaticConfigV1>,
    #[serde(rename = "route")]
    pub(crate) routes: Option<Vec<RouteV1>>,
}

impl Interfaces for NetConfigV2 {
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
            let mut interface = WickedInterface::new(name.clone());
            interface.ipv4_dhcp = config.dhcp4.clone().map(WickedDhcp4::from);
            interface.ipv6_dhcp = config.dhcp6.clone().map(WickedDhcp6::from);

            // Based on the existence of static addresses and routes, create the ipv4/6_static
            // struct members.  They must be `Option`s because we want to avoid serializing empty
            // tags into the config file
            let maybe_routes = config.routes.clone().map(WickedRoutes::from);
            let maybe_ipv4_static = WickedStaticAddress::maybe_new(
                config.static4.clone(),
                maybe_routes.as_ref().and_then(|s| s.ipv4.clone()),
            );
            let maybe_ipv6_static = WickedStaticAddress::maybe_new(
                config.static6.clone(),
                maybe_routes.as_ref().and_then(|s| s.ipv6.clone()),
            );
            interface.ipv4_static = maybe_ipv4_static;
            interface.ipv6_static = maybe_ipv6_static;

            wicked_interfaces.push(interface);
        }

        wicked_interfaces
    }
}

impl Validate for NetConfigV2 {
    fn validate(&self) -> Result<()> {
        for (_name, config) in &self.interfaces {
            let has_static = config.static4.is_some() || config.static6.is_some();
            let has_dhcp = config.dhcp4.is_some() || config.dhcp6.is_some();
            let has_routes = config.routes.is_some();

            if !has_dhcp && !has_static {
                return error::InvalidNetConfigSnafu {
                    reason: "each interface must configure dhcp and/or static addresses",
                }
                .fail();
            }

            // wicked doesn't support static routes with dhcp
            if has_dhcp && has_routes {
                return error::InvalidNetConfigSnafu {
                    reason: "static routes are not supported with dhcp",
                }
                .fail();
            }

            if has_routes && !has_static {
                return error::InvalidNetConfigSnafu {
                    reason: "interfaces must set static addresses in order to use routes",
                }
                .fail();
            }

            if let Some(config) = &config.static4 {
                ensure!(
                    config.addresses.iter().all(|a| matches!(a, IpNet::V4(_))),
                    error::InvalidNetConfigSnafu {
                        reason: "'static4' may only contain IPv4 addresses"
                    }
                )
            }

            if let Some(config) = &config.static6 {
                ensure!(
                    config.addresses.iter().all(|a| matches!(a, IpNet::V6(_))),
                    error::InvalidNetConfigSnafu {
                        reason: "'static6' may only contain IPv6 addresses"
                    }
                )
            }
        }

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

#[cfg(test)]
mod tests {
    use crate::net_config::test_macros::{basic_tests, dhcp_tests, static_address_tests};

    basic_tests!(2);
    dhcp_tests!(2);
    static_address_tests!(2);
}
