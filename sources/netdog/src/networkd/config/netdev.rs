use super::private::{Bond, Device, Vlan};
use super::CONFIG_FILE_PREFIX;
use crate::bonding::{ArpMonitoringConfigV1, ArpValidateV1, BondModeV1, MiiMonitoringConfigV1};
use crate::interface_id::{InterfaceName, MacAddress};
use crate::networkd::{error, Result};
use crate::vlan_id::VlanId;
use snafu::{OptionExt, ResultExt};
use std::fmt::Display;
use std::fs;
use std::marker::PhantomData;
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
    #[systemd(entry = "MACAddress")]
    mac_address: Option<NetDevMacAddress>,
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

// The `All` variant isn't currently used, but is valid
#[derive(Debug)]
enum ArpAllTargets {
    #[allow(dead_code)]
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

// NetDev has a special `none` which allows bonds to reuse the permanent address for the bond
#[derive(Debug)]
enum NetDevMacAddress {
    #[allow(dead_code)]
    MacAddress(MacAddress),
    Nothing,
}

impl Display for NetDevMacAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetDevMacAddress::MacAddress(m) => write!(f, "{}", m),
            NetDevMacAddress::Nothing => write!(f, "none"),
        }
    }
}

impl NetDevConfig {
    const FILE_EXT: &str = "netdev";

    /// Write the config to the proper directory with the proper prefix and file extention
    pub(crate) fn write_config_file<P: AsRef<Path>>(&self, config_dir: P) -> Result<()> {
        let cfg_path = self.config_path(config_dir)?;

        fs::write(&cfg_path, self.to_string()).context(error::NetworkDConfigWriteSnafu {
            what: "netdev_config",
            path: cfg_path,
        })
    }

    /// Build the proper prefixed path for the config file
    fn config_path<P: AsRef<Path>>(&self, config_dir: P) -> Result<PathBuf> {
        let device_name = &self.netdev.as_ref().and_then(|n| n.name.clone()).context(
            error::ConfigMissingNameSnafu {
                what: "netdev config".to_string(),
            },
        )?;

        let filename = format!("{}{}", CONFIG_FILE_PREFIX, device_name);
        let mut path = Path::new(config_dir.as_ref()).join(filename);
        path.set_extension(Self::FILE_EXT);

        Ok(path)
    }

    // The following *mut() methods are private and primarily meant for use by the NetDevBuilder.
    // They are convenience methods to access the referenced structs (which are `Option`s) since
    // they may need to be accessed in multiple places during the builder's construction process.
    // (And no one wants to call `get_or_insert_with()` everywhere)
    fn vlan_mut(&mut self) -> &mut VlanSection {
        self.vlan.get_or_insert_with(VlanSection::default)
    }

    fn bond_mut(&mut self) -> &mut BondSection {
        self.bond.get_or_insert_with(BondSection::default)
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=
//
/// The builder for `NetDevConfig`.
//
// Why a builder?  Great question.  As you can see below, some logic is involved to translate
// config struct fields to a valid NetDevConfig.  Since `NetDevConfig` will be created by multiple
// devices (bonds and VLANs to start), it makes sense to centralize that logic to avoid
// duplication/mistakes.  Using a builder means type parameters can be used to limit available
// methods based on the device being created.  Putting the type parameter on the builder and not
// NetDevConfig avoids proliferating the type parameter everywhere NetDevConfig may be used.
#[derive(Debug)]
pub(crate) struct NetDevBuilder<T: Device> {
    netdev: NetDevConfig,
    spooky: PhantomData<T>,
}

impl<T: Device> NetDevBuilder<T> {
    pub(crate) fn build(self) -> NetDevConfig {
        self.netdev
    }
}

impl NetDevBuilder<Bond> {
    /// Create a new .netdev config for a bond.
    pub(crate) fn new_bond(name: InterfaceName) -> Self {
        let netdev = NetDevConfig {
            netdev: Some(NetDevSection {
                name: Some(name),
                kind: Some(NetDevKind::Bond),
                mac_address: Some(NetDevMacAddress::Nothing),
            }),
            ..Default::default()
        };

        Self {
            netdev,
            spooky: PhantomData,
        }
    }

    /// Add bond mode
    pub(crate) fn with_mode(&mut self, mode: BondModeV1) {
        self.netdev.bond_mut().mode = match mode {
            BondModeV1::ActiveBackup => Some(BondMode::ActiveBackup),
        }
    }

    /// Add bond minimum links
    pub(crate) fn with_min_links(&mut self, min_links: usize) {
        self.netdev.bond_mut().min_links = Some(min_links)
    }

    /// Add MIIMon configuration
    pub(crate) fn with_miimon_config(&mut self, miimon: MiiMonitoringConfigV1) {
        let bond = self.netdev.bond_mut();

        bond.mii_mon_secs = Some(miimon.frequency);
        bond.up_delay_sec = Some(miimon.updelay);
        bond.down_delay_sec = Some(miimon.downdelay);
    }

    /// Add ARPMon configuration
    pub(crate) fn with_arpmon_config(&mut self, arpmon: ArpMonitoringConfigV1) {
        let bond = self.netdev.bond_mut();

        // Legacy alert: wicked defaults to "any", keep that default here
        // TODO: add a setting for this
        bond.arp_all_targets = Some(ArpAllTargets::Any);
        bond.arp_interval_secs = Some(arpmon.interval);
        bond.arp_targets.extend(arpmon.targets);
        bond.arp_validate = match arpmon.validate {
            ArpValidateV1::Active => Some(ArpValidate::Active),
            ArpValidateV1::All => Some(ArpValidate::All),
            ArpValidateV1::Backup => Some(ArpValidate::Backup),
            ArpValidateV1::None => Some(ArpValidate::r#None),
        };
    }
}

impl NetDevBuilder<Vlan> {
    /// Create a new .netdev config for a VLAN
    pub(crate) fn new_vlan(name: InterfaceName) -> Self {
        let netdev = NetDevConfig {
            netdev: Some(NetDevSection {
                name: Some(name),
                kind: Some(NetDevKind::Vlan),
                mac_address: None,
            }),
            ..Default::default()
        };

        Self {
            netdev,
            spooky: PhantomData,
        }
    }

    /// Add the VLAN's ID
    pub(crate) fn with_vlan_id(&mut self, id: VlanId) {
        self.netdev.vlan_mut().id = Some(id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bonding::BondMonitoringConfigV1;
    use crate::networkd::config::tests::{test_data, TestDevices, BUILDER_DATA};
    use crate::networkd::devices::{NetworkDBond, NetworkDVlan};

    const FAKE_TEST_DIR: &str = "testdir";

    fn netdev_path(name: String) -> PathBuf {
        test_data().join("netdev").join(format!("{}.netdev", name))
    }

    fn netdev_from_bond(bond: NetworkDBond) -> NetDevConfig {
        let mut netdev = NetDevBuilder::new_bond(bond.name.clone());
        netdev.with_mode(bond.mode);
        if let Some(m) = bond.min_links {
            netdev.with_min_links(m)
        }
        match bond.monitoring_config {
            BondMonitoringConfigV1::MiiMon(miimon) => netdev.with_miimon_config(miimon),
            BondMonitoringConfigV1::ArpMon(arpmon) => netdev.with_arpmon_config(arpmon),
        }
        netdev.build()
    }

    fn netdev_from_vlan(vlan: NetworkDVlan) -> NetDevConfig {
        let mut netdev = NetDevBuilder::new_vlan(vlan.name.clone());
        netdev.with_vlan_id(vlan.id);
        netdev.build()
    }

    #[test]
    fn bond_netdev_builder() {
        let devices = toml::from_str::<TestDevices>(BUILDER_DATA).unwrap();
        for bond in devices.bond {
            let expected_filename = netdev_path(bond.name.to_string());
            let expected = fs::read_to_string(expected_filename).unwrap();
            let got = netdev_from_bond(bond).to_string();

            assert_eq!(expected, got)
        }
    }

    #[test]
    fn vlan_netdev_builder() {
        let devices = toml::from_str::<TestDevices>(BUILDER_DATA).unwrap();
        for vlan in devices.vlan {
            let expected_filename = netdev_path(vlan.name.to_string());
            let expected = fs::read_to_string(expected_filename).unwrap();
            let got = netdev_from_vlan(vlan).to_string();

            assert_eq!(expected, got)
        }
    }

    #[test]
    fn config_path_empty() {
        let netdev = NetDevConfig::default();
        assert!(netdev.config_path(FAKE_TEST_DIR).is_err())
    }

    #[test]
    fn config_path_name() {
        let filename = format!("{}foo", CONFIG_FILE_PREFIX);
        let mut expected = Path::new(FAKE_TEST_DIR).join(filename);
        expected.set_extension(NetDevConfig::FILE_EXT);

        let netdev = NetDevConfig {
            netdev: Some(NetDevSection {
                name: Some(InterfaceName::try_from("foo").unwrap()),
                ..Default::default()
            }),
            ..Default::default()
        };

        assert_eq!(expected, netdev.config_path(FAKE_TEST_DIR).unwrap())
    }
}
