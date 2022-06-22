/*!
# Introduction

shibaken is called by sundog as a setting generator.

shibaken is used to fetch data from the instance metadata service (IMDS) in AWS.

shibaken can:
* Fetch and populate the admin container's user-data with authorized ssh keys from the IMDS.
* Perform boolean queries about the AWS partition in which the host is located.

(The name "shibaken" comes from the fact that Shiba are small, but agile, hunting dogs.)
*/

#![deny(rust_2018_idioms)]

use argh::FromArgs;
use simplelog::{ColorChoice, Config as LogConfig, LevelFilter, TermLogger, TerminalMode};
use snafu::ResultExt;
use std::process;

use crate::error::Result;

mod admin_userdata;
pub(crate) mod error;
mod partition;

/// Returns information gathered from the AWS instance metadata service (IMDS).
#[derive(FromArgs, Debug)]
struct Args {
    #[argh(option, default = "LevelFilter::Info")]
    /// filter level for log messages
    log_level: LevelFilter,

    #[argh(subcommand)]
    command: Commands,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand)]
enum Commands {
    /// Fetch and populate the admin container's user-data with authorized ssh keys.
    GenerateAdminUserdata(admin_userdata::GenerateAdminUserdata),

    /// Fetch and return whether or not this host is in the given partition.
    /// Accepts multiple partitions, returning `true` if the host is in any of the given partitions.
    IsPartition(partition::IsPartition),
}

async fn run() -> Result<()> {
    let args: Args = argh::from_env();

    // TerminalMode::Stderr will send all logs to stderr, as sundog only expects the json output of
    // the setting on stdout.
    TermLogger::init(
        args.log_level,
        LogConfig::default(),
        TerminalMode::Stderr,
        ColorChoice::Auto,
    )
    .context(error::LoggerSnafu)?;

    log::info!("shibaken started");

    match args.command {
        Commands::GenerateAdminUserdata(generate_admin_userdata) => {
            generate_admin_userdata.run().await
        }
        Commands::IsPartition(is_partition) => is_partition.run().await,
    }
}

// Returning a Result from main makes it print a Debug representation of the error, but with Snafu
// we have nice Display representations of the error, so we wrap "main" (run) and print any error.
// https://github.com/shepmaster/snafu/issues/110
#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{}", e);
        process::exit(1);
    }
}
