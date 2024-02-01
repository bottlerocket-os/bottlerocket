use crate::bonding::{ArpMonitoringConfigV1, ArpValidateV1, BondModeV1, MiiMonitoringConfigV1};
use crate::interface_id::InterfaceName;
use serde::Serialize;
use std::net::IpAddr;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct WickedBond {
    #[serde(rename = "$unflatten=mode")]
    mode: WickedBondMode,
    #[serde(rename = "slaves")]
    devices: SubDevices,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "miimon")]
    pub(crate) mii_monitoring: Option<WickedMiiMonitoringConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "arpmon")]
    pub(crate) arp_monitoring: Option<WickedArpMonitoringConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "$unflatten=min-links")]
    pub(crate) min_links: Option<usize>,
}

impl WickedBond {
    pub(crate) fn new(mode: WickedBondMode, devices: Vec<InterfaceName>) -> Self {
        let mut sub_devices: Vec<SubDevice> = Vec::new();
        // The first device is primary, the rest are not
        let mut devices_iter = devices.iter();
        if let Some(primary_device) = devices_iter.next() {
            sub_devices.push(SubDevice {
                device: primary_device.clone(),
                primary: Some(true),
            });
            for name in devices_iter {
                sub_devices.push(SubDevice {
                    device: name.clone(),
                    primary: None,
                })
            }
        }

        let s = SubDevices {
            devices: sub_devices,
        };
        Self {
            mode,
            devices: s,
            mii_monitoring: None,
            arp_monitoring: None,
            min_links: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) enum WickedBondMode {
    #[serde(rename = "$primitive=active-backup")]
    PrimaryBackup,
}

impl From<BondModeV1> for WickedBondMode {
    fn from(mode: BondModeV1) -> Self {
        match mode {
            BondModeV1::ActiveBackup => WickedBondMode::PrimaryBackup,
        }
    }
}

impl From<MiiMonitoringConfigV1> for WickedMiiMonitoringConfig {
    fn from(config: MiiMonitoringConfigV1) -> Self {
        WickedMiiMonitoringConfig {
            frequency: config.frequency,
            updelay: config.updelay,
            downdelay: config.downdelay,
            carrier_detect: 1,
        }
    }
}

impl From<ArpMonitoringConfigV1> for WickedArpMonitoringConfig {
    fn from(config: ArpMonitoringConfigV1) -> Self {
        let mut t_vec = Vec::new();
        for t in config.targets {
            t_vec.push(ArpTarget(t))
        }
        let targets = ArpTargets { t: t_vec };

        WickedArpMonitoringConfig {
            interval: config.interval,
            validate: WickedArpValidate::from(config.validate),
            targets,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct SubDevices {
    #[serde(rename = "slave")]
    devices: Vec<SubDevice>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct SubDevice {
    #[serde(rename = "$unflatten=device")]
    device: InterfaceName,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "$unflatten=primary")]
    primary: Option<bool>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct WickedMiiMonitoringConfig {
    #[serde(rename = "$unflatten=frequency")]
    frequency: u32,
    #[serde(rename = "$unflatten=updelay")]
    updelay: u32,
    #[serde(rename = "$unflatten=downdelay")]
    downdelay: u32,
    #[serde(rename = "$unflatten=carrier-detect")]
    carrier_detect: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct WickedArpMonitoringConfig {
    #[serde(rename = "$unflatten=interval")]
    interval: u32,
    #[serde(rename = "$unflatten=validate")]
    validate: WickedArpValidate,
    targets: ArpTargets,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) enum WickedArpValidate {
    #[serde(rename = "$primitive=active")]
    Active,
    #[serde(rename = "$primitive=all")]
    All,
    #[serde(rename = "$primitive=backup")]
    Backup,
    #[serde(rename = "$primitive=none")]
    None,
}

impl From<ArpValidateV1> for WickedArpValidate {
    fn from(validate: ArpValidateV1) -> Self {
        match validate {
            ArpValidateV1::Active => WickedArpValidate::Active,
            ArpValidateV1::All => WickedArpValidate::All,
            ArpValidateV1::Backup => WickedArpValidate::Backup,
            ArpValidateV1::None => WickedArpValidate::None,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct ArpTargets {
    t: Vec<ArpTarget>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct ArpTarget(IpAddr);
