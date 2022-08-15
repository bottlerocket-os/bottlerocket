use super::{error, InterfaceFamily, InterfaceType, Result};
use crate::dns::DnsSettings;
use crate::lease::{lease_path, LeaseInfo};
use crate::{CURRENT_IP, PRIMARY_INTERFACE};
use argh::FromArgs;
use snafu::{OptionExt, ResultExt};
use std::fs;
use std::net::IpAddr;
use std::path::PathBuf;

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
        (InterfaceType::Dhcp, InterfaceFamily::Ipv4 | InterfaceFamily::Ipv6) => {
            // A lease should exist when using DHCP
            let primary_lease_path =
                lease_path(&primary_interface).context(error::MissingLeaseSnafu {
                    interface: primary_interface,
                })?;
            if args.data_file != primary_lease_path {
                return error::PrimaryLeaseConflictSnafu {
                    wicked_path: args.data_file,
                    generated_path: primary_lease_path,
                }
                .fail();
            }

            // Use DNS API settings if they exist, supplementing any missing settings with settings
            // derived from the primary interface's DHCP lease
            let lease =
                LeaseInfo::from_lease(primary_lease_path).context(error::LeaseParseFailedSnafu)?;
            let dns_settings = DnsSettings::from_config_or_lease(Some(&lease))
                .context(error::GetDnsSettingsSnafu)?;
            dns_settings
                .write_resolv_conf()
                .context(error::ResolvConfWriteFailedSnafu)?;

            write_current_ip(&lease.ip_address.addr())?;
        }
    }
    Ok(())
}

/// Persist the current IP address to file
fn write_current_ip(ip: &IpAddr) -> Result<()> {
    fs::write(CURRENT_IP, ip.to_string())
        .context(error::CurrentIpWriteFailedSnafu { path: CURRENT_IP })
}
