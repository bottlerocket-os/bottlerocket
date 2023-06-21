//! The device module contains the structures representing the latest version of configuration for
//! interfaces, bonds, and VLANs.
mod bond;
mod interface;
mod vlan;

pub(crate) use bond::NetworkDBond;
pub(crate) use interface::NetworkDInterface;
pub(crate) use vlan::NetworkDVlan;

pub(crate) enum NetworkDDevice {
    Interface(NetworkDInterface),
    Bond(NetworkDBond),
    Vlan(NetworkDVlan),
}
