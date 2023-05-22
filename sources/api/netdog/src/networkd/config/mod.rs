//! The config module contains the structures and methods needed to create properly formatted
//! systemd-networkd configuration files
mod netdev;
mod network;

use netdev::NetDevConfig;
use network::NetworkConfig;

pub(crate) enum NetworkDConfigFile {
    Network(NetworkConfig),
    NetDev(NetDevConfig),
}
