use crate::addressing::{Dhcp4ConfigV1, Dhcp6ConfigV1, RouteV1, StaticConfigV1};
use crate::interface_id::InterfaceName;
use crate::networkd::config::{NetDevBuilder, NetDevConfig, NetworkBuilder, NetworkConfig};
use crate::networkd::devices::maybe_add_some;
use crate::networkd::{NetDevFileCreator, NetworkFileCreator, Vlans};
use crate::vlan_id::VlanId;

#[cfg(test)]
use serde::Deserialize;

// Builder unit tests deserialize config to this struct, but we never expect to do that otherwise so put
// the Deserialize derive behind the test attribute
#[cfg_attr(test, derive(Deserialize))]
#[derive(Debug)]
pub(crate) struct NetworkDVlan {
    pub(crate) name: InterfaceName,
    pub(crate) dhcp4: Option<Dhcp4ConfigV1>,
    pub(crate) dhcp6: Option<Dhcp6ConfigV1>,
    pub(crate) static4: Option<StaticConfigV1>,
    pub(crate) static6: Option<StaticConfigV1>,
    pub(crate) routes: Option<Vec<RouteV1>>,
    // The device field isn't used in the creation of the .network or .netdev files for this VLAN.
    // It is used to create a map of device -> VLANs to ensure the device's contain the "VLAN"
    // entry for this VLAN
    pub(crate) device: InterfaceName,
    pub(crate) id: VlanId,
}

impl NetDevFileCreator for NetworkDVlan {
    fn create_netdev(&self) -> NetDevConfig {
        // Destructure self to ensure we are intentional about skipping or using fields, especially
        // as new fields are added in the future.  The compiler will keep the code honest if fields
        // are accidentally skipped.
        let Self {
            name,
            dhcp4: _, // DHCP / static addressing isn't used in .netdev files
            dhcp6: _,
            static4: _,
            static6: _,
            routes: _,
            device: _, // Device isn't used in .netdev files
            id,
        } = self;

        let mut netdev = NetDevBuilder::new_vlan(name.clone());
        netdev.with_vlan_id(id.clone());

        netdev.build()
    }
}

impl NetworkFileCreator for NetworkDVlan {
    fn create_networks(&self, _vlans: &Vlans) -> Vec<NetworkConfig> {
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
            device: _, // device and id aren't used in .network files
            id: _,
        } = self;

        let mut network = NetworkBuilder::new_vlan(name.clone());
        network.with_dhcp(dhcp4.clone(), dhcp6.clone());
        maybe_add_some!(network, with_static_config, static4);
        maybe_add_some!(network, with_static_config, static6);
        maybe_add_some!(network, with_routes, routes);

        vec![network.build()]
    }
}
