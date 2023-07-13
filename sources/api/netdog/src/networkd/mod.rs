pub(crate) mod config;
mod devices;

use self::config::{NetDevConfig, NetworkConfig, NetworkDConfigFile};
use self::devices::{NetworkDDevice, NetworkDInterface};
use crate::interface_id::{InterfaceId, InterfaceName};
use std::collections::HashMap;

// A map of network device -> associated VLANs.  This type exists to assist in generating a
// device's network configuration, which must contain it's associated VLANs.
pub(self) type Vlans = HashMap<InterfaceName, Vec<InterfaceName>>;

pub(crate) struct NetworkDConfig {
    devices: Vec<NetworkDDevice>,
    vlans: Vlans,
}

impl NetworkDConfig {
    pub(crate) fn new<T>(network_devices: Vec<T>) -> Result<Self>
    where
        T: TryInto<NetworkDDevice, Error = self::error::Error>,
    {
        let mut devices = network_devices
            .into_iter()
            .map(|d| d.try_into())
            .collect::<Result<Vec<NetworkDDevice>>>()?;

        let mut device_names = Vec::with_capacity(devices.len());
        let mut vlans = HashMap::new();
        for device in &devices {
            device_names.push(device.name());
            // VLANs are typically attached to a network device (bond/interface). Net config
            // specifies the bond or interface in the VLAN config. In systemd-networkd config, the
            // VLAN is specified in the device config. Create a map of interface/bond name to list
            // of attached VLANs to assist with config generation later
            if let NetworkDDevice::Vlan(vlan) = device {
                vlans
                    .entry(vlan.device.clone())
                    .or_insert_with(Vec::new)
                    .push(vlan.name.clone())
            }
        }

        // If the VLANs map contains a device we don't otherwise have config for, it means the
        // device is used only as a link for the VLAN.  This is used in VLAN "tagged only" type
        // setups.  Add an empty NetworkDInterface to the list of config to be generated.  An empty
        // device will get a .network file so it is managed and becomes a member of the VLAN, but
        // will otherwise have all DHCP and addressing config turned off.
        for vlan_device in vlans.keys() {
            if !device_names.contains(&InterfaceId::from(vlan_device.clone())) {
                devices.push(NetworkDDevice::Interface(NetworkDInterface {
                    name: InterfaceId::Name(vlan_device.clone()),
                    dhcp4: None,
                    dhcp6: None,
                    static4: None,
                    static6: None,
                    routes: None,
                }))
            }
        }

        Ok(Self { devices, vlans })
    }

    /// Generate systemd-networkd configuration files for all known devices
    pub(crate) fn create_files(self) -> Vec<NetworkDConfigFile> {
        self.devices
            .iter()
            .flat_map(|d| d.create_files(&self.vlans))
            .collect()
    }
}

/// Devices implement this trait if they require a .netdev file
trait NetDevFileCreator {
    fn create_netdev(&self) -> NetDevConfig;
}

/// Devices implement this trait if they require one or more .network files (bonds, for example,
/// create multiple .network files for the bond and it's workers)
trait NetworkFileCreator {
    fn create_networks(&self, vlans: &Vlans) -> Vec<NetworkConfig>;
}

mod error {
    use snafu::Snafu;
    use std::io;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(crate)))]
    pub(crate) enum Error {
        #[snafu(display("Unable to create '{}', missing name or MAC", what))]
        ConfigMissingName { what: String },

        #[snafu(display("Unable to write {} to {}: {}", what, path.display(), source))]
        NetworkDConfigWrite {
            what: String,
            path: PathBuf,
            source: io::Error,
        },
    }
}
pub(crate) type Result<T> = std::result::Result<T, error::Error>;
