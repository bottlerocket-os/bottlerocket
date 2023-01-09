use super::{error, primary_interface_name, Result};
use crate::dns::DnsSettings;
use crate::lease::{dhcp_lease_path, LeaseInfo};
use argh::FromArgs;
use snafu::ResultExt;

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "write-resolv-conf")]
/// Writes /etc/resolv.conf, using DNS API settings if they exist
pub(crate) struct WriteResolvConfArgs {}

pub(crate) fn run() -> Result<()> {
    // Use DNS API settings if they exist, supplementing any missing settings with settings derived
    // from the primary interface's DHCP lease if it exists.  Static leases don't contain any DNS
    // data, so don't bother looking there.
    let primary_interface = primary_interface_name()?;

    let primary_lease_path = dhcp_lease_path(primary_interface);
    let dns_settings = if let Some(primary_lease_path) = primary_lease_path {
        let lease =
            LeaseInfo::from_lease(primary_lease_path).context(error::LeaseParseFailedSnafu)?;
        DnsSettings::from_config_or_lease(Some(&lease)).context(error::GetDnsSettingsSnafu)?
    } else {
        DnsSettings::from_config_or_lease(None).context(error::GetDnsSettingsSnafu)?
    };

    dns_settings
        .write_resolv_conf()
        .context(error::ResolvConfWriteFailedSnafu)?;
    Ok(())
}
