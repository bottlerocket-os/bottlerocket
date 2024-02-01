use crate::addressing::{Dhcp4ConfigV1, Dhcp6ConfigV1, RouteV1, StaticConfigV1};
use crate::bonding::{BondModeV1, BondMonitoringConfigV1};
use crate::interface_id::InterfaceName;
use crate::networkd::config::{NetDevBuilder, NetDevConfig, NetworkBuilder, NetworkConfig};
use crate::networkd::devices::maybe_add_some;
use crate::networkd::{NetDevFileCreator, NetworkFileCreator, Vlans};

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

impl NetDevFileCreator for NetworkDBond {
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
            mode,
            min_links,
            monitoring_config,
            interfaces: _, // Used in .network files, not here
        } = self;

        let mut netdev = NetDevBuilder::new_bond(name.clone());
        netdev.with_mode(mode.clone());
        maybe_add_some!(netdev, with_min_links, min_links);

        match monitoring_config.clone() {
            BondMonitoringConfigV1::MiiMon(miimon) => netdev.with_miimon_config(miimon),
            BondMonitoringConfigV1::ArpMon(arpmon) => netdev.with_arpmon_config(arpmon),
        }

        netdev.build()
    }
}

impl NetworkFileCreator for NetworkDBond {
    fn create_networks(&self, vlans: &Vlans) -> Vec<NetworkConfig> {
        let mut configs = Vec::new();

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
            mode: _, // mode / min_links / monitoring are used in .netdev files
            min_links: _,
            monitoring_config: _,
            interfaces,
        } = self;

        let mut network = NetworkBuilder::new_bond(name.clone());
        network.with_dhcp(dhcp4.clone(), dhcp6.clone());
        maybe_add_some!(network, with_static_config, static4);
        maybe_add_some!(network, with_static_config, static6);
        maybe_add_some!(network, with_routes, routes);

        network.with_bind_carrier(interfaces.clone());

        // Attach VLANs to this interface, if any
        if let Some(vlans) = vlans.get(name) {
            network.with_vlans(vlans.to_vec())
        }

        configs.push(network.build());

        // Create the .network files for the worker interfaces
        for (index, worker_name) in interfaces.iter().enumerate() {
            let mut worker = NetworkBuilder::new_bond_worker(worker_name.clone());
            worker.bound_to_bond(name.clone());

            // The first worker in the list is the primary
            if index == 0 {
                worker.primary_bond_worker();
            }
            configs.push(worker.build());
        }

        configs
    }
}
