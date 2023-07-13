//! The conversions module contains all of the trait implementations necessary to convert net
//! config structures to their corresponding networkd device structures
use super::devices::{NetworkDBond, NetworkDDevice, NetworkDInterface, NetworkDVlan};
use super::error;
use crate::interface_id::{InterfaceId, InterfaceName};
use crate::net_config::devices::bond::NetBondV1;
use crate::net_config::devices::vlan::NetVlanV1;
use crate::net_config::devices::{interface::NetInterfaceV2, NetworkDeviceV1};
use crate::net_config::NetInterfaceV1;

impl TryFrom<(InterfaceId, NetworkDeviceV1)> for NetworkDDevice {
    type Error = error::Error;

    fn try_from(value: (InterfaceId, NetworkDeviceV1)) -> Result<Self, Self::Error> {
        let (name, config) = value;
        match config {
            NetworkDeviceV1::Interface(i) => (name, i).try_into(),
            NetworkDeviceV1::BondDevice(b) => (name, b).try_into(),
            NetworkDeviceV1::VlanDevice(v) => (name, v).try_into(),
        }
    }
}

impl TryFrom<(InterfaceName, NetInterfaceV1)> for NetworkDDevice {
    type Error = error::Error;

    fn try_from(value: (InterfaceName, NetInterfaceV1)) -> Result<Self, Self::Error> {
        let (name, config) = value;
        Ok(NetworkDDevice::Interface(NetworkDInterface {
            name: name.into(),
            dhcp4: config.dhcp4,
            dhcp6: config.dhcp6,
            static4: None,
            static6: None,
            routes: None,
        }))
    }
}

impl TryFrom<(InterfaceName, NetInterfaceV2)> for NetworkDDevice {
    type Error = error::Error;

    fn try_from(value: (InterfaceName, NetInterfaceV2)) -> Result<Self, Self::Error> {
        let (name, config) = value;
        Ok(NetworkDDevice::Interface(NetworkDInterface {
            name: name.into(),
            dhcp4: config.dhcp4,
            dhcp6: config.dhcp6,
            static4: config.static4,
            static6: config.static6,
            routes: config.routes,
        }))
    }
}

impl TryFrom<(InterfaceId, NetInterfaceV2)> for NetworkDDevice {
    type Error = error::Error;

    fn try_from(value: (InterfaceId, NetInterfaceV2)) -> Result<Self, Self::Error> {
        let (name, config) = value;
        Ok(NetworkDDevice::Interface(NetworkDInterface {
            name,
            dhcp4: config.dhcp4,
            dhcp6: config.dhcp6,
            static4: config.static4,
            static6: config.static6,
            routes: config.routes,
        }))
    }
}

impl TryFrom<(InterfaceId, NetBondV1)> for NetworkDDevice {
    type Error = error::Error;

    fn try_from(value: (InterfaceId, NetBondV1)) -> Result<Self, Self::Error> {
        let (name, config) = value;
        let name = if let InterfaceId::Name(n) = name {
            n
        } else {
            return error::InvalidWithMacSnafu {
                what: "bond".to_string(),
            }
            .fail();
        };

        Ok(NetworkDDevice::Bond(NetworkDBond {
            name,
            dhcp4: config.dhcp4,
            dhcp6: config.dhcp6,
            static4: config.static4,
            static6: config.static6,
            routes: config.routes,
            mode: config.mode,
            min_links: config.min_links,
            monitoring_config: config.monitoring_config,
            interfaces: config.interfaces,
        }))
    }
}

impl TryFrom<(InterfaceId, NetVlanV1)> for NetworkDDevice {
    type Error = error::Error;

    fn try_from(value: (InterfaceId, NetVlanV1)) -> Result<Self, Self::Error> {
        let (name, config) = value;
        let name = if let InterfaceId::Name(n) = name {
            n
        } else {
            return error::InvalidWithMacSnafu {
                what: "vlan".to_string(),
            }
            .fail();
        };

        Ok(NetworkDDevice::Vlan(NetworkDVlan {
            name,
            dhcp4: config.dhcp4,
            dhcp6: config.dhcp6,
            static4: config.static4,
            static6: config.static6,
            routes: config.routes,
            device: config.device,
            id: config.id,
        }))
    }
}
