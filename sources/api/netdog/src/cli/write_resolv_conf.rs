use super::{error, Result};
use crate::dns::DnsSettings;
use crate::lease::{lease_path, LeaseInfo};
use crate::PRIMARY_INTERFACE;
use argh::FromArgs;
use snafu::ResultExt;
use std::fs;

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "write-resolv-conf")]
/// Writes /etc/resolv.conf, using DNS API settings if they exist
pub(crate) struct WriteResolvConfArgs {}

pub(crate) fn run() -> Result<()> {
    // Use DNS API settings if they exist, supplementing any missing settings with settings
    // derived from the primary interface's DHCP lease if it exists
    let primary_interface = fs::read_to_string(PRIMARY_INTERFACE)
        .context(error::PrimaryInterfaceReadSnafu {
            path: PRIMARY_INTERFACE,
        })?
        .trim()
        .to_lowercase();

    let primary_lease_path = lease_path(&primary_interface);
    let dns_settings = if let Some(primary_lease_path) = primary_lease_path {
        let lease =
            LeaseInfo::from_lease(&primary_lease_path).context(error::LeaseParseFailedSnafu)?;
        DnsSettings::from_config_or_lease(Some(&lease)).context(error::GetDnsSettingsSnafu)?
    } else {
        DnsSettings::from_config_or_lease(None).context(error::GetDnsSettingsSnafu)?
    };

    dns_settings
        .write_resolv_conf()
        .context(error::ResolvConfWriteFailedSnafu)?;
    Ok(())
}
