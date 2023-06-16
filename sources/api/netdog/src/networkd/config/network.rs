use super::{CONFIG_FILE_PREFIX, NETWORKD_CONFIG_DIR};
use crate::interface_id::{InterfaceName, MacAddress};
use crate::networkd::{error, Result};
use ipnet::IpNet;
use snafu::{OptionExt, ResultExt};
use std::fmt::Display;
use std::fs;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use systemd_derive::{SystemdUnit, SystemdUnitSection};

#[derive(Debug, Default, SystemdUnit)]
pub(crate) struct NetworkConfig {
    r#match: Option<MatchSection>,
    link: Option<LinkSection>,
    network: Option<NetworkSection>,
    route: Vec<RouteSection>,
    dhcp4: Option<Dhcp4Section>,
    dhcp6: Option<Dhcp6Section>,
}

#[derive(Debug, Default, SystemdUnitSection)]
#[systemd(section = "Match")]
struct MatchSection {
    #[systemd(entry = "Name")]
    name: Option<InterfaceName>,
    #[systemd(entry = "PermanentMACAddress")]
    permanent_mac_address: Vec<MacAddress>,
}

#[derive(Debug, Default, SystemdUnitSection)]
#[systemd(section = "Link")]
struct LinkSection {
    #[systemd(entry = "RequiredForOnline")]
    required: Option<bool>,
    #[systemd(entry = "RequiredFamilyForOnline")]
    required_family: Option<RequiredFamily>,
}

#[derive(Debug, Default, SystemdUnitSection)]
#[systemd(section = "Network")]
struct NetworkSection {
    #[systemd(entry = "Address")]
    addresses: Vec<IpNet>,
    #[systemd(entry = "Bond")]
    bond: Option<InterfaceName>,
    #[systemd(entry = "ConfigureWithoutCarrier")]
    configure_wo_carrier: Option<bool>,
    #[systemd(entry = "DHCP")]
    dhcp: Option<DhcpBool>,
    #[systemd(entry = "IPv6AcceptRA")]
    ipv6_accept_ra: Option<bool>,
    #[systemd(entry = "LinkLocalAddressing")]
    link_local_addressing: Option<DhcpBool>,
    #[systemd(entry = "PrimarySlave")]
    primary_bond_worker: Option<bool>,
    #[systemd(entry = "VLAN")]
    vlan: Vec<InterfaceName>,
}

#[derive(Debug, Default, SystemdUnitSection)]
#[systemd(section = "Route")]
struct RouteSection {
    #[systemd(entry = "Destination")]
    destination: Option<IpNet>,
    #[systemd(entry = "Gateway")]
    gateway: Option<IpAddr>,
    #[systemd(entry = "Metric")]
    metric: Option<u32>,
    #[systemd(entry = "PreferredSource")]
    preferred_source: Option<IpAddr>,
}

#[derive(Debug, Default, SystemdUnitSection)]
#[systemd(section = "DHCPv4")]
struct Dhcp4Section {
    #[systemd(entry = "RouteMetric")]
    metric: Option<u32>,
    #[systemd(entry = "UseDNS")]
    use_dns: Option<bool>,
    #[systemd(entry = "UseDomains")]
    use_domains: Option<bool>,
}

#[derive(Debug, Default, SystemdUnitSection)]
#[systemd(section = "DHCPv6")]
struct Dhcp6Section {
    #[systemd(entry = "UseDNS")]
    use_dns: Option<bool>,
    #[systemd(entry = "UseDomains")]
    use_domains: Option<bool>,
}

#[derive(Debug)]
enum RequiredFamily {
    Any,
    Both,
    Ipv4,
    Ipv6,
}

impl Display for RequiredFamily {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequiredFamily::Any => write!(f, "any"),
            RequiredFamily::Both => write!(f, "both"),
            RequiredFamily::Ipv4 => write!(f, "ipv4"),
            RequiredFamily::Ipv6 => write!(f, "ipv6"),
        }
    }
}

#[derive(Debug)]
enum DhcpBool {
    Ipv4,
    Ipv6,
    No,
    Yes,
}

impl Display for DhcpBool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DhcpBool::Ipv4 => write!(f, "ipv4"),
            DhcpBool::Ipv6 => write!(f, "ipv6"),
            DhcpBool::No => write!(f, "no"),
            DhcpBool::Yes => write!(f, "yes"),
        }
    }
}
impl NetworkConfig {
    const FILE_EXT: &str = "network";

    /// Write the config to the proper directory with the proper prefix and file extention
    pub(crate) fn write_config_file(&self) -> Result<()> {
        let cfg_path = self.config_path()?;

        fs::write(&cfg_path, self.to_string()).context(error::NetworkDConfigWriteSnafu {
            what: "network config",
            path: cfg_path,
        })
    }
    /// Build the proper prefixed path for the config file
    fn config_path(&self) -> Result<PathBuf> {
        let match_section = self
            .r#match
            .as_ref()
            .context(error::ConfigMissingNameSnafu {
                what: "network config".to_string(),
            })?;

        // Choose the device name for the filename if it exists, otherwise use the MAC
        let device_name = match (
            &match_section.name,
            match_section.permanent_mac_address.first(),
        ) {
            (Some(name), _) => name.to_string(),
            (None, Some(mac)) => mac.to_string().replace(':', ""),
            (None, None) => {
                return error::ConfigMissingNameSnafu {
                    what: "network_config".to_string(),
                }
                .fail();
            }
        };

        let filename = format!("{}{}", CONFIG_FILE_PREFIX, device_name);
        let mut cfg_path = Path::new(NETWORKD_CONFIG_DIR).join(filename);
        cfg_path.set_extension(Self::FILE_EXT);
        Ok(cfg_path)
    }
}
