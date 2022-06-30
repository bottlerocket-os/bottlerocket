//! The wicked module contains the data structures and functions needed to create network interface
//! configuration files for wicked.
//!
//! The structures in this module are meant to be created from the user-facing structures in the
//! `net_config` module.  `Default` implementations for WickedInterface exist here as well.
use crate::interface_name::InterfaceName;
use crate::net_config::{Dhcp4ConfigV1, Dhcp4OptionsV1, Dhcp6ConfigV1, Dhcp6OptionsV1};
use serde::Serialize;
use snafu::ResultExt;
use std::fs;
use std::path::Path;

const WICKED_CONFIG_DIR: &str = "/etc/wicked/ifconfig";
const WICKED_FILE_EXT: &str = "xml";

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename = "interface")]
pub(crate) struct WickedInterface {
    pub(crate) name: InterfaceName,
    pub(crate) control: WickedControl,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ipv4:dhcp")]
    pub(crate) ipv4_dhcp: Option<WickedDhcp4>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ipv6:dhcp")]
    pub(crate) ipv6_dhcp: Option<WickedDhcp6>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct WickedControl {
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

impl From<Dhcp4ConfigV1> for WickedDhcp4 {
    fn from(dhcp4: Dhcp4ConfigV1) -> Self {
        match dhcp4 {
            Dhcp4ConfigV1::DhcpEnabled(b) => WickedDhcp4 {
                enabled: b,
                _f: (),
                ..Default::default()
            },
            Dhcp4ConfigV1::WithOptions(o) => WickedDhcp4::from(o),
        }
    }
}

impl From<Dhcp4OptionsV1> for WickedDhcp4 {
    fn from(options: Dhcp4OptionsV1) -> Self {
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

impl From<Dhcp6ConfigV1> for WickedDhcp6 {
    fn from(dhcp6: Dhcp6ConfigV1) -> Self {
        match dhcp6 {
            Dhcp6ConfigV1::DhcpEnabled(b) => WickedDhcp6 {
                enabled: b,
                _f: (),
                ..Default::default()
            },
            Dhcp6ConfigV1::WithOptions(o) => WickedDhcp6::from(o),
        }
    }
}

impl From<Dhcp6OptionsV1> for WickedDhcp6 {
    fn from(options: Dhcp6OptionsV1) -> Self {
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
    /// Serialize the interface's configuration file
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
    use crate::net_config::{self, Interfaces, NetConfigV1};
    use std::path::PathBuf;
    use std::str::FromStr;

    fn test_data() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data")
    }

    fn wicked_config() -> PathBuf {
        test_data().join("wicked")
    }

    fn net_config() -> PathBuf {
        test_data().join("net_config")
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
            let net_config = NetConfigV1::from_str(&ok_str).unwrap();

            let wicked_interfaces = net_config.into_wicked_interfaces();
            for interface in wicked_interfaces {
                let generated = serde_xml_rs::to_string(&interface).unwrap();

                let mut path = wicked_config().join(interface.name.to_string());
                path.set_extension("xml");
                let expected = fs::read_to_string(path).unwrap();

                assert_eq!(expected.trim(), generated)
            }
        }
    }

    // Test the end to end trip: "net config -> wicked -> serialized XML"
    #[test]
    fn net_config_to_interface_config() {
        let net_config_path = net_config().join("net_config.toml");
        let net_config = net_config::from_path(&net_config_path).unwrap().unwrap();

        let wicked_interfaces = net_config.into_wicked_interfaces();
        for interface in wicked_interfaces {
            let mut path = wicked_config().join(interface.name.to_string());
            path.set_extension("xml");
            let expected = fs::read_to_string(path).unwrap();

            let generated = serde_xml_rs::to_string(&interface).unwrap();

            assert_eq!(expected.trim(), generated)
        }
    }
}
