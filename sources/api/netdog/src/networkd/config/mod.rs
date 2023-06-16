//! The config module contains the structures and methods needed to create properly formatted
//! systemd-networkd configuration files
mod netdev;
mod network;

use super::Result;
use netdev::NetDevConfig;
use network::NetworkConfig;

const NETWORKD_CONFIG_DIR: &str = "/etc/systemd/network";
const CONFIG_FILE_PREFIX: &str = "10-";

pub(crate) enum NetworkDConfigFile {
    Network(NetworkConfig),
    NetDev(NetDevConfig),
}

impl NetworkDConfigFile {
    pub(crate) fn write_config_file(&self) -> Result<()> {
        match self {
            NetworkDConfigFile::Network(network) => network.write_config_file(),
            NetworkDConfigFile::NetDev(netdev) => netdev.write_config_file(),
        }
    }
}
