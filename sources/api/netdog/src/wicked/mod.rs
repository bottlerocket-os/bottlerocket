//! The wicked module contains the data structures and functions needed to create network interface
//! configuration files for wicked.
//!
//! The structures in this module are meant to be created from the user-facing structures in the
//! `net_config` module.  `Default` implementations for WickedInterface exist here as well.
mod bonding;
mod dhcp;
mod static_address;
mod vlan;

use crate::bonding::BondMonitoringConfigV1;
use crate::interface_id::{InterfaceId, InterfaceName, MacAddress};
use crate::net_config::devices::bond::NetBondV1;
use crate::net_config::devices::interface::NetInterfaceV2;
use crate::net_config::devices::vlan::NetVlanV1;
use crate::net_config::devices::NetworkDeviceV1;
use crate::wicked::bonding::{
    WickedArpMonitoringConfig, WickedBondMode, WickedMiiMonitoringConfig,
};
use bonding::WickedBond;
pub(crate) use dhcp::{WickedDhcp4, WickedDhcp6};
pub(crate) use error::Error;
use serde::Serialize;
use snafu::ResultExt;
pub(crate) use static_address::{WickedRoutes, WickedStaticAddress};
use std::fmt::{self, Display};
use std::fs;
use std::path::Path;
use vlan::WickedVlanTag;

const WICKED_CONFIG_DIR: &str = "/etc/wicked/ifconfig";
const WICKED_FILE_EXT: &str = "xml";

macro_rules! wicked_from {
    ($name:ident, $config:ident) => {
        ({
            let mut wicked_interface = WickedInterface::new($name.clone());
            wicked_interface.ipv4_dhcp = $config.dhcp4.clone().map(WickedDhcp4::from);
            wicked_interface.ipv6_dhcp = $config.dhcp6.clone().map(WickedDhcp6::from);

            // Based on the existence of static addresses and routes, create the ipv4/6_static
            // struct members.  They must be `Option`s because we want to avoid serializing empty
            // tags into the config file
            let maybe_routes = $config.routes.clone().map(WickedRoutes::from);
            let maybe_ipv4_static = WickedStaticAddress::maybe_new(
                $config.static4.clone(),
                maybe_routes.as_ref().and_then(|s| s.ipv4.clone()),
            );
            let maybe_ipv6_static = WickedStaticAddress::maybe_new(
                $config.static6.clone(),
                maybe_routes.as_ref().and_then(|s| s.ipv6.clone()),
            );
            wicked_interface.ipv4_static = maybe_ipv4_static;
            wicked_interface.ipv6_static = maybe_ipv6_static;

            wicked_interface
        }) as WickedInterface
    };
}

pub(crate) use wicked_from;

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename = "interface")]
pub(crate) struct WickedInterface {
    pub(crate) name: WickedName,
    pub(crate) control: WickedControl,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ipv4:dhcp")]
    pub(crate) ipv4_dhcp: Option<WickedDhcp4>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ipv6:dhcp")]
    pub(crate) ipv6_dhcp: Option<WickedDhcp6>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ipv4:static")]
    pub(crate) ipv4_static: Option<WickedStaticAddress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ipv6:static")]
    pub(crate) ipv6_static: Option<WickedStaticAddress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ipv6")]
    pub(crate) ipv6: Option<WickedIpv6>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "vlan")]
    pub(crate) vlan_tag: Option<WickedVlanTag>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "bond")]
    pub(crate) bond: Option<WickedBond>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) link: Option<WickedLinkConfig>,
}

#[derive(Debug, Serialize, PartialEq)]
pub(crate) struct WickedName {
    #[serde(skip_serializing_if = "Option::is_none")]
    namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "$value")]
    name_body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "$unflatten=permanent-address")]
    permanent_address: Option<MacAddress>,
}

impl WickedName {
    pub(crate) fn new(id: InterfaceId) -> Self {
        // When using a MAC address as an identifier, the resulting XML must be in a different
        // format.
        // When using a name, the resulting XML looks like:
        // <name>eth0</name>
        // Using a MAC address looks like:
        // <name namespace="ethernet"><permanent-address>...</permanent-address></name>
        match id {
            InterfaceId::Name(name) => Self {
                namespace: None,
                name_body: Some(name.to_string()),
                permanent_address: None,
            },
            InterfaceId::MacAddress(mac) => Self {
                namespace: Some("ethernet".to_string()),
                name_body: None,
                permanent_address: Some(mac),
            },
        }
    }
}

impl Display for WickedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.name_body.as_ref(), self.permanent_address.as_ref()) {
            (Some(_), Some(_)) => Err(fmt::Error),
            (Some(name), None) => write!(f, "{}", name),
            (None, Some(mac)) => write!(f, "{}", mac.to_string().replace(':', "")),
            (None, None) => Err(fmt::Error),
        }
    }
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct WickedControl {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "$unflatten=mode")]
    mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    link_detection: Option<LinkDetection>,
}

// We assume that all configured interfaces are wanted at boot and will require a link to
// be considered configured
impl Default for WickedControl {
    fn default() -> Self {
        WickedControl {
            mode: Some("boot".to_string()),
            link_detection: Some(LinkDetection::default()),
        }
    }
}

#[derive(Default, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
struct LinkDetection {
    // This will serialize to an empty tag
    #[serde(rename = "$unflatten=require-link")]
    require_link: (),
}

#[derive(Debug, Serialize, PartialEq)]
pub(crate) struct WickedIpv6 {
    #[serde(rename = "$unflatten=accept-ra")]
    accept_ra: WickedIpv6AcceptRA,
}

// There are technically a few options here, but currently we only use "router"
#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) enum WickedIpv6AcceptRA {
    #[serde(rename = "$primitive=router")]
    Router,
}

impl Default for WickedIpv6 {
    fn default() -> Self {
        WickedIpv6 {
            accept_ra: WickedIpv6AcceptRA::Router,
        }
    }
}

impl WickedInterface {
    pub(crate) fn new<I>(id: I) -> Self
    where
        I: Into<InterfaceId>,
    {
        let name_node = WickedName::new(id.into());
        Self {
            name: name_node,
            control: WickedControl::default(),
            ipv4_dhcp: None,
            ipv6_dhcp: None,
            ipv4_static: None,
            ipv6_static: None,
            ipv6: None,
            vlan_tag: None,
            bond: None,
            link: None,
        }
    }

    /// Add config to accept IPv6 router advertisements
    // TODO: expose a network config option for this
    pub(crate) fn accept_ra(&mut self) {
        self.ipv6 = Some(WickedIpv6::default())
    }

    /// Serialize the interface's configuration file
    pub(crate) fn write_config_file(&self) -> Result<()> {
        let mut cfg_path = Path::new(WICKED_CONFIG_DIR).join(self.name.to_string());
        cfg_path.set_extension(WICKED_FILE_EXT);

        let xml = quick_xml::se::to_string(&self).context(error::XmlSerializeSnafu {
            interface: self.name.to_string(),
        })?;
        fs::write(&cfg_path, xml).context(error::WickedConfigWriteSnafu { path: cfg_path })
    }
}

impl<T> From<(&T, &NetworkDeviceV1)> for WickedInterface
where
    T: Into<InterfaceId> + Clone,
{
    fn from(device_tup: (&T, &NetworkDeviceV1)) -> Self {
        match device_tup.1 {
            NetworkDeviceV1::Interface(i) => WickedInterface::from((device_tup.0, i)),
            NetworkDeviceV1::BondDevice(b) => WickedInterface::from((device_tup.0, b)),
            NetworkDeviceV1::VlanDevice(v) => WickedInterface::from((device_tup.0, v)),
        }
    }
}

impl<T> From<(&T, &NetInterfaceV2)> for WickedInterface
where
    T: Into<InterfaceId> + Clone,
{
    fn from(device_tup: (&T, &NetInterfaceV2)) -> Self {
        let name = device_tup.0;
        let config = device_tup.1;
        wicked_from!(name, config)
    }
}

impl<T> From<(&T, &NetBondV1)> for WickedInterface
where
    T: Into<InterfaceId> + Clone,
{
    fn from(device_tup: (&T, &NetBondV1)) -> Self {
        let name = device_tup.0;
        let config = device_tup.1;
        let mut wicked_interface = wicked_from!(name, config);

        // Here is where bonding specific things begin
        let mut wicked_bond = WickedBond::new(
            WickedBondMode::from(config.mode.clone()),
            config.interfaces.clone(),
        );

        wicked_bond.min_links = config.min_links;

        match &config.monitoring_config {
            BondMonitoringConfigV1::MiiMon(config) => {
                wicked_bond.mii_monitoring = Some(WickedMiiMonitoringConfig::from(config.clone()))
            }
            BondMonitoringConfigV1::ArpMon(config) => {
                wicked_bond.arp_monitoring = Some(WickedArpMonitoringConfig::from(config.clone()))
            }
        }

        wicked_interface.bond = Some(wicked_bond);

        wicked_interface
    }
}

impl<T> From<(&T, &NetVlanV1)> for WickedInterface
where
    T: Into<InterfaceId> + Clone,
{
    fn from(device_tup: (&T, &NetVlanV1)) -> Self {
        let name = device_tup.0;
        let config = device_tup.1;
        let mut wicked_interface = wicked_from!(name, config);

        wicked_interface.vlan_tag = Some(WickedVlanTag::new(config.device.clone(), *config.id));

        wicked_interface
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct WickedLinkConfig {
    #[serde(rename = "$unflatten=master")]
    pub(crate) master: InterfaceName,
}

mod error {
    use snafu::Snafu;
    use std::io;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(crate)))]
    pub(crate) enum Error {
        #[snafu(display("Failed to write network configuration to '{}': {}", path.display(), source))]
        WickedConfigWrite { path: PathBuf, source: io::Error },

        #[snafu(display("Error serializing config for '{}' to XML: {}", interface, source))]
        XmlSerialize {
            interface: String,
            source: quick_xml::DeError,
        },
    }
}

type Result<T> = std::result::Result<T, error::Error>;

#[cfg(net_backend = "wicked")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::net_config::{self, Interfaces, NetConfigV1};
    use handlebars::Handlebars;
    use serde::Serialize;
    use std::path::PathBuf;
    use std::str::FromStr;

    static NET_CONFIG_VERSIONS: &[u8] = &[1, 2, 3];

    fn test_data() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data")
    }

    fn wicked_config() -> PathBuf {
        test_data().join("wicked")
    }

    // Test the end-to-end trip: "net config from cmdline -> wicked -> serialized XML"
    #[test]
    fn interface_config_from_str() {
        // Interface names here coincide with config files, some of which are shared with the
        // `net_config` test below
        let ok = &[
            "eno1:dhcp4",
            "eno2:dhcp6",
            "eno9:dhcp4?",
            "eno10:dhcp6?",
            "eno5:dhcp4,dhcp6",
            "eno5:dhcp6,dhcp4",
            "eno7:dhcp4,dhcp6?",
            "eno7:dhcp6?,dhcp4",
            "eno8:dhcp6?,dhcp4?",
            "eno8:dhcp4?,dhcp6?",
        ];
        for ok_str in ok {
            let net_config = NetConfigV1::from_str(ok_str).unwrap();

            let mut wicked_interfaces = net_config.as_wicked_interfaces();
            for interface in &mut wicked_interfaces {
                let generated = quick_xml::se::to_string(&interface).unwrap();
                let mut path = wicked_config().join(interface.name.to_string());
                path.set_extension("xml");
                let expected = fs::read_to_string(&path).unwrap();

                assert_eq!(
                    expected.trim(),
                    generated,
                    "Generated output does not match file: {}",
                    path.display()
                );

                // Add IPv6 `accept-ra` config to the interface, regenerate it, and ensure the
                // generated config contains the added IPv6 option
                interface.accept_ra();
                let generated = quick_xml::se::to_string(&interface).unwrap();
                let mut path = wicked_config().join(format!("{}-ra", interface.name));
                path.set_extension("xml");
                let expected = fs::read_to_string(&path).unwrap();

                assert_eq!(
                    expected.trim(),
                    generated,
                    "Generated output does not match file: {}",
                    path.display()
                )
            }
        }
    }

    // Test the end to end trip: "net config -> wicked -> serialized XML"
    #[test]
    #[allow(clippy::to_string_in_format_args)]
    fn net_config_to_interface_config() {
        let net_config_path = test_data().join("net_config.toml");

        for version in NET_CONFIG_VERSIONS {
            let temp_config = tempfile::NamedTempFile::new().unwrap();

            render_config_template(&net_config_path, &temp_config, version);
            let net_config = net_config::from_path(&temp_config).unwrap().unwrap();
            let wicked_interfaces = net_config.as_wicked_interfaces();
            for interface in wicked_interfaces {
                let mut path = wicked_config().join(interface.name.to_string());
                path.set_extension("xml");
                let generated = quick_xml::se::to_string(&interface).unwrap();
                dbg!(&generated);
                let expected = fs::read_to_string(path).unwrap();

                assert_eq!(
                    expected.trim(),
                    generated,
                    "failed test for net config version: '{}', interface: '{}'",
                    version,
                    interface.name.to_string()
                )
            }
        }
    }

    fn render_config_template<P1, P2>(template_path: P1, output_path: P2, version: &u8)
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>,
    {
        #[derive(Serialize)]
        struct Context {
            version: u8,
        }

        let output_path = output_path.as_ref();
        let template_path = template_path.as_ref();
        let template_str = fs::read_to_string(template_path).unwrap();

        let mut hb = Handlebars::new();
        hb.register_template_string("template", &template_str)
            .unwrap();

        let context = Context { version: *version };
        let rendered = hb.render("template", &context).unwrap();
        fs::write(output_path, rendered).unwrap()
    }
}
