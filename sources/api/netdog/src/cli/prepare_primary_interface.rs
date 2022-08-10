use super::{error, Result};
use crate::{PRIMARY_INTERFACE, PRIMARY_SYSCTL_CONF, SYSTEMD_SYSCTL};
use argh::FromArgs;
use snafu::{ensure, ResultExt};
use std::fmt::Write;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "prepare-primary-interface")]
/// Sets the default sysctls for the primary interface
pub(crate) struct PreparePrimaryInterfaceArgs {}

/// Set and apply default sysctls for the primary network interface
pub(crate) fn run() -> Result<()> {
    let primary_interface =
        fs::read_to_string(PRIMARY_INTERFACE).context(error::PrimaryInterfaceReadSnafu {
            path: PRIMARY_INTERFACE,
        })?;
    write_interface_sysctl(primary_interface, PRIMARY_SYSCTL_CONF)?;

    // Execute `systemd-sysctl` with our configuration file to set the sysctls
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
