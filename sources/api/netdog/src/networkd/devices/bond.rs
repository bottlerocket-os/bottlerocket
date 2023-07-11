use crate::addressing::{Dhcp4ConfigV1, Dhcp6ConfigV1, RouteV1, StaticConfigV1};
use crate::bonding::{BondModeV1, BondMonitoringConfigV1};
use crate::interface_id::{InterfaceId, InterfaceName};

#[cfg(test)]
use serde::Deserialize;

// Builder unit tests deserialize config to this struct, but we never expect to do that otherwise so put
// the Deserialize derive behind the test attribute
#[cfg_attr(test, derive(Deserialize))]
#[derive(Debug)]
pub(crate) struct NetworkDBond {
    pub(crate) name: InterfaceName,
    pub(crate) dhcp4: Option<Dhcp4ConfigV1>,
    pub(crate) dhcp6: Option<Dhcp6ConfigV1>,
    pub(crate) static4: Option<StaticConfigV1>,
    pub(crate) static6: Option<StaticConfigV1>,
    pub(crate) routes: Option<Vec<RouteV1>>,
    pub(crate) mode: BondModeV1,
    #[cfg_attr(test, serde(rename = "min-links"))]
    pub(crate) min_links: Option<usize>,
    pub(crate) monitoring_config: BondMonitoringConfigV1,
    pub(crate) interfaces: Vec<InterfaceName>,
}
