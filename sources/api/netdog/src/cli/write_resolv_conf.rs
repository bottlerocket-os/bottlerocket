use super::{error, primary_interface_name, Result};
use crate::dns::DnsSettings;
#[cfg(net_backend = "wicked")]
use crate::lease::{dhcp_lease_path, LeaseInfo};
#[cfg(net_backend = "systemd-networkd")]
use crate::networkd_status::NetworkDInterfaceStatus;
use argh::FromArgs;
use snafu::ResultExt;

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "write-resolv-conf")]
/// Writes /etc/resolv.conf, using DNS API settings if they exist
pub(crate) struct WriteResolvConfArgs {}

#[cfg(net_backend = "systemd-networkd")]
fn get_dns_settings(primary_interface: String) -> Result<DnsSettings> {
    let primary_link_status = NetworkDInterfaceStatus::new(primary_interface)
        .context(error::NetworkDInterfaceStatusSnafu)?;
    let dns_settings = DnsSettings::from_config_or_status(&primary_link_status)
        .context(error::GetDnsSettingsSnafu)?;
    Ok(dns_settings)
}

#[cfg(net_backend = "wicked")]
fn get_dns_settings(primary_interface: String) -> Result<DnsSettings> {
    let primary_lease_path = dhcp_lease_path(primary_interface);
    let dns_settings = if let Some(primary_lease_path) = primary_lease_path {
        let lease =
            LeaseInfo::from_lease(primary_lease_path).context(error::LeaseParseFailedSnafu)?;
        DnsSettings::from_config_or_lease(Some(&lease)).context(error::GetDnsSettingsSnafu)?
    } else {
        DnsSettings::from_config_or_lease(None).context(error::GetDnsSettingsSnafu)?
    };
    Ok(dns_settings)
}

pub(crate) fn run() -> Result<()> {
    let primary_interface = primary_interface_name()?;
    // Use DNS API settings if they exist, supplementing any missing settings with settings derived
    // from the primary interface's DHCP lease if it exists.  Static leases don't contain any DNS
    // data, so don't bother looking there.
    let dns_settings = get_dns_settings(primary_interface)?;

    dns_settings
        .write_resolv_conf()
        .context(error::ResolvConfWriteFailedSnafu)?;
    Ok(())
}
