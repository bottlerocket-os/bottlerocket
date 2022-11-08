//! The wicked module contains the data structures and functions needed to create network interface
//! configuration files for wicked.
//!
//! The structures in this module are meant to be created from the user-facing structures in the
//! `net_config` module.  `Default` implementations for WickedInterface exist here as well.
mod dhcp;
mod static_address;

use crate::interface_name::InterfaceName;
pub(crate) use dhcp::{WickedDhcp4, WickedDhcp6};
use serde::Serialize;
use snafu::ResultExt;
pub(crate) use static_address::{WickedRoutes, WickedStaticAddress};
use std::fs;
use std::path::Path;

const WICKED_CONFIG_DIR: &str = "/etc/wicked/ifconfig";
const WICKED_FILE_EXT: &str = "xml";

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename = "interface")]
pub(crate) struct WickedInterface {
    #[serde(rename = "$unflatten=name")]
    pub(crate) name: InterfaceName,
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

impl WickedInterface {
    pub(crate) fn new(name: InterfaceName) -> Self {
        Self {
            name,
            control: WickedControl::default(),
            ipv4_dhcp: None,
            ipv6_dhcp: None,
            ipv4_static: None,
            ipv6_static: None,
        }
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

pub(crate) use error::Error;
type Result<T> = std::result::Result<T, error::Error>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net_config::{self, Interfaces, NetConfigV1};
    use handlebars::Handlebars;
    use serde::Serialize;
    use std::path::PathBuf;
    use std::str::FromStr;

    static NET_CONFIG_VERSIONS: &[u8] = &[1, 2];

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

            let wicked_interfaces = net_config.as_wicked_interfaces();
            for interface in wicked_interfaces {
                let generated = quick_xml::se::to_string(&interface).unwrap();

                let mut path = wicked_config().join(interface.name.to_string());
                path.set_extension("xml");
                let expected = fs::read_to_string(path).unwrap();

                assert_eq!(expected.trim(), generated)
            }
        }
    }

    // Test the end to end trip: "net config -> wicked -> serialized XML"
    #[test]
    #[allow(clippy::to_string_in_format_args)]
    fn net_config_to_interface_config() {
        let net_config_path = wicked_config().join("net_config.toml");

        for version in NET_CONFIG_VERSIONS {
            let temp_config = tempfile::NamedTempFile::new().unwrap();

            render_config_template(&net_config_path, &temp_config, version);
            let net_config = net_config::from_path(&temp_config).unwrap().unwrap();
            let wicked_interfaces = net_config.as_wicked_interfaces();
            for interface in wicked_interfaces {
                let mut path = wicked_config().join(interface.name.to_string());
                path.set_extension("xml");
                let expected = fs::read_to_string(path).unwrap();
                let generated = quick_xml::se::to_string(&interface).unwrap();
                dbg!(&generated);

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
