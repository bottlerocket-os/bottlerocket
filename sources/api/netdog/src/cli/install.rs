use super::{error, InterfaceFamily, InterfaceType, Result};
use crate::lease::LeaseInfo;
use crate::{CURRENT_IP, PRIMARY_INTERFACE, RESOLV_CONF};
use argh::FromArgs;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use snafu::ResultExt;
use std::fmt::Write;
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
        (InterfaceType::Dhcp, InterfaceFamily::Ipv4) => {
            let info =
                LeaseInfo::from_lease(&args.data_file).context(error::LeaseParseFailedSnafu {
                    path: &args.data_file,
                })?;
            // Randomize name server order, for libc implementations like musl that send
            // queries to the first N servers.
            let mut dns_servers: Vec<_> = info.dns_servers.iter().collect();
            dns_servers.shuffle(&mut thread_rng());
            write_resolv_conf(&dns_servers, &info.dns_search)?;
            write_current_ip(&info.ip_address.addr())?;
        }
        _ => eprintln!("Unhandled 'install' command: {:?}", &args),
    }
    Ok(())
}

/// Write resolver configuration for libc.
fn write_resolv_conf(dns_servers: &[&IpAddr], dns_search: &Option<Vec<String>>) -> Result<()> {
    let mut output = String::new();

    if let Some(s) = dns_search {
        writeln!(output, "search {}", s.join(" ")).context(error::ResolvConfBuildFailedSnafu)?;
    }

    for n in dns_servers {
        writeln!(output, "nameserver {}", n).context(error::ResolvConfBuildFailedSnafu)?;
    }

    fs::write(RESOLV_CONF, output)
        .context(error::ResolvConfWriteFailedSnafu { path: RESOLV_CONF })?;
    Ok(())
}

/// Persist the current IP address to file
fn write_current_ip(ip: &IpAddr) -> Result<()> {
    fs::write(CURRENT_IP, ip.to_string())
        .context(error::CurrentIpWriteFailedSnafu { path: CURRENT_IP })
}
