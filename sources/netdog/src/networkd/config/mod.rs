//! The config module contains the structures and methods needed to create properly formatted
//! systemd-networkd configuration files
mod netdev;
mod network;

use super::Result;
pub(crate) use netdev::{NetDevBuilder, NetDevConfig};
pub(crate) use network::{NetworkBuilder, NetworkConfig};

pub(crate) const NETWORKD_CONFIG_DIR: &str = "/etc/systemd/network";
const CONFIG_FILE_PREFIX: &str = "10-";

pub(crate) enum NetworkDConfigFile {
    Network(NetworkConfig),
    NetDev(NetDevConfig),
}

impl NetworkDConfigFile {
    pub(crate) fn write_config_file(&self) -> Result<()> {
        match self {
            NetworkDConfigFile::Network(network) => network.write_config_file(NETWORKD_CONFIG_DIR),
            NetworkDConfigFile::NetDev(netdev) => netdev.write_config_file(NETWORKD_CONFIG_DIR),
        }
    }
}

// This private module defines some empty traits meant to be used as type parameters for the
// networkd config builders.  The type parameters limit the methods that can be called on the
// builders so a user of this code can't inadvertently add configuration options that aren't
// applicable to a particular device.  For example, a user can't add bond monitoring options to a
// VLAN config.
//
// The following traits and enums are only meant to be used within the config module of this crate;
// putting them in a private module guarantees this behavior.  See the "sealed trait" pattern here:
// https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed
mod private {
    // The following zero-variant enums represent the device types we currently support.  They
    // cannot be constructed and exist only as phantom types.
    pub enum Bond {}
    pub enum Interface {}
    pub enum Vlan {}
    // Interfaces that are bound to a bond
    pub enum BondWorker {}
    // Interfaces without config, used as the link for a VLAN: typically
    // "tagged-only" setups
    pub enum VlanLink {}

    // The devices for which we are generating a configuration file.  All device types should
    // implement this trait.
    pub trait Device {}
    impl Device for Bond {}
    impl Device for Interface {}
    impl Device for Vlan {}
    impl Device for BondWorker {}
    impl Device for VlanLink {}

    // Devices not bound to a bond, i.e. everything EXCEPT BondWorker(s)
    pub trait NotBonded {}
    impl NotBonded for Bond {}
    impl NotBonded for Interface {}
    impl NotBonded for Vlan {}

    // Devices able to be members of VLANs
    pub trait CanHaveVlans {}
    impl CanHaveVlans for Bond {}
    impl CanHaveVlans for Interface {}
    impl CanHaveVlans for VlanLink {}
}

#[cfg(test)]
mod tests {
    use crate::networkd::devices::{NetworkDBond, NetworkDInterface, NetworkDVlan};
    use serde::Deserialize;
    use std::path::PathBuf;

    pub(super) const BUILDER_DATA: &str = include_str!("../../../test_data/networkd/builder.toml");

    pub(super) fn test_data() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test_data")
            .join("networkd")
    }

    #[derive(Debug, Deserialize)]
    pub(super) struct TestDevices {
        pub(super) interface: Vec<NetworkDInterface>,
        pub(super) bond: Vec<NetworkDBond>,
        pub(super) vlan: Vec<NetworkDVlan>,
    }
}
