pub(crate) mod config;
mod conversions;
mod devices;

use self::config::{NetDevConfig, NetworkConfig, NetworkDConfigFile};
use self::devices::{NetworkDDevice, NetworkDInterface};
use crate::interface_id::{InterfaceId, InterfaceName};
use std::collections::HashMap;

// A map of network device -> associated VLANs.  This type exists to assist in generating a
// device's network configuration, which must contain it's associated VLANs.
type Vlans = HashMap<InterfaceName, Vec<InterfaceName>>;

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

        #[snafu(display(
            "Unable to create systemd-networkd '{}' with MAC address, must use a name",
            what
        ))]
        InvalidWithMac { what: String },

        #[snafu(display("Unable to write {} to {}: {}", what, path.display(), source))]
        NetworkDConfigWrite {
            what: String,
            path: PathBuf,
            source: io::Error,
        },
    }
}
pub(crate) use error::Error;
pub(crate) type Result<T> = std::result::Result<T, error::Error>;

#[cfg(test)]
mod tests {
    use super::devices::{NetworkDBond, NetworkDInterface, NetworkDVlan};
    use super::*;
    use crate::net_config::{self, Interfaces, NetConfigV1};
    use handlebars::Handlebars;
    use serde::Serialize;
    use std::fmt::Display;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::str::FromStr;

    static NET_CONFIG_VERSIONS: &[u8] = &[1, 2, 3];
    const NET_CONFIG: &str = include_str!("../../test_data/net_config.toml");

    fn networkd_data() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test_data")
            .join("networkd")
    }

    // Only needed for test purposes to easily create a string from the underlying config
    impl Display for NetworkDConfigFile {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                NetworkDConfigFile::Network(nw) => write!(f, "{}", nw),
                NetworkDConfigFile::NetDev(nd) => write!(f, "{}", nd),
            }
        }
    }

    fn device_name(device: &NetworkDDevice) -> String {
        match device {
            NetworkDDevice::Interface(i) => i.name.to_string(),
            NetworkDDevice::Bond(b) => b.name.to_string(),
            NetworkDDevice::Vlan(v) => v.name.to_string(),
        }
    }

    // Test the end-to-end trip: "net config from cmdline -> networkd config -> serialized config"
    #[test]
    fn interface_config_from_str() {
        // Interface names here coincide with config files, some of which are shared with the
        // `net_config` test below
        let ok = &[
            "eno1:dhcp4",
            "eno2:dhcp6",
            "eno9:dhcp4?",
            "eno10:dhcp6?",
            "eno5:dhcp4,dhcp6",
            "eno5:dhcp6,dhcp4",
            "eno7:dhcp4,dhcp6?",
            "eno7:dhcp6?,dhcp4",
            "eno8:dhcp6?,dhcp4?",
            "eno8:dhcp4?,dhcp6?",
        ];
        for ok_str in ok {
            let net_config = NetConfigV1::from_str(ok_str).unwrap();

            let networkd_config = net_config.as_networkd_config().unwrap();
            for device in networkd_config.devices {
                let name = device_name(&device);
                let configs = device.create_files(&networkd_config.vlans);

                // We know the array of strings only creates interface configs, which are 1:1 with
                // the device.
                assert!(configs.len() == 1);
                for mut config in configs {
                    let mut path = networkd_data().join("network").join(&name);
                    path.set_extension("network");

                    let expected = fs::read_to_string(&path).unwrap();
                    let generated = config.to_string();
                    assert_eq!(
                        expected,
                        generated,
                        "Generated output does not match file: {}",
                        path.display()
                    );

                    // Add IPv6 `accept-ra` config to the interface, regenerate it, and ensure the
                    // generated config contains the added IPv6 option
                    if let NetworkDConfigFile::Network(ref mut n) = config {
                        n.accept_ra();
                    }
                    let generated = config.to_string();
                    let mut path = networkd_data()
                        .join("network")
                        .join(format!("{}-ra", &name));
                    path.set_extension("network");
                    let expected = fs::read_to_string(&path).unwrap();

                    assert_eq!(
                        expected,
                        generated,
                        "Generated output does not match file: {}",
                        path.display()
                    )
                }
            }
        }
    }

    // Test the end-to-end trip: "net config -> networkd config -> serialized config"
    #[test]
    fn net_config_to_networkd_config() {
        for version in NET_CONFIG_VERSIONS {
            let temp_config = tempfile::NamedTempFile::new().unwrap();
            render_config_template(NET_CONFIG, &temp_config, version);
            let net_config = net_config::from_path(&temp_config).unwrap().unwrap();

            let networkd_config = net_config.as_networkd_config().unwrap();
            for device in networkd_config.devices {
                validate_device_config(device, &networkd_config.vlans)
            }
        }
    }

    fn validate_device_config(device: NetworkDDevice, vlans: &Vlans) {
        let configs = device.create_files(vlans);
        match device {
            NetworkDDevice::Interface(i) => validate_interface_config(i, configs),
            NetworkDDevice::Bond(b) => validate_bond_config(b, configs),
            NetworkDDevice::Vlan(v) => validate_vlan_config(v, configs),
        }
    }

    fn validate_interface_config(i: NetworkDInterface, configs: Vec<NetworkDConfigFile>) {
        let msg = format!(
            "Interfaces ({}) should create 1 .network file and 0 .netdev files",
            &i.name.to_string(),
        );

        let (networks, netdevs): (Vec<NetworkDConfigFile>, Vec<NetworkDConfigFile>) = configs
            .into_iter()
            .partition(|f| matches!(f, NetworkDConfigFile::Network(_)));

        // Interfaces should create a single network file with the interface's name
        assert!(networks.len() == 1, "{}", msg);
        assert!(netdevs.is_empty(), "{}", msg);
        for network in networks {
            validate_config_file(&i.name.to_string(), network)
        }
    }

    fn validate_vlan_config(v: NetworkDVlan, configs: Vec<NetworkDConfigFile>) {
        let msg = format!(
            "VLANs ({}) should create 1 .network file and 1 .netdev files",
            &v.name.to_string(),
        );

        let (networks, netdevs): (Vec<NetworkDConfigFile>, Vec<NetworkDConfigFile>) = configs
            .into_iter()
            .partition(|f| matches!(f, NetworkDConfigFile::Network(_)));

        // VLANs should create 1 netdev and 1 network, both named with the VLAN's name
        assert!(networks.len() == 1, "{}", msg);
        assert!(netdevs.len() == 1, "{}", msg);

        for config in networks.into_iter().chain(netdevs.into_iter()) {
            validate_config_file(&v.name.to_string(), config)
        }
    }

    fn validate_bond_config(b: NetworkDBond, configs: Vec<NetworkDConfigFile>) {
        let msg = format!(
            "Bonds ({}) should create 1 .netdev file, and enough .network files for itself and its workers",
            &b.name.to_string(),
        );

        let (networks, netdevs): (Vec<NetworkDConfigFile>, Vec<NetworkDConfigFile>) = configs
            .into_iter()
            .partition(|f| matches!(f, NetworkDConfigFile::Network(_)));

        // Bonds should create enough network interfaces for itself and its workers
        let network_count = 1 + b.interfaces.len();
        assert!(networks.len() == network_count, "{}", msg);

        // Create a Vec of all the interface names for which we expect to have a .network file
        let mut interfaces: Vec<String> = b.interfaces.iter().map(|i| i.to_string()).collect();
        interfaces.push(b.name.to_string());

        // Validate we have a config file for each of the interfaces in the above list (bond +
        // workers)
        for network in networks {
            // We know networks only contains NetworkDConfigFile::Network, but we need to access
            // the NetworkConfig inside to get at the name
            if let NetworkDConfigFile::Network(nw) = network {
                let network_name = nw.name().unwrap().to_string();

                // Ensure our list contains an interface with this name, then pop it off the list
                assert!(interfaces.contains(&network_name));
                interfaces.retain(|iface_name| iface_name != &network_name);

                validate_config_file(&network_name, NetworkDConfigFile::Network(nw))
            }
        }
        // This Vec should be empty at this point, since we removed all the interfaces we have
        // files for in the above loop
        assert!(interfaces.is_empty(), "{}", msg);

        // Bonds should create a single netdev named with the bond's name
        assert!(netdevs.len() == 1, "{}", msg);
        for netdev in netdevs {
            validate_config_file(&b.name.to_string(), netdev)
        }
    }

    fn validate_config_file(device_name: &str, config: NetworkDConfigFile) {
        // Handle MAC addresses; this also happens in the device's methods to write the config file
        // and is unit tested there.
        let device_name = device_name.to_lowercase().replace(':', "");
        let config_type = match config {
            NetworkDConfigFile::Network(_) => "network",
            NetworkDConfigFile::NetDev(_) => "netdev",
        };

        let mut path = networkd_data().join(config_type).join(device_name);
        path.set_extension(config_type);

        let expected = fs::read_to_string(&path).unwrap();
        let generated = config.to_string();
        assert_eq!(
            expected,
            generated,
            "Generated output does not match file: {}",
            path.display()
        );
    }

    fn render_config_template<P1>(template: &str, output_path: P1, version: &u8)
    where
        P1: AsRef<Path>,
    {
        #[derive(Serialize)]
        struct Context {
            version: u8,
        }

        let mut hb = Handlebars::new();
        hb.register_template_string("template", template).unwrap();

        let context = Context { version: *version };
        let rendered = hb.render("template", &context).unwrap();
        fs::write(output_path.as_ref(), rendered).unwrap()
    }
}
