use crate::interface_id::InterfaceName;
use crate::vlan_id::VlanId;
use std::fmt::Display;
use std::net::IpAddr;
use systemd_derive::{SystemdUnit, SystemdUnitSection};

#[derive(Debug, Default, SystemdUnit)]
pub(crate) struct NetDevConfig {
    netdev: Option<NetDevSection>,
    vlan: Option<VlanSection>,
    bond: Option<BondSection>,
}

#[derive(Debug, Default, SystemdUnitSection)]
#[systemd(section = "NetDev")]
struct NetDevSection {
    #[systemd(entry = "Name")]
    name: Option<InterfaceName>,
    #[systemd(entry = "Kind")]
    kind: Option<NetDevKind>,
}

#[derive(Debug, Default, SystemdUnitSection)]
#[systemd(section = "VLAN")]
struct VlanSection {
    #[systemd(entry = "Id")]
    id: Option<VlanId>,
}

#[derive(Debug, Default, SystemdUnitSection)]
#[systemd(section = "Bond")]
struct BondSection {
    #[systemd(entry = "Mode")]
    mode: Option<BondMode>,
    #[systemd(entry = "MinLinks")]
    min_links: Option<usize>,
    #[systemd(entry = "MIIMonitorSec")]
    mii_mon_secs: Option<u32>,
    #[systemd(entry = "UpDelaySec")]
    up_delay_sec: Option<u32>,
    #[systemd(entry = "DownDelaySec")]
    down_delay_sec: Option<u32>,
    #[systemd(entry = "ARPIntervalSec")]
    arp_interval_secs: Option<u32>,
    #[systemd(entry = "ARPValidate")]
    arp_validate: Option<ArpValidate>,
    #[systemd(entry = "ARPIPTargets")]
    arp_targets: Vec<IpAddr>,
    #[systemd(entry = "ARPAllTargets")]
    arp_all_targets: Option<ArpAllTargets>,
}

#[derive(Debug)]
enum NetDevKind {
    Bond,
    Vlan,
}

impl Display for NetDevKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetDevKind::Bond => write!(f, "bond"),
            NetDevKind::Vlan => write!(f, "vlan"),
        }
    }
}

#[derive(Debug)]
enum BondMode {
    ActiveBackup,
}

impl Display for BondMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BondMode::ActiveBackup => write!(f, "active-backup"),
        }
    }
}

#[derive(Debug)]
enum ArpValidate {
    Active,
    All,
    Backup,
    r#None,
}

impl Display for ArpValidate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArpValidate::Active => write!(f, "active"),
            ArpValidate::All => write!(f, "all"),
            ArpValidate::Backup => write!(f, "backup"),
            ArpValidate::r#None => write!(f, "none"),
        }
    }
}

#[derive(Debug)]
enum ArpAllTargets {
    All,
}

impl Display for ArpAllTargets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArpAllTargets::All => write!(f, "all"),
        }
    }
}
