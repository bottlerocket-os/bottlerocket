use super::{error, print_json, Result};
use crate::CURRENT_IP;
use argh::FromArgs;
use dns_lookup::lookup_addr;
use snafu::ResultExt;
use std::fs;
use std::net::IpAddr;
use std::str::FromStr;

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "generate-hostname")]
/// Generate hostname from DNS reverse lookup or use current IP
pub(crate) struct GenerateHostnameArgs {}

/// Attempt to resolve assigned IP address, if unsuccessful use the IP as the hostname.
///
/// The result is returned as JSON. (intended for use as a settings generator)
pub(crate) fn run() -> Result<()> {
    let ip_string = fs::read_to_string(CURRENT_IP)
        .context(error::CurrentIpReadFailedSnafu { path: CURRENT_IP })?;
    let ip = IpAddr::from_str(&ip_string).context(error::IpFromStringSnafu { ip: &ip_string })?;
    let hostname = match lookup_addr(&ip) {
        Ok(hostname) => hostname,
        Err(e) => {
            eprintln!("Reverse DNS lookup failed: {}", e);
            ip_string
        }
    };

    // sundog expects JSON-serialized output
    print_json(hostname)
}
