use super::{error, Result};
use crate::KERNEL_HOSTNAME;
use argh::FromArgs;
use snafu::ResultExt;
use std::fs;

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "set-hostname")]
/// Sets the hostname
pub(crate) struct SetHostnameArgs {
    #[argh(positional)]
    /// hostname for the system
    hostname: String,
}

/// Sets the hostname for the system
pub(crate) fn run(args: SetHostnameArgs) -> Result<()> {
    fs::write(KERNEL_HOSTNAME, args.hostname).context(error::HostnameWriteFailedSnafu {
        path: KERNEL_HOSTNAME,
    })?;
    Ok(())
}
