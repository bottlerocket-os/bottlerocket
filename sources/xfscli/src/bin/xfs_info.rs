use snafu::OptionExt;
use std::process::ExitCode;
use std::str;

use argh::FromArgs;

use xfscli::error::{self, Result};
use xfscli::{get_mount_point, query_xfs_db, query_xfs_spaceman};

// This CLI is rewritten in Rust for xfs_info shell script
// https://git.kernel.org/pub/scm/fs/xfs/xfsprogs-dev.git/tree/spaceman/xfs_info.sh
// since there isn't a shell on Bottlerocket hosts.

/// display XFS filesystem geometry information
#[derive(FromArgs)]
struct Args {
    /// print the version
    #[argh(switch, short = 'V')]
    version: bool,

    /// mount-point | block-device | file-image
    #[argh(positional)]
    target: Option<String>,
}

fn main() -> ExitCode {
    let args: Args = argh::from_env();
    match run(args) {
        Ok(exit_code) => exit_code,
        Err(e) => {
            eprintln!("{}", e);
            e.exit_code()
        }
    }
}

fn run(args: Args) -> Result<ExitCode> {
    // Return the version for arg version as true
    if args.version {
        let xfs_spaceman_response = query_xfs_spaceman(vec!["-p", "xfs_info", "-V"])?;
        return Ok(ExitCode::from(xfs_spaceman_response as u8));
    }

    let target = &args.target.as_ref().context(error::ParseTargetSnafu)?;

    // Get info using XFS_SPACEMAN if mounted else from XFS_DB
    let response = if let Ok(mpt) = get_mount_point(target) {
        query_xfs_spaceman(vec!["-p", "xfs_info", "-c", "info", mpt.as_str().trim()])?
    } else {
        query_xfs_db(vec!["-p", "xfs_info", "-c", "info", target.as_str()])?
    };
    Ok(ExitCode::from(response as u8))
}
