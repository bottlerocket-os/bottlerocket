use crate::addressing::{Dhcp4ConfigV1, Dhcp6ConfigV1, RouteV1, StaticConfigV1};
use crate::bonding::{BondModeV1, BondMonitoringConfigV1};
use crate::interface_id::{InterfaceId, InterfaceName};

#[derive(Debug)]
pub(crate) struct NetworkDBond {
    pub(crate) name: InterfaceName,
    pub(crate) dhcp4: Option<Dhcp4ConfigV1>,
    pub(crate) dhcp6: Option<Dhcp6ConfigV1>,
    pub(crate) static4: Option<StaticConfigV1>,
    pub(crate) static6: Option<StaticConfigV1>,
    pub(crate) routes: Option<Vec<RouteV1>>,
    pub(crate) mode: BondModeV1,
    pub(crate) min_links: Option<usize>,
    pub(crate) monitoring_config: BondMonitoringConfigV1,
    pub(crate) interfaces: Vec<InterfaceName>,
}
