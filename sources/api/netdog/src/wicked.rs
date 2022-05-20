//! The wicked module contains the data structures and functions needed to create network interface
//! configuration files for wicked.
//!
//! The structures in this module are meant to be created from the user-facing structures in the
//! `net_config` module.  `Default` implementations for WickedInterface exist here as well.
use crate::interface_name::InterfaceName;
use crate::net_config::{Dhcp4Config, Dhcp4Options, Dhcp6Config, Dhcp6Options, NetInterface};
use serde::Serialize;
use snafu::ResultExt;
use std::fs;
use std::path::Path;

const WICKED_CONFIG_DIR: &str = "/etc/wicked/ifconfig";
const WICKED_FILE_EXT: &str = "xml";

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename = "interface")]
pub(crate) struct WickedInterface {
    name: InterfaceName,
    control: WickedControl,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ipv4:dhcp")]
    ipv4_dhcp: Option<WickedDhcp4>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ipv6:dhcp")]
    ipv6_dhcp: Option<WickedDhcp6>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
struct WickedControl {
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    link_detection: Option<LinkDetection>,
    // TODO: `serde_xml_rs` has a known issue with serializing nested structures, where it will
    // insert additional tag with the structure name.  It has since been fixed but not released
    // officially yet.  This struct member works around that issue.
    // https://github.com/RReverser/serde-xml-rs/issues/126
    // The workaround:
    // https://stackoverflow.com/questions/70124048/how-to-create-xml-from-struct-in-rust
    #[serde(flatten, skip)]
    _f: (),
}

// We assume that all configured interfaces are wanted at boot and will require a link to
// be considered configured
impl Default for WickedControl {
    fn default() -> Self {
        WickedControl {
            mode: Some("boot".to_string()),
            link_detection: Some(LinkDetection::default()),
            _f: (),
        }
    }
}

#[derive(Default, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
struct LinkDetection {
    // This will serialize to an empty tag
    require_link: (),
    #[serde(flatten, skip)]
    _f: (),
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct WickedDhcp4 {
    enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    route_priority: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    defer_timeout: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    flags: Option<AddrConfFlags>,
    #[serde(flatten, skip)]
    _f: (),
}

impl Default for WickedDhcp4 {
    fn default() -> Self {
        WickedDhcp4 {
            enabled: true,
            route_priority: None,
            defer_timeout: None,
            flags: None,
            _f: (),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct WickedDhcp6 {
    enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    defer_timeout: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    flags: Option<AddrConfFlags>,
    #[serde(flatten, skip)]
    _f: (),
}

impl Default for WickedDhcp6 {
    fn default() -> Self {
        WickedDhcp6 {
            enabled: true,
            defer_timeout: None,
            flags: None,
            _f: (),
        }
    }
}

// This is technically an enum, but considering we don't expose anything other than "optional" to
// the user, a struct makes handling tags much simpler.
#[derive(Default, Clone, Debug, Serialize, PartialEq)]
struct AddrConfFlags {
    optional: (),
    #[serde(flatten, skip)]
    _f: (),
}

impl From<Dhcp4Config> for WickedDhcp4 {
    fn from(dhcp4: Dhcp4Config) -> Self {
        match dhcp4 {
            Dhcp4Config::DhcpEnabled(b) => WickedDhcp4 {
                enabled: b,
                _f: (),
                ..Default::default()
            },
            Dhcp4Config::WithOptions(o) => WickedDhcp4::from(o),
        }
    }
}

impl From<Dhcp4Options> for WickedDhcp4 {
    fn from(options: Dhcp4Options) -> Self {
        let mut defer_timeout = None;
        let mut flags = None;

        if options.optional == Some(true) {
            defer_timeout = Some(1);
            flags = Some(AddrConfFlags::default());
        }

        WickedDhcp4 {
            enabled: options.enabled,
            route_priority: options.route_metric,
            defer_timeout,
            flags,
            _f: (),
        }
    }
}

impl From<Dhcp6Config> for WickedDhcp6 {
    fn from(dhcp6: Dhcp6Config) -> Self {
        match dhcp6 {
            Dhcp6Config::DhcpEnabled(b) => WickedDhcp6 {
                enabled: b,
                _f: (),
                ..Default::default()
            },
            Dhcp6Config::WithOptions(o) => WickedDhcp6::from(o),
        }
    }
}

impl From<Dhcp6Options> for WickedDhcp6 {
    fn from(options: Dhcp6Options) -> Self {
        let mut defer_timeout = None;
        let mut flags = None;

        if options.optional == Some(true) {
            defer_timeout = Some(1);
            flags = Some(AddrConfFlags::default());
        }

        WickedDhcp6 {
            enabled: options.enabled,
            defer_timeout,
            flags,
            _f: (),
        }
    }
}

impl WickedInterface {
    /// Create a WickedInterface given a name and configuration
    pub(crate) fn from_config(name: InterfaceName, config: NetInterface) -> Self {
        let wicked_dhcp4 = config.dhcp4.map(WickedDhcp4::from);
        // As additional options are added for IPV6, implement `From` similar to WickedDhcp4
        let wicked_dhcp6 = config.dhcp6.map(WickedDhcp6::from);
        WickedInterface {
            name,
            control: WickedControl::default(),
            ipv4_dhcp: wicked_dhcp4,
            ipv6_dhcp: wicked_dhcp6,
        }
    }

    /// Serialize the interface's configuration file
    // Consume `self` to enforce that changes aren't made to the interface type after it has been
    // written to file
    pub(crate) fn write_config_file(&self) -> Result<()> {
        let mut cfg_path = Path::new(WICKED_CONFIG_DIR).join(self.name.to_string());
        cfg_path.set_extension(WICKED_FILE_EXT);

        // TODO: pretty print these files.  `serde_xml_rs` doesn't support pretty printing;
        // `quick_xml` does, however we require a few features that haven't been released yet to
        // properly serialize the above data structures:
        // https://github.com/tafia/quick-xml/issues/340#issuecomment-981093602
        let xml = serde_xml_rs::to_string(&self).context(error::XmlSerializeSnafu {
            interface: self.name.to_string(),
        })?;
        fs::write(&cfg_path, xml).context(error::WickedConfigWriteSnafu { path: cfg_path })
    }
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
            source: serde_xml_rs::Error,
        },
    }
}

pub(crate) use error::Error;
type Result<T> = std::result::Result<T, error::Error>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net_config::NetConfig;
    use std::path::PathBuf;
    use std::str::FromStr;

    fn test_data() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test_data")
            .join("wicked")
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
            let net_config = NetConfig::from_str(&ok_str).unwrap();

            for (name, config) in net_config.interfaces {
                let interface = WickedInterface::from_config(name, config);
                let generated = serde_xml_rs::to_string(&interface).unwrap();

                let mut path = test_data().join(interface.name.to_string());
                path.set_extension("xml");
                let expected = fs::read_to_string(path).unwrap();

                assert_eq!(expected.trim(), generated)
            }
        }
    }

    // Test the end to end trip: "net config -> wicked -> serialized XML"
    #[test]
    fn net_config_to_interface_config() {
        let net_config_str: &str = include_str!("../test_data/net_config/net_config.toml");
        let net_config: NetConfig = toml::from_str(&net_config_str).unwrap();

        for (name, config) in net_config.interfaces {
            let mut path = test_data().join(&name.to_string());
            path.set_extension("xml");
            let expected = fs::read_to_string(path).unwrap();

            let interface = WickedInterface::from_config(name, config);
            let generated = serde_xml_rs::to_string(&interface).unwrap();

            assert_eq!(expected.trim(), generated)
        }
    }
}
