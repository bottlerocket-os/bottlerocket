//! The devices module contains all the types of network devices that `netdog` supports. These are
//! intended to be the structs used for net.toml deserialization including the validation logic for
//! each device.

pub(crate) mod bond;
pub(crate) mod interface;
pub(crate) mod vlan;

use super::{error, Result, Validate};
use crate::addressing::{Dhcp4ConfigV1, Dhcp6ConfigV1, RouteV1, StaticConfigV1};
use bond::NetBondV1;
use interface::NetInterfaceV2;
use serde::Deserialize;
use vlan::NetVlanV1;

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub(crate) enum NetworkDeviceV1 {
    Interface(NetInterfaceV2),
    BondDevice(NetBondV1),
    VlanDevice(NetVlanV1),
}

impl NetworkDeviceV1 {
    pub(crate) fn primary(&self) -> Option<bool> {
        match self {
            Self::Interface(i) => i.primary,
            Self::BondDevice(i) => i.primary,
            Self::VlanDevice(i) => i.primary,
        }
    }
}

impl Validate for NetworkDeviceV1 {
    fn validate(&self) -> Result<()> {
        match self {
            Self::Interface(config) => config.validate()?,
            Self::BondDevice(config) => config.validate()?,
            Self::VlanDevice(config) => config.validate()?,
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum DeviceType {
    #[serde(rename = "bond")]
    Bond,
    #[serde(rename = "vlan")]
    Vlan,
}

pub(crate) trait HasIpAddressing {
    fn has_static(&self) -> bool;

    fn validate_static4(&self) -> Result<()>;
    fn validate_static6(&self) -> Result<()>;

    fn has_dhcp(&self) -> bool;
    fn has_routes(&self) -> bool;
}

pub(crate) fn validate_addressing<D>(device: D) -> Result<()>
where
    D: HasIpAddressing,
{
    if !device.has_dhcp() && !device.has_static() {
        return error::InvalidNetConfigSnafu {
            reason: "each interface must configure dhcp and/or static addresses",
        }
        .fail();
    }

    // wicked doesn't support static routes with dhcp
    if device.has_dhcp() && device.has_routes() {
        return error::InvalidNetConfigSnafu {
            reason: "static routes are not supported with dhcp",
        }
        .fail();
    }

    if device.has_routes() && !device.has_static() {
        return error::InvalidNetConfigSnafu {
            reason: "interfaces must set static addresses in order to use routes",
        }
        .fail();
    }

    // call into struct for access to fields for validation
    device.validate_static4()?;
    device.validate_static6()?;

    Ok(())
}

// For all devices that have IP Addressing available, generate the trait implementation
macro_rules! generate_addressing_validation {
    ($name:ty) => {
        use crate::net_config::devices::HasIpAddressing;
        impl HasIpAddressing for $name {
            fn has_static(&self) -> bool {
                self.static4.is_some() || self.static6.is_some()
            }
            fn validate_static4(&self) -> Result<()> {
                if let Some(config) = &self.static4 {
                    config.validate()?
                }
                Ok(())
            }

            fn validate_static6(&self) -> Result<()> {
                if let Some(config) = &self.static6 {
                    config.validate()?
                }
                Ok(())
            }

            fn has_dhcp(&self) -> bool {
                self.dhcp4.is_some() || self.dhcp6.is_some()
            }
            fn has_routes(&self) -> bool {
                self.routes.is_some()
            }
        }
    };
}
pub(crate) use generate_addressing_validation;
