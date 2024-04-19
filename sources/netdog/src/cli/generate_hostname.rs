use super::{error, print_json, Result};
use crate::CURRENT_IP;
use argh::FromArgs;
use dogtag::find_hostname;
use snafu::ResultExt;
use std::fs;
use std::net::IpAddr;
use std::str::FromStr;

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "generate-hostname")]
/// Generate hostname from installed hostname resolvers
pub(crate) struct GenerateHostnameArgs {}

/// Attempt to resolve assigned IP address, if unsuccessful use the IP as the hostname.
///
/// The result is returned as JSON. (intended for use as a settings generator)
pub(crate) async fn run() -> Result<()> {
    let ip_string = fs::read_to_string(CURRENT_IP)
        .context(error::CurrentIpReadFailedSnafu { path: CURRENT_IP })?;
    let ip = IpAddr::from_str(&ip_string).context(error::IpFromStringSnafu { ip: &ip_string })?;

    // If no hostname has been determined we return the IP address of the host, replacing invalid ipv6 chars.
    let hostname = find_hostname(ip)
        .await
        .unwrap_or(ip_string.replace(':', "-"));

    // sundog expects JSON-serialized output
    print_json(hostname)
}
