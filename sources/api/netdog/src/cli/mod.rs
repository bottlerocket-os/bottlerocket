pub(crate) mod generate_hostname;
pub(crate) mod generate_net_config;
pub(crate) mod node_ip;
pub(crate) mod set_hostname;
pub(crate) mod write_resolv_conf;

#[cfg(net_backend = "wicked")]
pub(crate) mod install;
#[cfg(net_backend = "wicked")]
pub(crate) mod remove;

#[cfg(net_backend = "systemd-networkd")]
pub(crate) mod write_network_status;

use crate::net_config::{self, Interfaces};
use crate::{
    DEFAULT_NET_CONFIG_FILE, KERNEL_CMDLINE, OVERRIDE_NET_CONFIG_FILE, PRIMARY_INTERFACE,
    PRIMARY_MAC_ADDRESS, PRIMARY_SYSCTL_CONF, SYSCTL_MARKER_FILE, SYSTEMD_SYSCTL, SYS_CLASS_NET,
};
pub(crate) use generate_hostname::GenerateHostnameArgs;
pub(crate) use generate_net_config::GenerateNetConfigArgs;
pub(crate) use node_ip::NodeIpArgs;
use serde::{Deserialize, Serialize};
pub(crate) use set_hostname::SetHostnameArgs;
use snafu::{ensure, OptionExt, ResultExt};
use std::fmt::Write;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fs, io};
pub(crate) use write_resolv_conf::WriteResolvConfArgs;

#[cfg(net_backend = "wicked")]
pub(crate) use install::InstallArgs;
#[cfg(net_backend = "wicked")]
pub(crate) use remove::RemoveArgs;

#[cfg(net_backend = "systemd-networkd")]
pub(crate) use write_network_status::WriteNetworkStatusArgs;

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

/// Set sysctl settings for provided interface
// This manages the logic around ensuring required sysctls is up to date for the primary interface.
fn write_primary_interface_sysctl(interface: String) -> Result<()> {
    // If we haven't already, set and apply default sysctls for the primary network
    // interface
    if !Path::exists(Path::new(PRIMARY_SYSCTL_CONF)) {
        write_interface_sysctl(interface, PRIMARY_SYSCTL_CONF)?;
    };

    // Execute `systemd-sysctl` with our configuration file to set the sysctls
    if !Path::exists(Path::new(SYSCTL_MARKER_FILE)) {
        let systemd_sysctl_result = Command::new(SYSTEMD_SYSCTL)
            .arg(PRIMARY_SYSCTL_CONF)
            .output()
            .context(error::SystemdSysctlExecutionSnafu)?;
        ensure!(
            systemd_sysctl_result.status.success(),
            error::FailedSystemdSysctlSnafu {
                stderr: String::from_utf8_lossy(&systemd_sysctl_result.stderr)
            }
        );

        fs::write(SYSCTL_MARKER_FILE, "").unwrap_or_else(|e| {
            eprintln!(
                "Failed to create marker file {}, netdog may attempt to set sysctls again: {}",
                SYSCTL_MARKER_FILE, e
            )
        });
    };
    Ok(())
}

/// Write the default sysctls for a given interface to a given path
fn write_interface_sysctl<S, P>(interface: S, path: P) -> Result<()>
where
    S: AsRef<str>,
    P: AsRef<Path>,
{
    let interface = interface.as_ref();
    let path = path.as_ref();
    // Note: The dash (-) preceding the "net..." variable assignment below is important; it
    // ensures failure to set the variable for any reason will be logged, but not cause the sysctl
    // service to fail
    // Enable loose mode for reverse path filter
    let ipv4_rp_filter = format!("-net.ipv4.conf.{}.rp_filter = 2", interface);

    let mut output = String::new();
    writeln!(output, "{}", ipv4_rp_filter).context(error::SysctlConfBuildSnafu)?;

    #[cfg(net_backend = "systemd-networkd")]
    // systemd-networkd implements its own RA client, and expects the kernel implementation to be
    // unused. However, various solutions that run in EC2 might "helpfully" turn it on since it's
    // required for most non-systemd-networkd systems. Guard against this by explicitly disabling
    // all of the sysctls that default to enabled when "accept_ra" is enabled.
    for ipv6_sysctl in [
        format!("-net.ipv6.conf.{}.accept_ra = 0", interface),
        format!("-net.ipv6.conf.{}.accept_ra_defrtr = 0", interface),
        format!("-net.ipv6.conf.{}.accept_ra_pinfo = 0", interface),
        format!("-net.ipv6.conf.{}.accept_ra_rtr_pref = 0", interface),
        format!("-net.ipv6.conf.{}.accept_ra_mtu = 0", interface),
    ] {
        writeln!(output, "{}", ipv6_sysctl).context(error::SysctlConfBuildSnafu)?;
    }

    fs::write(path, output).context(error::SysctlConfWriteSnafu { path })?;
    Ok(())
}

// Gather net config from possible sources, returning both the config and the source
fn fetch_net_config() -> Result<(Option<Box<dyn Interfaces>>, PathBuf)> {
    let override_path = PathBuf::from(OVERRIDE_NET_CONFIG_FILE);
    let default_path = PathBuf::from(DEFAULT_NET_CONFIG_FILE);
    let cmdline = PathBuf::from(KERNEL_CMDLINE);

    for path in [override_path, default_path] {
        if Path::exists(&path) {
            return Ok((
                net_config::from_path(&path).context(error::NetConfigParseSnafu { path: &path })?,
                path,
            ));
        }
    }

    Ok((
        net_config::from_command_line(&cmdline)
            .context(error::NetConfigParseSnafu { path: &cmdline })?,
        cmdline,
    ))
}

fn force_symlink<P1, P2>(target: P1, link: P2) -> Result<()>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    let target = target.as_ref();
    let link = link.as_ref();

    // Remove link if it already exists
    if let Err(e) = fs::remove_file(link) {
        if e.kind() != io::ErrorKind::NotFound {
            Err(e).context(error::SymlinkSnafu { target, link })?;
        }
    }
    // Link to requested target
    symlink(target, link).context(error::SymlinkSnafu { target, link })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_sysctls() {
        let interface = "eno1";
        let fake_file = tempfile::NamedTempFile::new().unwrap();
        let expected = "-net.ipv4.conf.eno1.rp_filter = 2\n";
        write_interface_sysctl(interface, &fake_file).unwrap();
        assert_eq!(std::fs::read_to_string(&fake_file).unwrap(), expected);
    }
}

/// Potential errors during netdog execution
mod error {
    use crate::{dns, interface_id, net_config};
    use snafu::Snafu;
    use std::ffi::OsString;
    use std::io;
    use std::path::PathBuf;

    #[cfg(net_backend = "wicked")]
    use crate::{lease, wicked};

    #[cfg(net_backend = "systemd-networkd")]
    use crate::{networkd, networkd_status};

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(crate)))]
    #[allow(clippy::enum_variant_names)]
    pub(crate) enum Error {
        #[cfg(net_backend = "systemd-networkd")]
        #[snafu(display("Unable to create directory '{}': {}", path.display(),source))]
        CreateDir { path: PathBuf, source: io::Error },

        #[snafu(display("Failed to write current IP to '{}': {}", path.display(), source))]
        CurrentIpWriteFailed { path: PathBuf, source: io::Error },

        #[snafu(display("Failed to read current IP data in '{}': {}", path.display(), source))]
        CurrentIpReadFailed { path: PathBuf, source: io::Error },

        #[snafu(display("Unable to gather DNS settings: {}", source))]
        GetDnsSettings { source: dns::Error },

        #[snafu(display("Failed to read/parse DNS settings from DHCP lease: {}", source))]
        DnsFromLease { source: dns::Error },

        #[cfg(net_backend = "systemd-networkd")]
        #[snafu(display("Unable to create interface drop-in directory '{}': {}", path.display(), source))]
        DropInDirCreate { path: PathBuf, source: io::Error },

        #[cfg(net_backend = "systemd-networkd")]
        #[snafu(display("Unable to write interface drop-in '{}': {}", path.display(), source))]
        DropInFileWrite { path: PathBuf, source: io::Error },

        #[snafu(display("'systemd-sysctl' failed: {}", stderr))]
        FailedSystemdSysctl { stderr: String },

        #[cfg(net_backend = "systemd-networkd")]
        #[snafu(display("'systemctl' failed: {}", stderr))]
        FailedSystemctl { stderr: String },

        #[snafu(display("Failed to remove '{}': {}", path.display(), source))]
        FileRemove { path: PathBuf, source: io::Error },

        #[snafu(display("Failed to discern primary interface"))]
        GetPrimaryInterface,

        #[snafu(display("Failed to write hostname to '{}': {}", path.display(), source))]
        HostnameWriteFailed { path: PathBuf, source: io::Error },

        #[cfg(net_backend = "wicked")]
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

        #[cfg(net_backend = "wicked")]
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

        #[cfg(net_backend = "systemd-networkd")]
        #[snafu(display("Unable to create systemd-networkd config: {}", source))]
        NetworkDConfigCreate { source: net_config::Error },

        #[cfg(net_backend = "systemd-networkd")]
        #[snafu(display("Failed to write network interface configuration: {}", source))]
        NetworkDConfigWrite { source: networkd::Error },

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

        #[cfg(net_backend = "systemd-networkd")]
        #[snafu(display("Failed to run 'systemctl': {}", source))]
        SystemctlExecution { source: io::Error },

        #[cfg(net_backend = "systemd-networkd")]
        #[snafu(display("Failed to retrieve networkctl status: {}", source))]
        NetworkDInterfaceStatus {
            source: networkd_status::NetworkDStatusError,
        },

        #[cfg(net_backend = "systemd-networkd")]
        #[snafu(display("Unable to determine primary interface IP Address: {}", source))]
        PrimaryInterfaceAddress {
            source: networkd_status::NetworkDStatusError,
        },

        #[snafu(display("Unable to symlink '{}' to '{}': {}", link.display(), target.display(), source))]
        Symlink {
            target: PathBuf,
            link: PathBuf,
            source: io::Error,
        },
    }
}

pub(crate) type Result<T> = std::result::Result<T, error::Error>;
