use crate::addressing::{RouteTo, RouteV1, StaticConfigV1};
use ipnet::IpNet;
use lazy_static::lazy_static;
use serde::Serialize;
use std::net::IpAddr;

lazy_static! {
    static ref DEFAULT_ROUTE_IPV4: IpNet = "0.0.0.0/0".parse().unwrap();
    static ref DEFAULT_ROUTE_IPV6: IpNet = "::/0".parse().unwrap();
}

#[derive(Default, Debug, Serialize, PartialEq)]
pub(crate) struct WickedStaticAddress {
    #[serde(skip_serializing_if = "Option::is_none")]
    address: Option<Vec<StaticAddress>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "route")]
    routes: Option<Vec<WickedRoute>>,
}

impl WickedStaticAddress {
    /// Given the existence, or lack thereof, of addresses and routes, return a
    /// WickedStaticAddress.  The reason we return an `Option` here is that we don't want to
    /// serialize an empty tag if no addresses or routes exist.
    ///
    /// If routes exist, but no static addresses exist, we drop them on the floor since there is a
    /// guard for this condition when validating the network configuration,
    pub(crate) fn maybe_new(
        addresses: Option<StaticConfigV1>,
        routes: Option<Vec<WickedRoute>>,
    ) -> Option<Self> {
        let static_addresses: Option<Vec<StaticAddress>> = addresses.map(StaticConfigV1::into);
        // Wicked doesn't allow routes with DHCP, and routes are worthless without addresses, so
        // don't bother creating anything without addresses
        static_addresses.as_ref()?;

        Some(WickedStaticAddress {
            address: static_addresses,
            routes,
        })
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub(crate) struct StaticAddress {
    #[serde(rename = "$unflatten=local")]
    local: IpNet,
}

impl From<StaticConfigV1> for Vec<StaticAddress> {
    fn from(s: StaticConfigV1) -> Self {
        s.addresses
            .into_iter()
            .map(|a| StaticAddress { local: a })
            .collect()
    }
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct WickedRoute {
    #[serde(rename = "$unflatten=destination")]
    destination: IpNet,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "$unflatten=pref-source")]
    pref_source: Option<IpAddr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nexthop: Option<WickedNextHop>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "$unflatten=priority")]
    priority: Option<u32>,
}

impl WickedRoute {
    pub(crate) fn is_ipv4(&self) -> bool {
        match self.destination {
            IpNet::V4(_) => true,
            IpNet::V6(_) => false,
        }
    }

    pub(crate) fn is_ipv6(&self) -> bool {
        match self.destination {
            IpNet::V4(_) => false,
            IpNet::V6(_) => true,
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct WickedNextHop {
    #[serde(rename = "$unflatten=gateway")]
    gateway: Option<IpAddr>,
}

impl From<RouteV1> for WickedRoute {
    fn from(route: RouteV1) -> Self {
        let destination = match route.to {
            RouteTo::DefaultRoute => match route.via.or(route.from) {
                Some(IpAddr::V4(_)) => *DEFAULT_ROUTE_IPV4,
                Some(IpAddr::V6(_)) => *DEFAULT_ROUTE_IPV6,
                // If no gateway or from is given, assume the ipv4 default
                None => *DEFAULT_ROUTE_IPV4,
            },
            RouteTo::Ip(ip) => ip,
        };

        let nexthop = WickedNextHop { gateway: route.via };

        WickedRoute {
            destination,
            nexthop: Some(nexthop),
            pref_source: route.from,
            priority: route.route_metric,
        }
    }
}

// This type is not meant to be serialized, it's only purpose is to aggregate and categorize the
// ipv4/6 routes on their way to (maybe) being included in a `WickedRoute` which ends up being
// serialized to file.
#[derive(Clone, Default, Debug, PartialEq)]
pub(crate) struct WickedRoutes {
    pub(crate) ipv4: Option<Vec<WickedRoute>>,
    pub(crate) ipv6: Option<Vec<WickedRoute>>,
}

impl WickedRoutes {
    pub(crate) fn add_route(&mut self, route: WickedRoute) {
        if route.is_ipv4() {
            self.ipv4.get_or_insert_with(Vec::new).push(route)
        } else if route.is_ipv6() {
            self.ipv6.get_or_insert_with(Vec::new).push(route)
        }
    }
}

impl From<Vec<RouteV1>> for WickedRoutes {
    fn from(routes: Vec<RouteV1>) -> Self {
        let mut wicked_routes = Self::default();
        for route in routes {
            let wicked_route = WickedRoute::from(route);
            wicked_routes.add_route(wicked_route);
        }
        wicked_routes
    }
}
