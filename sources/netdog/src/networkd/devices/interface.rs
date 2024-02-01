use crate::addressing::{Dhcp4ConfigV1, Dhcp6ConfigV1, RouteV1, StaticConfigV1};
use crate::interface_id::InterfaceId;
use crate::networkd::config::{NetworkBuilder, NetworkConfig};
use crate::networkd::devices::maybe_add_some;
use crate::networkd::{NetworkFileCreator, Vlans};

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

impl NetworkDInterface {
    fn is_unconfigured(&self) -> bool {
        // Destructure self to ensure all applicable Option fields are checked, especially as new
        // fields are added in the future.  The compiler will keep the code honest if fields are
        // accidentally skipped.
        let Self {
            name: _, // name is not an Option
            dhcp4,
            dhcp6,
            static4,
            static6,
            routes,
        } = self;
        dhcp4.is_none()
            && dhcp6.is_none()
            && static4.is_none()
            && static6.is_none()
            && routes.is_none()
    }
}

impl NetworkFileCreator for NetworkDInterface {
    fn create_networks(&self, vlans: &Vlans) -> Vec<NetworkConfig> {
        // Destructure self to ensure we are intentional about skipping or using fields, especially
        // as new fields are added in the future.  The compiler will keep the code honest if fields
        // are accidentally skipped.
        let Self {
            name,
            dhcp4,
            dhcp6,
            static4,
            static6,
            routes,
        } = self;

        // Attach VLANs to this interface if configured with a name.
        let attached_vlans = if let InterfaceId::Name(n) = name {
            vlans.get(n)
        } else {
            None
        };

        // If this interface has attached VLANs but no config, we treat it solely as the link for
        // the VLANs
        if self.is_unconfigured() && attached_vlans.is_some() {
            let mut network = NetworkBuilder::new_vlan_link(name.clone());
            if let Some(vlans) = attached_vlans {
                network.with_vlans(vlans.to_vec())
            }

            vec![network.build()]
        } else {
            let mut network = NetworkBuilder::new_interface(name.clone());
            network.with_dhcp(dhcp4.clone(), dhcp6.clone());
            maybe_add_some!(network, with_static_config, static4);
            maybe_add_some!(network, with_static_config, static6);
            maybe_add_some!(network, with_routes, routes);
            if let Some(vlans) = attached_vlans {
                network.with_vlans(vlans.to_vec())
            }

            vec![network.build()]
        }
    }
}
