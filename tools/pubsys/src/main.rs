/*!
`pubsys` simplifies the process of publishing Bottlerocket updates.

Currently implemented:
* building repos, whether starting from an existing repo or from scratch
* registering and copying EC2 AMIs

To be implemented:
* machine-readable output describing AMI registrations
* updating SSM parameters
* high-level document describing pubsys usage with examples

Configuration comes from:
* command-line parameters, to specify basic options and paths to the below files
* Infra.toml, for repo and AMI configuration
* Release.toml, for migrations
* Policy files for repo metadata expiration and update wave timing
*/

#![deny(rust_2018_idioms)]

mod aws;
mod config;
mod repo;

use chrono::Duration;
use parse_datetime::parse_offset;
use semver::Version;
use serde::{Deserialize, Deserializer};
use simplelog::{Config as LogConfig, LevelFilter, TermLogger, TerminalMode};
use snafu::ResultExt;
use std::path::PathBuf;
use std::process;
use structopt::StructOpt;

async fn run() -> Result<()> {
    // Parse and store the args passed to the program
    let args = Args::from_args();

    // TerminalMode::Mixed will send errors to stderr and anything less to stdout.
    TermLogger::init(args.log_level, LogConfig::default(), TerminalMode::Mixed)
        .context(error::Logger)?;

    match args.subcommand {
        SubCommand::Repo(ref repo_args) => repo::run(&args, &repo_args).context(error::Repo),
        SubCommand::Ami(ref ami_args) => aws::ami::run(&args, &ami_args).await.context(error::Ami),
    }
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{}", e);
        process::exit(1);
    }
}

/// Automates publishing of Bottlerocket updates
#[derive(Debug, StructOpt)]
#[structopt(setting = clap::AppSettings::DeriveDisplayOrder)]
struct Args {
    #[structopt(global = true, long, default_value = "INFO")]
    /// How much detail to log; from least to most: ERROR, WARN, INFO, DEBUG, TRACE
    log_level: LevelFilter,

    #[structopt(long, parse(from_os_str))]
    /// Path to Infra.toml  (NOTE: must be specified before subcommand)
    infra_config_path: PathBuf,

    #[structopt(subcommand)]
    subcommand: SubCommand,
}

#[derive(Debug, StructOpt)]
enum SubCommand {
    Repo(repo::RepoArgs),
    Ami(aws::ami::AmiArgs),
}

/// Parses a SemVer, stripping a leading 'v' if present
pub(crate) fn friendly_version(
    mut version_str: &str,
) -> std::result::Result<Version, semver::SemVerError> {
    if version_str.starts_with('v') {
        version_str = &version_str[1..];
    };

    Version::parse(version_str)
}

/// Deserializes a Duration in the form of "in X hours/days/weeks"
pub(crate) fn deserialize_offset<'de, D>(deserializer: D) -> std::result::Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    parse_offset(s).map_err(serde::de::Error::custom)
}

mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(super) enum Error {
        #[snafu(display("Logger setup error: {}", source))]
        Logger { source: simplelog::TermLogError },

        #[snafu(display("Failed to build repo: {}", source))]
        Repo { source: crate::repo::Error },

        #[snafu(display("Failed to build AMI: {}", source))]
        Ami { source: crate::aws::ami::Error },
    }
}
type Result<T> = std::result::Result<T, error::Error>;
