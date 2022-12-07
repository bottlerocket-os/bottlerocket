use super::{error, InterfaceFamily, InterfaceType, Result};
use crate::dns::DnsSettings;
use crate::lease::{dhcp_lease_path, static_lease_path, LeaseInfo};
use crate::{
    CURRENT_IP, PRIMARY_INTERFACE, PRIMARY_SYSCTL_CONF, SYSCTL_MARKER_FILE, SYSTEMD_SYSCTL,
};
use argh::FromArgs;
use snafu::{ensure, OptionExt, ResultExt};
use std::fmt::Write;
use std::fs;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "install")]
/// Write resolv.conf and current IP to disk
pub(crate) struct InstallArgs {
    #[argh(option, short = 'i')]
    /// name of the network interface
    interface_name: String,

    #[argh(option, short = 't')]
    /// network interface type
    interface_type: InterfaceType,

    #[argh(option, short = 'f')]
    /// network interface family (ipv4/6)
    interface_family: InterfaceFamily,

    #[argh(positional)]
    /// lease info data file
    data_file: PathBuf,

    #[argh(positional)]
    // wicked adds `info` to the call to this program.  We don't do anything with it but must
    // be able to parse the option to avoid failing
    /// ignored
    info: Option<String>,
}

pub(crate) fn run(args: InstallArgs) -> Result<()> {
    // Wicked doesn't mangle interface names, but let's be defensive.
    let install_interface = args.interface_name.trim().to_lowercase();
    let primary_interface = fs::read_to_string(PRIMARY_INTERFACE)
        .context(error::PrimaryInterfaceReadSnafu {
            path: PRIMARY_INTERFACE,
        })?
        .trim()
        .to_lowercase();

    if install_interface != primary_interface {
        return Ok(());
    }

    match (&args.interface_type, &args.interface_family) {
        (
            interface_type @ (InterfaceType::Dhcp | InterfaceType::Static),
            InterfaceFamily::Ipv4 | InterfaceFamily::Ipv6,
        ) => {
            let lease = fetch_lease(&primary_interface, interface_type, args.data_file)?;
            write_resolv_conf(&lease)?;
            write_current_ip(&lease.ip_address.addr())?;

            // If we haven't already, set and apply default sysctls for the primary network
            // interface
            if !Path::exists(Path::new(PRIMARY_SYSCTL_CONF)) {
                write_interface_sysctl(primary_interface, PRIMARY_SYSCTL_CONF)?;
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
            }
        }
    }
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
    // TODO if we accumulate more of these we should have a better way to create than format!()
    // Note: The dash (-) preceding the "net..." variable assignment below is important; it
    // ensures failure to set the variable for any reason will be logged, but not cause the sysctl
    // service to fail
    // Accept router advertisement (RA) packets even if IPv6 forwarding is enabled on interface
    let ipv6_accept_ra = format!("-net.ipv6.conf.{}.accept_ra = 2", interface);
    // Enable loose mode for reverse path filter
    let ipv4_rp_filter = format!("-net.ipv4.conf.{}.rp_filter = 2", interface);

    let mut output = String::new();
    writeln!(output, "{}", ipv6_accept_ra).context(error::SysctlConfBuildSnafu)?;
    writeln!(output, "{}", ipv4_rp_filter).context(error::SysctlConfBuildSnafu)?;

    fs::write(path, output).context(error::SysctlConfWriteSnafu { path })?;
    Ok(())
}

/// Given an interface, its type, and wicked's known location of the lease, compare our known lease
/// location, parse and return a LeaseInfo.
fn fetch_lease<S, P>(
    interface: S,
    interface_type: &InterfaceType,
    data_file: P,
) -> Result<LeaseInfo>
where
    S: AsRef<str>,
    P: AsRef<Path>,
{
    let interface = interface.as_ref();
    let data_file = data_file.as_ref();
    let lease_path = match interface_type {
        InterfaceType::Dhcp => dhcp_lease_path(interface),
        InterfaceType::Static => static_lease_path(interface),
    }
    .context(error::MissingLeaseSnafu { interface })?;

    ensure!(
        data_file == lease_path,
        error::PrimaryLeaseConflictSnafu {
            wicked_path: data_file,
            generated_path: lease_path,
        }
    );

    LeaseInfo::from_lease(&lease_path).context(error::LeaseParseFailedSnafu)
}

/// Given a lease, fetch DNS settings from the lease and/or config and write the resolv.conf
fn write_resolv_conf(lease: &LeaseInfo) -> Result<()> {
    let dns_settings =
        DnsSettings::from_config_or_lease(Some(lease)).context(error::GetDnsSettingsSnafu)?;
    dns_settings
        .write_resolv_conf()
        .context(error::ResolvConfWriteFailedSnafu)
}

/// Persist the current IP address to file
fn write_current_ip(ip: &IpAddr) -> Result<()> {
    fs::write(CURRENT_IP, ip.to_string())
        .context(error::CurrentIpWriteFailedSnafu { path: CURRENT_IP })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_sysctls() {
        let interface = "eno1";
        let fake_file = tempfile::NamedTempFile::new().unwrap();
        let expected = "-net.ipv6.conf.eno1.accept_ra = 2\n-net.ipv4.conf.eno1.rp_filter = 2\n";
        write_interface_sysctl(interface, &fake_file).unwrap();
        assert_eq!(std::fs::read_to_string(&fake_file).unwrap(), expected);
    }
}
