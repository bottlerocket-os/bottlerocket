use super::validate_addressing;
use super::{Dhcp4ConfigV1, Dhcp6ConfigV1, Result, Validate};
use crate::interface_name::InterfaceName;
use crate::net_config::devices::generate_addressing_validation;
use crate::net_config::{RouteV1, StaticConfigV1};
use serde::de::Error;
use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(remote = "Self")]
pub(crate) struct NetVlanV1 {
    pub(crate) primary: Option<bool>,
    pub(crate) dhcp4: Option<Dhcp4ConfigV1>,
    pub(crate) dhcp6: Option<Dhcp6ConfigV1>,
    pub(crate) static4: Option<StaticConfigV1>,
    pub(crate) static6: Option<StaticConfigV1>,
    #[serde(rename = "route")]
    pub(crate) routes: Option<Vec<RouteV1>>,
    kind: String,
    pub(crate) device: InterfaceName,
    pub(crate) id: u16,
}

impl<'de> Deserialize<'de> for NetVlanV1 {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let this = Self::deserialize(deserializer)?;

        if this.kind.to_lowercase().as_str() != "vlan" {
            return Err(D::Error::custom(format!(
                "kind of '{}' does not match 'vlan'",
                this.kind.as_str()
            )));
        }

        // Validate its a valid vlan id - 0-4095
        if this.id > 4095 {
            return Err(D::Error::custom(
                "invalid vlan ID specified, must be between 0-4095",
            ));
        }

        Ok(this)
    }
}

impl Validate for NetVlanV1 {
    fn validate(&self) -> Result<()> {
        validate_addressing(self)?;
        Ok(())
    }
}

// Generate the traits for IP Address validation
generate_addressing_validation!(&NetVlanV1);
