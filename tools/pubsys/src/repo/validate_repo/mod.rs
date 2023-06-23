//! The validate_repo module owns the 'validate-repo' subcommand and provides methods for validating
//! a given TUF repository by attempting to load the repository and download its targets.

use crate::repo::{error as repo_error, repo_urls};
use crate::Args;
use clap::Parser;
use log::{info, trace};
use pubsys_config::InfraConfig;
use snafu::{OptionExt, ResultExt};
use std::cmp::min;
use std::fs::File;
use std::io;
use std::path::PathBuf;
use std::sync::mpsc;
use tough::{Repository, RepositoryLoader, TargetName};
use url::Url;

/// Validates a set of TUF repositories
#[derive(Debug, Parser)]
pub(crate) struct ValidateRepoArgs {
    #[arg(long)]
    /// Use this named repo infrastructure from Infra.toml
    repo: String,

    #[arg(long)]
    /// The architecture of the repo being validated
    arch: String,
    #[arg(long)]
    /// The variant of the repo being validated
    variant: String,

    #[arg(long)]
    /// Path to root.json for this repo
    root_role_path: PathBuf,

    #[arg(long)]
    /// Specifies whether to validate all listed targets by attempting to download them
    validate_targets: bool,
}

/// If we are on a machine with a large number of cores, then we limit the number of simultaneous
/// downloads to this arbitrarily chosen maximum.
const MAX_DOWNLOAD_THREADS: usize = 16;

/// Retrieves listed targets and attempts to download them for validation purposes. We use a Rayon
/// thread pool instead of tokio for async execution because `reqwest::blocking` creates a tokio
/// runtime (and multiple tokio runtimes are not supported).
fn retrieve_targets(repo: &Repository) -> Result<(), Error> {
    let targets = &repo.targets().signed.targets;
    let thread_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(min(num_cpus::get(), MAX_DOWNLOAD_THREADS))
        .build()
        .context(error::ThreadPoolSnafu)?;

    // create the channels through which our download results will be passed
    let (tx, rx) = mpsc::channel();

    for target in targets.keys() {
        let repo = repo.clone();
        let tx = tx.clone();
        info!("Downloading target: {}", target.raw());
        let target = target.clone();
        thread_pool.spawn(move || {
            tx.send(download_targets(&repo, target))
                // inability to send on this channel is unrecoverable
                .unwrap();
        });
    }
    // close all senders
    drop(tx);

    // block and await all downloads
    let results: Vec<Result<u64, error::Error>> = rx.into_iter().collect();

    // check all results and return the first error we see
    for result in results {
        result?;
    }

    // no errors were found, the targets are validated
    Ok(())
}

fn download_targets(repo: &Repository, target: TargetName) -> Result<u64, Error> {
    let mut reader = match repo.read_target(&target) {
        Ok(Some(reader)) => reader,
        Ok(None) => {
            return error::TargetMissingSnafu {
                target: target.raw(),
            }
            .fail()
        }
        Err(e) => {
            return Err(e).context(error::TargetReadSnafu {
                target: target.raw(),
            })
        }
    };
    // tough's `Read` implementation validates the target as it's being downloaded
    io::copy(&mut reader, &mut io::sink()).context(error::TargetDownloadSnafu {
        target: target.raw(),
    })
}

fn validate_repo(
    root_role_path: &PathBuf,
    metadata_url: Url,
    targets_url: &Url,
    validate_targets: bool,
) -> Result<(), Error> {
    // Load the repository
    let repo = RepositoryLoader::new(
        File::open(root_role_path).context(repo_error::FileSnafu {
            path: root_role_path,
        })?,
        metadata_url.clone(),
        targets_url.clone(),
    )
    .load()
    .context(repo_error::RepoLoadSnafu {
        metadata_base_url: metadata_url.clone(),
    })?;
    info!("Loaded TUF repo: {}", metadata_url);
    if validate_targets {
        // Try retrieving listed targets
        retrieve_targets(&repo)?;
    }

    Ok(())
}

/// Common entrypoint from main()
pub(crate) fn run(args: &Args, validate_repo_args: &ValidateRepoArgs) -> Result<(), Error> {
    // If a lock file exists, use that, otherwise use Infra.toml
    let infra_config = InfraConfig::from_path_or_lock(&args.infra_config_path, false)
        .context(repo_error::ConfigSnafu)?;
    trace!("Parsed infra config: {:?}", infra_config);
    let repo_config = infra_config
        .repo
        .as_ref()
        .context(repo_error::MissingConfigSnafu {
            missing: "repo section",
        })?
        .get(&validate_repo_args.repo)
        .context(repo_error::MissingConfigSnafu {
            missing: format!("definition for repo {}", &validate_repo_args.repo),
        })?;

    let repo_urls = repo_urls(
        repo_config,
        &validate_repo_args.variant,
        &validate_repo_args.arch,
    )?
    .context(repo_error::MissingRepoUrlsSnafu {
        repo: &validate_repo_args.repo,
    })?;
    validate_repo(
        &validate_repo_args.root_role_path,
        repo_urls.0,
        repo_urls.1,
        validate_repo_args.validate_targets,
    )
}

mod error {
    use snafu::Snafu;
    use std::io;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(crate) enum Error {
        #[snafu(display("Invalid percentage specified: {} is greater than 100", percentage))]
        InvalidPercentage { percentage: u8 },

        #[snafu(context(false), display("{}", source))]
        Repo {
            #[snafu(source(from(crate::repo::Error, Box::new)))]
            source: Box<crate::repo::Error>,
        },

        #[snafu(display("Failed to download and write target '{}': {}", target, source))]
        TargetDownload { target: String, source: io::Error },

        #[snafu(display("Missing target: {}", target))]
        TargetMissing { target: String },

        #[snafu(display("Failed to read target '{}' from repo: {}", target, source))]
        TargetRead {
            target: String,
            #[snafu(source(from(tough::error::Error, Box::new)))]
            source: Box<tough::error::Error>,
        },

        #[snafu(display("Unable to create thread pool: {}", source))]
        ThreadPool { source: rayon::ThreadPoolBuildError },
    }
}
pub(crate) use error::Error;
