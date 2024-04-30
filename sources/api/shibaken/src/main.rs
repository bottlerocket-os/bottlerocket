/*!
# Introduction

shibaken is called by sundog as a setting generator.

shibaken is used to fetch data from the instance metadata service (IMDS) in AWS.

shibaken can:
* Fetch and populate the admin container's user-data with authorized ssh keys from the IMDS.
* Perform boolean queries about the AWS partition in which the host is located.
* Wait in a warm pool until the instance is marked as InService before starting the orchestrator.

(The name "shibaken" comes from the fact that Shiba are small, but agile, hunting dogs.)
*/

use argh::FromArgs;
use simplelog::{ColorChoice, Config as LogConfig, LevelFilter, TermLogger, TerminalMode};
use snafu::ResultExt;
use std::process::ExitCode;

use crate::error::Result;

mod admin_userdata;
pub(crate) mod error;
mod value_equal;
mod partition;
mod warmpool;

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

    /// Match IMDS against one or more values, return true/false.
    DoesValueStartWith(value_equal::ValueStartsWith),

    /// Fetch and return whether or not this host is in the given partition.
    /// Accepts multiple partitions, returning `true` if the host is in any of the given partitions.
    IsPartition(partition::IsPartition),

    /// Poll lifecycle state and wait until instance to be marked as InService
    WarmPoolWait(warmpool::autoscaling_warm_pool::WarmPoolWait),
}

async fn run() -> Result<ExitCode> {
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
    let mut exit_code = ExitCode::SUCCESS;
    match args.command {
        Commands::GenerateAdminUserdata(generate_admin_userdata) => generate_admin_userdata.run().await?,
        Commands::DoesValueStartWith(does_value_start) => exit_code = does_value_start.run().await?,
        Commands::IsPartition(is_partition) => is_partition.run().await?,
        Commands::WarmPoolWait(warm_pool_wait) => warm_pool_wait
            .run()
            .await
            .context(error::WarmPoolCheckFailedSnafu)?,
    }
    Ok(exit_code)
}

// Returning an ExitCode from main will propagate the success or failure to our caller, and permit
// normal rust teardown (unlike process::exit()): a kinder, gentler failure branch.
#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Err(e) => {
            eprintln!("{}", e);
            ExitCode::FAILURE
        }
        Ok(code) => code
    }
}
