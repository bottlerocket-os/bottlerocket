use super::{error, force_symlink, primary_interface_name, write_primary_interface_sysctl, Result};
use crate::networkd_status::NetworkDInterfaceStatus;
use crate::{CURRENT_IP, NETDOG_RESOLV_CONF, REAL_RESOLV_CONF};
use argh::FromArgs;
use snafu::ResultExt;
use std::fs;
use std::net::IpAddr;

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "write-primary-interface-status")]
/// Writes important network-related files early in boot
pub(crate) struct WriteNetworkStatusArgs {}

pub(crate) fn run() -> Result<()> {
    let primary_interface = primary_interface_name()?;

    let primary_link_status = NetworkDInterfaceStatus::new(primary_interface.clone())
        .context(error::NetworkDInterfaceStatusSnafu {})?;

    // Write out current IP
    let primary_ip = &primary_link_status
        .primary_address()
        .context(error::PrimaryInterfaceAddressSnafu {})?;
    write_current_ip(primary_ip)?;
    write_primary_interface_sysctl(primary_interface)?;

    // Symlink resolv.conf to a common path
    // We don't ever write a resolv.conf when using networkd with resolved; instead we write a
    // drop-in configuration for resolved when the API settings change.
    force_symlink(REAL_RESOLV_CONF, NETDOG_RESOLV_CONF)?;

    Ok(())
}

/// Persist the current IP address to file
fn write_current_ip(ip: &IpAddr) -> Result<()> {
    fs::write(CURRENT_IP, ip.to_string())
        .context(error::CurrentIpWriteFailedSnafu { path: CURRENT_IP })
}
