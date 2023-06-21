use crate::addressing::{Dhcp4ConfigV1, Dhcp6ConfigV1, RouteV1, StaticConfigV1};
use crate::interface_id::InterfaceId;

#[cfg(test)]
use serde::Deserialize;

// Builder unit tests deserialize config to this struct, but we never expect to do that otherwise so put
// the Deserialize derive behind the test attribute
#[cfg_attr(test, derive(Deserialize))]
#[derive(Debug)]
pub(crate) struct NetworkDInterface {
    pub(crate) name: InterfaceId,
    pub(crate) dhcp4: Option<Dhcp4ConfigV1>,
    pub(crate) dhcp6: Option<Dhcp6ConfigV1>,
    pub(crate) static4: Option<StaticConfigV1>,
    pub(crate) static6: Option<StaticConfigV1>,
    pub(crate) routes: Option<Vec<RouteV1>>,
}
