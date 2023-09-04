//! The device module contains the structures representing the latest version of configuration for
//! interfaces, bonds, and VLANs.
mod bond;
mod interface;
mod vlan;

use super::config::NetworkDConfigFile;
use super::{NetDevFileCreator, NetworkFileCreator, Vlans};
use crate::interface_id::InterfaceId;
pub(crate) use bond::NetworkDBond;
pub(crate) use interface::NetworkDInterface;
pub(crate) use vlan::NetworkDVlan;

pub(crate) enum NetworkDDevice {
    Interface(NetworkDInterface),
    Bond(NetworkDBond),
    Vlan(NetworkDVlan),
}

impl NetworkDDevice {
    pub(super) fn create_files(&self, vlans: &Vlans) -> Vec<NetworkDConfigFile> {
        let mut configs = Vec::new();

        match self {
            NetworkDDevice::Interface(i) => {
                configs.extend(
                    i.create_networks(vlans)
                        .into_iter()
                        .map(NetworkDConfigFile::Network),
                );
            }
            NetworkDDevice::Bond(b) => {
                configs.push(NetworkDConfigFile::NetDev(b.create_netdev()));
                configs.extend(
                    b.create_networks(vlans)
                        .into_iter()
                        .map(NetworkDConfigFile::Network),
                );
            }
            NetworkDDevice::Vlan(v) => {
                configs.push(NetworkDConfigFile::NetDev(v.create_netdev()));
                configs.extend(
                    v.create_networks(vlans)
                        .into_iter()
                        .map(NetworkDConfigFile::Network),
                );
            }
        };

        configs
    }

    pub(super) fn name(&self) -> InterfaceId {
        match self {
            NetworkDDevice::Interface(i) => i.name.clone(),
            NetworkDDevice::Bond(b) => b.name.clone().into(),
            NetworkDDevice::Vlan(v) => v.name.clone().into(),
        }
    }
}

/// A tiny macro to ease calling builder methods only if an `Option` is `Some()`.  NetworkDDevices
/// contain quite a few `Option`s (and probably more in the future), so when driving the config
/// builders there becomes a bit of `if let Some()` boilerplate.  This macro reduces that
/// boilerplate to a single line.
///
/// The first argument is the builder, the second is the builder method, and the third is the
/// Option from which you need to add.
macro_rules! maybe_add_some {
    ($builder:ident, $method:ident, $option:ident) => {
        if let Some(thing) = $option.clone() {
            $builder.$method(thing)
        }
    };
}
use maybe_add_some;
