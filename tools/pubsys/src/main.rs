/*!
`pubsys` simplifies the process of publishing Bottlerocket updates.

Currently implemented:
* building repos, whether starting from an existing repo or from scratch
* validating repos by loading them and retrieving their targets
* checking for repository metadata expirations within specified number of days
* refreshing and re-signing repos' non-root metadata files
* registering and copying EC2 AMIs
* Marking EC2 AMIs public (or private again)
* setting SSM parameters based on built AMIs
* promoting SSM parameters from versioned entries to named (e.g. 'latest')
* validating SSM parameters by comparing the returned parameters in a region to a given list of parameters

To be implemented:
* high-level document describing pubsys usage with examples

Configuration comes from:
* command-line parameters, to specify basic options and paths to the below files
* Infra.toml, for repo and AMI configuration
* Release.toml, for migrations
* Policy files for repo metadata expiration and update wave timing
*/

mod aws;
mod repo;
mod vmware;

use clap::Parser;
use semver::Version;
use simplelog::{CombinedLogger, Config as LogConfig, ConfigBuilder, LevelFilter, SimpleLogger};
use snafu::ResultExt;
use std::path::PathBuf;
use std::process;
use tokio::runtime::Runtime;

fn run() -> Result<()> {
    // Parse and store the args passed to the program
    let args = Args::parse();

    // SimpleLogger will send errors to stderr and anything less to stdout.
    // To reduce verbosity of messages related to the AWS SDK for Rust we need
    // to spin up two loggers, setting different levels for each. This allows
    // us to retain the mixed logging of stdout/stderr in simplelog.
    match args.log_level {
        LevelFilter::Info => {
            CombinedLogger::init(vec![
                SimpleLogger::new(
                    LevelFilter::Info,
                    ConfigBuilder::new()
                        .add_filter_ignore_str("aws_config")
                        .add_filter_ignore_str("aws_credential_types")
                        .add_filter_ignore_str("aws_smithy")
                        .add_filter_ignore_str("tracing::span")
                        .build(),
                ),
                SimpleLogger::new(
                    LevelFilter::Warn,
                    ConfigBuilder::new()
                        .add_filter_allow_str("aws_config")
                        .add_filter_allow_str("aws_credential_types")
                        .add_filter_allow_str("aws_smithy")
                        .add_filter_allow_str("tracing::span")
                        .build(),
                ),
            ])
            .context(error::LoggerSnafu)?;
        }
        _ => {
            SimpleLogger::init(args.log_level, LogConfig::default()).context(error::LoggerSnafu)?
        }
    }

    match args.subcommand {
        SubCommands::Repo(ref repo_args) => repo::run(&args, repo_args).context(error::RepoSnafu),
        SubCommands::ValidateRepo(ref validate_repo_args) => {
            repo::validate_repo::run(&args, validate_repo_args).context(error::ValidateRepoSnafu)
        }
        SubCommands::CheckRepoExpirations(ref check_expirations_args) => {
            repo::check_expirations::run(&args, check_expirations_args)
                .context(error::CheckExpirationsSnafu)
        }
        SubCommands::RefreshRepo(ref refresh_repo_args) => {
            repo::refresh_repo::run(&args, refresh_repo_args).context(error::RefreshRepoSnafu)
        }
        SubCommands::Ami(ref ami_args) => {
            let rt = Runtime::new().context(error::RuntimeSnafu)?;
            rt.block_on(async {
                aws::ami::run(&args, ami_args)
                    .await
                    .context(error::AmiSnafu)
            })
        }
        SubCommands::PublishAmi(ref publish_args) => {
            let rt = Runtime::new().context(error::RuntimeSnafu)?;
            rt.block_on(async {
                aws::publish_ami::run(&args, publish_args)
                    .await
                    .context(error::PublishAmiSnafu)
            })
        }
        SubCommands::Ssm(ref ssm_args) => {
            let rt = Runtime::new().context(error::RuntimeSnafu)?;
            rt.block_on(async {
                aws::ssm::run(&args, ssm_args)
                    .await
                    .context(error::SsmSnafu)
            })
        }
        SubCommands::PromoteSsm(ref promote_args) => {
            let rt = Runtime::new().context(error::RuntimeSnafu)?;
            rt.block_on(async {
                aws::promote_ssm::run(&args, promote_args)
                    .await
                    .context(error::PromoteSsmSnafu)
            })
        }
        SubCommands::ValidateSsm(ref validate_ssm_args) => {
            let rt = Runtime::new().context(error::RuntimeSnafu)?;
            rt.block_on(async {
                aws::validate_ssm::run(&args, validate_ssm_args)
                    .await
                    .context(error::ValidateSsmSnafu)
            })
        }
        SubCommands::ValidateAmi(ref validate_ami_args) => {
            let rt = Runtime::new().context(error::RuntimeSnafu)?;
            rt.block_on(async {
                aws::validate_ami::run(&args, validate_ami_args)
                    .await
                    .context(error::ValidateAmiSnafu)
            })
        }
        SubCommands::UploadOva(ref upload_args) => {
            vmware::upload_ova::run(&args, upload_args).context(error::UploadOvaSnafu)
        }
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}

/// Automates publishing of Bottlerocket updates
#[derive(Debug, Parser)]
pub struct Args {
    #[arg(global = true, long, default_value = "INFO")]
    /// How much detail to log; from least to most: ERROR, WARN, INFO, DEBUG, TRACE
    log_level: LevelFilter,

    #[arg(long)]
    /// Path to Infra.toml (NOTE: must be specified before subcommand)
    infra_config_path: PathBuf,

    #[command(subcommand)]
    subcommand: SubCommands,
}

#[derive(Debug, Parser)]
enum SubCommands {
    Repo(repo::RepoArgs),
    ValidateRepo(repo::validate_repo::ValidateRepoArgs),
    CheckRepoExpirations(repo::check_expirations::CheckExpirationsArgs),
    RefreshRepo(repo::refresh_repo::RefreshRepoArgs),

    Ami(aws::ami::AmiArgs),
    PublishAmi(aws::publish_ami::Who),
    ValidateAmi(aws::validate_ami::ValidateAmiArgs),

    Ssm(aws::ssm::SsmArgs),
    PromoteSsm(aws::promote_ssm::PromoteArgs),
    ValidateSsm(aws::validate_ssm::ValidateSsmArgs),

    UploadOva(vmware::upload_ova::UploadArgs),
}

/// Parses a SemVer, stripping a leading 'v' if present
pub(crate) fn friendly_version(
    mut version_str: &str,
) -> std::result::Result<Version, semver::Error> {
    if version_str.starts_with('v') {
        version_str = &version_str[1..];
    };

    Version::parse(version_str)
}

mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(super) enum Error {
        #[snafu(display("Failed to build AMI: {}", source))]
        Ami { source: crate::aws::ami::Error },

        #[snafu(display("Logger setup error: {}", source))]
        Logger { source: log::SetLoggerError },

        #[snafu(display(
            "Error during publish-ami command: {}: {}",
            publish_ami_message(source),
            source
        ))]
        PublishAmi {
            source: crate::aws::publish_ami::Error,
        },

        #[snafu(display("Failed to promote SSM: {}", source))]
        PromoteSsm {
            source: crate::aws::promote_ssm::Error,
        },

        #[snafu(display("Failed to build repo: {}", source))]
        Repo { source: crate::repo::Error },

        #[snafu(display("Failed to validate repository: {}", source))]
        ValidateRepo {
            source: crate::repo::validate_repo::Error,
        },

        #[snafu(display("Check expirations error: {}", source))]
        CheckExpirations {
            source: crate::repo::check_expirations::Error,
        },

        #[snafu(display("Failed to refresh repository metadata: {}", source))]
        RefreshRepo {
            source: crate::repo::refresh_repo::Error,
        },

        #[snafu(display("Failed to create async runtime: {}", source))]
        Runtime { source: std::io::Error },

        #[snafu(display("Failed to update SSM: {}", source))]
        Ssm { source: crate::aws::ssm::Error },

        #[snafu(display("Failed to upload OVA: {}", source))]
        UploadOva {
            source: crate::vmware::upload_ova::Error,
        },

        #[snafu(display("Failed to validate SSM parameters: {}", source))]
        ValidateSsm {
            source: crate::aws::validate_ssm::Error,
        },

        #[snafu(display("Failed to validate EC2 images: {}", source))]
        ValidateAmi {
            source: crate::aws::validate_ami::Error,
        },
    }

    fn publish_ami_message(error: &crate::aws::publish_ami::Error) -> String {
        match error.amis_affected() {
            0 => String::from("No AMI permissions were updated"),
            1 => String::from("Permissions for 1 AMI were updated, the rest failed"),
            n => format!("Permissions for {} AMIs were updated, the rest failed", n),
        }
    }
}
type Result<T> = std::result::Result<T, error::Error>;
