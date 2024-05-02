use super::{error, print_json, Result};
use crate::CURRENT_IP;
use argh::FromArgs;
use snafu::ResultExt;
use std::fs;
use std::net::IpAddr;
use std::str::FromStr;

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "node-ip")]
/// Return the current IP address
pub(crate) struct NodeIpArgs {}

/// Return the current IP address as JSON (intended for use as a settings generator)
pub(crate) fn run() -> Result<()> {
    let ip_string = fs::read_to_string(CURRENT_IP)
        .context(error::CurrentIpReadFailedSnafu { path: CURRENT_IP })?;
    // Validate that we read a proper IP address
    let _ = IpAddr::from_str(&ip_string).context(error::IpFromStringSnafu { ip: &ip_string })?;

    // sundog expects JSON-serialized output
    print_json(ip_string)
}
