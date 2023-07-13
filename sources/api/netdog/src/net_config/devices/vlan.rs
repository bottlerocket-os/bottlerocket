use super::validate_addressing;
use super::{Dhcp4ConfigV1, Dhcp6ConfigV1, Result, Validate};
use crate::addressing::{RouteV1, StaticConfigV1};
use crate::interface_id::InterfaceName;
use crate::net_config::devices::generate_addressing_validation;
use crate::vlan_id::VlanId;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct NetVlanV1 {
    pub(crate) primary: Option<bool>,
    pub(crate) dhcp4: Option<Dhcp4ConfigV1>,
    pub(crate) dhcp6: Option<Dhcp6ConfigV1>,
    pub(crate) static4: Option<StaticConfigV1>,
    pub(crate) static6: Option<StaticConfigV1>,
    #[serde(rename = "route")]
    pub(crate) routes: Option<Vec<RouteV1>>,
    #[serde(rename = "kind")]
    _kind: VlanKind,
    pub(crate) device: InterfaceName,
    pub(crate) id: VlanId,
}

// Single variant enum only used to direct deserialization.  If the kind is not "VLAN", "Vlan", or
// "vlan" deserialization will fail.
#[derive(Debug, Deserialize, Clone)]
enum VlanKind {
    #[serde(alias = "VLAN")]
    #[serde(alias = "vlan")]
    Vlan,
}

impl Validate for NetVlanV1 {
    fn validate(&self) -> Result<()> {
        validate_addressing(self)?;
        Ok(())
    }
}

// Generate the traits for IP Address validation
generate_addressing_validation!(&NetVlanV1);
