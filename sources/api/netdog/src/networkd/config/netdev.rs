use super::{CONFIG_FILE_PREFIX, NETWORKD_CONFIG_DIR};
use crate::interface_id::InterfaceName;
use crate::networkd::{error, Result};
use crate::vlan_id::VlanId;
use snafu::{OptionExt, ResultExt};
use std::fmt::Display;
use std::fs;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
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
    Any,
}

impl Display for ArpAllTargets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArpAllTargets::All => write!(f, "all"),
            ArpAllTargets::Any => write!(f, "any"),
        }
    }
}

impl NetDevConfig {
    const FILE_EXT: &str = "netdev";

    /// Write the config to the proper directory with the proper prefix and file extention
    pub(crate) fn write_config_file(&self) -> Result<()> {
        let cfg_path = self.config_path()?;

        fs::write(&cfg_path, self.to_string()).context(error::NetworkDConfigWriteSnafu {
            what: "netdev_config",
            path: cfg_path,
        })
    }

    /// Build the proper prefixed path for the config file
    fn config_path(&self) -> Result<PathBuf> {
        let device_name = &self.netdev.as_ref().and_then(|n| n.name.clone()).context(
            error::ConfigMissingNameSnafu {
                what: "netdev config".to_string(),
            },
        )?;

        let filename = format!("{}{}", CONFIG_FILE_PREFIX, device_name);
        let mut path = Path::new(NETWORKD_CONFIG_DIR).join(filename);
        path.set_extension(Self::FILE_EXT);

        Ok(path)
    }
}
