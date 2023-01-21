pub(crate) mod check_net_config;
pub(crate) mod generate_hostname;
pub(crate) mod generate_net_config;
pub(crate) mod install;
pub(crate) mod node_ip;
pub(crate) mod remove;
pub(crate) mod set_hostname;
pub(crate) mod write_resolv_conf;

use crate::net_config::Interfaces;
use crate::{
    net_config, DEFAULT_NET_CONFIG_FILE, KERNEL_CMDLINE, OVERRIDE_NET_CONFIG_FILE,
    PRIMARY_INTERFACE, PRIMARY_MAC_ADDRESS, SYS_CLASS_NET,
};
pub(crate) use check_net_config::CheckNetConfigArgs;
pub(crate) use generate_hostname::GenerateHostnameArgs;
pub(crate) use generate_net_config::GenerateNetConfigArgs;
pub(crate) use install::InstallArgs;
pub(crate) use node_ip::NodeIpArgs;
pub(crate) use remove::RemoveArgs;
use serde::{Deserialize, Serialize};
pub(crate) use set_hostname::SetHostnameArgs;
use snafu::{OptionExt, ResultExt};
use std::fs;
use std::path::Path;
pub(crate) use write_resolv_conf::WriteResolvConfArgs;

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum InterfaceType {
    Dhcp,
    Static,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum InterfaceFamily {
    Ipv4,
    Ipv6,
}

// Implement `from_str()` so argh can attempt to deserialize args into their proper types
derive_fromstr_from_deserialize!(InterfaceType);
derive_fromstr_from_deserialize!(InterfaceFamily);

/// Helper function that serializes the input to JSON and prints it
fn print_json<S>(val: S) -> Result<()>
where
    S: AsRef<str> + Serialize,
{
    let val = val.as_ref();
    let output = serde_json::to_string(val).context(error::JsonSerializeSnafu { output: val })?;
    println!("{}", output);
    Ok(())
}

/// Search for the network configuration file and return the configuration if it parses successfully,
/// otherwise return an error
fn check_net_config() -> Result<Option<Box<dyn Interfaces>>> {
    let maybe_net_config = if Path::exists(Path::new(OVERRIDE_NET_CONFIG_FILE)) {
        net_config::from_path(OVERRIDE_NET_CONFIG_FILE).context(error::NetConfigParseSnafu {
            path: OVERRIDE_NET_CONFIG_FILE,
        })?
    } else if Path::exists(Path::new(DEFAULT_NET_CONFIG_FILE)) {
        net_config::from_path(DEFAULT_NET_CONFIG_FILE).context(error::NetConfigParseSnafu {
            path: DEFAULT_NET_CONFIG_FILE,
        })?
    } else {
        net_config::from_command_line(KERNEL_CMDLINE).context(error::NetConfigParseSnafu {
            path: KERNEL_CMDLINE,
        })?
    };
    Ok(maybe_net_config)
}

/// Return the primary interface name
// A primary_interface or primary_mac_address file should exist.  If the primary_interface file
// exists use it, otherwise read the primary_mac_address file and crawl sysfs to find which
// interface has the corresponding MAC, if any.
fn primary_interface_name() -> Result<String> {
    let clean = |s: String| s.trim().to_lowercase();

    let maybe_name = fs::read_to_string(PRIMARY_INTERFACE).ok();
    if let Some(name) = maybe_name {
        return Ok(clean(name));
    }

    let primary_mac = clean(fs::read_to_string(PRIMARY_MAC_ADDRESS).context(
        error::PathReadSnafu {
            path: PRIMARY_MAC_ADDRESS,
        },
    )?);

    // There should be directories for each of the interfaces, i.e /sys/class/net/eth0
    let sysfs_net = fs::read_dir(SYS_CLASS_NET)
        .context(error::PathReadSnafu {
            path: SYS_CLASS_NET,
        })?
        .flatten()
        .filter(|p| p.path().is_dir());

    for interface in sysfs_net {
        let mac_address_path = interface.path().join("address");

        if let Ok(address) = fs::read_to_string(mac_address_path) {
            if clean(address) == primary_mac {
                return interface.file_name().into_string().ok().context(
                    error::InterfaceNameUtf8Snafu {
                        name: interface.file_name(),
                    },
                );
            }
        };
    }

    error::NonExistentMacSnafu { mac: primary_mac }.fail()
}

/// Potential errors during netdog execution
mod error {
    use crate::{dns, interface_id, lease, net_config, wicked};
    use snafu::Snafu;
    use std::ffi::OsString;
    use std::io;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(crate)))]
    #[allow(clippy::enum_variant_names)]
    pub(crate) enum Error {
        #[snafu(display("Failed to write current IP to '{}': {}", path.display(), source))]
        CurrentIpWriteFailed { path: PathBuf, source: io::Error },

        #[snafu(display("Failed to read current IP data in '{}': {}", path.display(), source))]
        CurrentIpReadFailed { path: PathBuf, source: io::Error },

        #[snafu(display("Unable to gather DNS settings: {}", source))]
        GetDnsSettings { source: dns::Error },

        #[snafu(display("Failed to read/parse DNS settings from DHCP lease: {}", source))]
        DnsFromLease { source: dns::Error },

        #[snafu(display("'systemd-sysctl' failed: {}", stderr))]
        FailedSystemdSysctl { stderr: String },

        #[snafu(display("Failed to remove '{}': {}", path.display(), source))]
        FileRemove { path: PathBuf, source: io::Error },

        #[snafu(display("Failed to discern primary interface"))]
        GetPrimaryInterface,

        #[snafu(display("Failed to write hostname to '{}': {}", path.display(), source))]
        HostnameWriteFailed { path: PathBuf, source: io::Error },

        #[snafu(display("Failed to write network interface configuration: {}", source))]
        InterfaceConfigWrite { source: wicked::Error },

        #[snafu(display("Unable to determine interface name: {}", source))]
        InterfaceName { source: interface_id::Error },

        #[snafu(display("Non-UTF8 interface name '{:?}'", name.to_string_lossy()))]
        InterfaceNameUtf8 { name: OsString },

        #[snafu(display("Invalid IP address '{}': {}", ip, source))]
        IpFromString {
            ip: String,
            source: std::net::AddrParseError,
        },

        #[snafu(display("Error serializing to JSON: '{}': {}", output, source))]
        JsonSerialize {
            output: String,
            source: serde_json::error::Error,
        },

        #[snafu(display("Failed to read/parse lease data: {}", source))]
        LeaseParseFailed { source: lease::Error },

        #[snafu(display("No DHCP lease found for interface '{}'", interface))]
        MissingLease { interface: String },

        #[snafu(display("Unable to read/parse network config from '{}': {}", path.display(), source))]
        NetConfigParse {
            path: PathBuf,
            source: net_config::Error,
        },

        #[snafu(display("Unable to find an interface with MAC address '{}'", mac))]
        NonExistentMac { mac: String },

        #[snafu(display("Unable to read '{}': {}", path.display(), source))]
        PathRead {
            path: PathBuf,
            source: std::io::Error,
        },

        #[snafu(display("Failed to write primary interface to '{}': {}", path.display(), source))]
        PrimaryInterfaceWrite { path: PathBuf, source: io::Error },

        #[snafu(display("Failed to read primary interface from '{}': {}", path.display(), source))]
        PrimaryInterfaceRead { path: PathBuf, source: io::Error },

        #[snafu(display("Conflicting primary lease location; from wicked: '{}', generated by netdog: '{}'", wicked_path.display(), generated_path.display()))]
        PrimaryLeaseConflict {
            wicked_path: PathBuf,
            generated_path: PathBuf,
        },

        #[snafu(display("Failed to write resolver configuration: {}", source))]
        ResolvConfWriteFailed { source: dns::Error },

        #[snafu(display("Failed to build sysctl config: {}", source))]
        SysctlConfBuild { source: std::fmt::Error },

        #[snafu(display("Failed to write sysctl config to '{}': {}", path.display(), source))]
        SysctlConfWrite { path: PathBuf, source: io::Error },

        #[snafu(display("Failed to run 'systemd-sysctl': {}", source))]
        SystemdSysctlExecution { source: io::Error },
    }
}

pub(crate) type Result<T> = std::result::Result<T, error::Error>;
