//! The addressing module contains the config structures for DHCP and static network addressing.
mod dhcp;
mod static_address;

pub(crate) use dhcp::{Dhcp4ConfigV1, Dhcp4OptionsV1, Dhcp6ConfigV1, Dhcp6OptionsV1};
pub(crate) use static_address::{RouteTo, RouteV1, StaticConfigV1};
