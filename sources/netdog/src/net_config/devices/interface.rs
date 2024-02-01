use super::validate_addressing;
use super::{Dhcp4ConfigV1, Dhcp6ConfigV1, Result, Validate};
use crate::addressing::{RouteV1, StaticConfigV1};
use crate::net_config::devices::generate_addressing_validation;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
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

impl Validate for NetInterfaceV2 {
    fn validate(&self) -> Result<()> {
        validate_addressing(self)
    }
}

// Generate the traits for IP Address validation
generate_addressing_validation!(&NetInterfaceV2);
