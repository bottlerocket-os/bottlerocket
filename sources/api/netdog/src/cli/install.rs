use super::{
    error, force_symlink, primary_interface_name, write_primary_interface_sysctl, InterfaceFamily,
    InterfaceType, Result,
};
use crate::dns::DnsSettings;
use crate::lease::{dhcp_lease_path, static_lease_path, LeaseInfo};
use crate::{CURRENT_IP, NETDOG_RESOLV_CONF, REAL_RESOLV_CONF};
use argh::FromArgs;
use snafu::{ensure, OptionExt, ResultExt};
use std::fs;
use std::net::IpAddr;
use std::path::{Path, PathBuf};

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
    let primary_interface = primary_interface_name()?;

    if install_interface != primary_interface {
        return Ok(());
    }

    match (&args.interface_type, &args.interface_family) {
        (
            interface_type @ (InterfaceType::Dhcp | InterfaceType::Static),
            InterfaceFamily::Ipv4 | InterfaceFamily::Ipv6,
        ) => {
            let lease = fetch_lease(&primary_interface, interface_type, args.data_file)?;
            write_current_ip(&lease.ip_address.addr())?;
            write_primary_interface_sysctl(primary_interface)?;

            write_resolv_conf(&lease)?;
            force_symlink(REAL_RESOLV_CONF, NETDOG_RESOLV_CONF)?
        }
    }
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
