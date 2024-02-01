//! The `v3` module contains the third version of the network configuration and implements the
//! appropriate traits.

use super::devices::NetworkDeviceV1;
use super::{error, Interfaces, Result, Validate};
use crate::interface_id::{InterfaceId, InterfaceName};
use indexmap::IndexMap;
use serde::Deserialize;
use snafu::ensure;
use std::collections::HashSet;

#[cfg(net_backend = "wicked")]
use crate::wicked::{WickedInterface, WickedLinkConfig};

#[cfg(net_backend = "systemd-networkd")]
use crate::networkd::NetworkDConfig;
#[cfg(net_backend = "systemd-networkd")]
use snafu::ResultExt;

#[derive(Debug, Deserialize)]
pub(crate) struct NetConfigV3 {
    #[serde(flatten)]
    pub(crate) net_devices: IndexMap<InterfaceId, NetworkDeviceV1>,
}

impl Interfaces for NetConfigV3 {
    fn primary_interface(&self) -> Option<InterfaceId> {
        self.net_devices
            .iter()
            .find(|(_, v)| v.primary() == Some(true))
            .or_else(|| self.net_devices.first())
            .map(|(n, _)| n.clone())
    }

    fn has_interfaces(&self) -> bool {
        !self.net_devices.is_empty()
    }

    fn interfaces(&self) -> Vec<InterfaceId> {
        self.net_devices.keys().cloned().collect()
    }

    #[cfg(net_backend = "wicked")]
    fn as_wicked_interfaces(&self) -> Vec<WickedInterface> {
        let mut wicked_interfaces = Vec::new();
        for (name, config) in &self.net_devices {
            let interface = WickedInterface::from((name, config));

            // If config is a Bond, we will generate the interface configuration for interfaces in
            // that bond since we have all of the data and the bond consumes the device for other uses.
            // For each interface: call WickedInterface::new(name), configure it and add that to
            // wicked_interfaces Vec.
            // At this point we can be sure that bonds are being configured with a name rather than
            // a MAC address since that validation happens during deserialize/validation.
            if let (InterfaceId::Name(name), NetworkDeviceV1::BondDevice(b)) = (name, config) {
                for device in &b.interfaces {
                    let mut wicked_sub_interface = WickedInterface::new(device.clone());
                    wicked_sub_interface.link = Some(WickedLinkConfig {
                        master: name.clone(),
                    });

                    wicked_interfaces.push(wicked_sub_interface)
                }
            }

            wicked_interfaces.push(interface)
        }

        wicked_interfaces
    }

    #[cfg(net_backend = "systemd-networkd")]
    fn as_networkd_config(&self) -> Result<NetworkDConfig> {
        let devices = self.net_devices.clone().into_iter().collect();
        NetworkDConfig::new(devices).context(error::NetworkDConfigCreateSnafu)
    }
}

#[allow(clippy::to_string_in_format_args)]
impl Validate for NetConfigV3 {
    fn validate(&self) -> Result<()> {
        // Create HashSet of known device names for checking duplicates
        let mut interface_names: HashSet<&InterfaceName> = self
            .net_devices
            .keys()
            .filter_map(|i| match i {
                InterfaceId::Name(name) => Some(name),
                _ => None,
            })
            .collect();
        for (_name, device) in &self.net_devices {
            if let NetworkDeviceV1::VlanDevice(vlan) = device {
                // It is valid to stack more than one vlan on a single device, but we need them all
                // for checking bonds which can't share devices.
                interface_names.insert(&vlan.device);
            }
        }

        for (name, device) in &self.net_devices {
            // Bonds / vlans cannot be configured via MAC address as it is unsupported in wicked
            if let NetworkDeviceV1::BondDevice(_) | NetworkDeviceV1::VlanDevice(_) = device {
                ensure!(
                    !matches!(name, InterfaceId::MacAddress(_)),
                    error::InvalidNetConfigSnafu {
                        reason: "bonds and vlans may not be configured using MAC address"
                    }
                )
            };

            // Bonds create the interfaces automatically, specifying those interfaces would cause a
            // collision so this emits an error for any that are found
            if let NetworkDeviceV1::BondDevice(config) = device {
                for interface in &config.interfaces {
                    if !interface_names.insert(interface) {
                        return error::InvalidNetConfigSnafu {
                            reason: format!(
                                "{} in bond {} cannot be manually configured",
                                interface.to_string(),
                                name.to_string()
                            ),
                        }
                        .fail();
                    }
                }
            }
            device.validate()?;
        }

        let primary_count = self
            .net_devices
            .values()
            .filter(|v| v.primary() == Some(true))
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
    use crate::net_config::test_macros::{
        basic_tests, bonding_tests, dhcp_tests, static_address_tests, vlan_tests,
    };

    basic_tests!(3);
    dhcp_tests!(3);
    static_address_tests!(3);
    vlan_tests!(3);
    bonding_tests!(3);
}
