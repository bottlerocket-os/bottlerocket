use super::{error, primary_interface_name, write_primary_interface_sysctl, Result};
use crate::dns::DnsSettings;
use crate::networkd_status::NetworkDInterfaceStatus;
use crate::CURRENT_IP;
use argh::FromArgs;
use snafu::ResultExt;
use std::fs;
use std::net::IpAddr;

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "write-primary-interface-status")]
/// Updates the various files needed when responding to events
pub(crate) struct WritePrimaryInterfaceStatusArgs {}

/// Updates the various files needed when responding to events
pub(crate) fn run() -> Result<()> {
    let primary_interface = primary_interface_name()?;

    let primary_link_status = NetworkDInterfaceStatus::new(primary_interface.clone())
        .context(error::NetworkDInterfaceStatusSnafu {})?;

    // Write out current IP
    let primary_ip = &primary_link_status
        .primary_address()
        .context(error::PrimaryInterfaceAddressSnafu {})?;
    write_current_ip(primary_ip)?;

    // Write out resolv.conf
    write_resolv_conf(&primary_link_status)?;

    write_primary_interface_sysctl(primary_interface)?;

    Ok(())
}

/// Persist the current IP address to file
fn write_current_ip(ip: &IpAddr) -> Result<()> {
    fs::write(CURRENT_IP, ip.to_string())
        .context(error::CurrentIpWriteFailedSnafu { path: CURRENT_IP })
}

/// Given network status find DNS settings from the status and/or config and write the resolv.conf
fn write_resolv_conf(status: &NetworkDInterfaceStatus) -> Result<()> {
    let dns_settings =
        DnsSettings::from_config_or_status(status).context(error::GetDnsSettingsSnafu)?;

    dns_settings
        .write_resolv_conf()
        .context(error::ResolvConfWriteFailedSnafu)
}
