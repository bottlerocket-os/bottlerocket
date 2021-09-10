//! The validate_repo module owns the 'validate-repo' subcommand and provides methods for validating
//! a given TUF repository by attempting to load the repository and download its targets.

use crate::repo::{error as repo_error, repo_urls};
use crate::Args;
use log::{info, trace};
use pubsys_config::InfraConfig;
use snafu::{OptionExt, ResultExt};
use std::cmp::min;
use std::fs::File;
use std::io;
use std::path::PathBuf;
use std::sync::mpsc;
use structopt::StructOpt;
use tough::{Repository, RepositoryLoader};
use url::Url;

/// Validates a set of TUF repositories
#[derive(Debug, StructOpt)]
#[structopt(setting = clap::AppSettings::DeriveDisplayOrder)]
pub(crate) struct ValidateRepoArgs {
    #[structopt(long)]
    /// Use this named repo infrastructure from Infra.toml
    repo: String,

    #[structopt(long)]
    /// The architecture of the repo being validated
    arch: String,
    #[structopt(long)]
    /// The variant of the repo being validated
    variant: String,

    #[structopt(long, parse(from_os_str))]
    /// Path to root.json for this repo
    root_role_path: PathBuf,

    #[structopt(long)]
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
        .context(error::ThreadPool)?;

    // create the channels through which our download results will be passed
    let (tx, rx) = mpsc::channel();

    for target in targets.keys().cloned() {
        let tx = tx.clone();
        let mut reader = repo
            .read_target(&target)
            .with_context(|| repo_error::ReadTarget {
                target: target.to_string(),
            })?
            .with_context(|| error::TargetMissing {
                target: target.to_string(),
            })?;
        info!("Downloading target: {}", target);
        thread_pool.spawn(move || {
            tx.send({
                // tough's `Read` implementation validates the target as it's being downloaded
                io::copy(&mut reader, &mut io::sink()).context(error::TargetDownload {
                    target: target.to_string(),
                })
            })
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

fn validate_repo(
    root_role_path: &PathBuf,
    metadata_url: Url,
    targets_url: &Url,
    validate_targets: bool,
) -> Result<(), Error> {
    // Load the repository
    let repo = RepositoryLoader::new(
        File::open(root_role_path).context(repo_error::File {
            path: root_role_path,
        })?,
        metadata_url.clone(),
        targets_url.clone(),
    )
    .load()
    .context(repo_error::RepoLoad {
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
        .context(repo_error::Config)?;
    trace!("Parsed infra config: {:?}", infra_config);
    let repo_config = infra_config
        .repo
        .as_ref()
        .context(repo_error::MissingConfig {
            missing: "repo section",
        })?
        .get(&validate_repo_args.repo)
        .context(repo_error::MissingConfig {
            missing: format!("definition for repo {}", &validate_repo_args.repo),
        })?;

    let repo_urls = repo_urls(
        &repo_config,
        &validate_repo_args.variant,
        &validate_repo_args.arch,
    )?
    .context(repo_error::MissingRepoUrls {
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
    #[snafu(visibility = "pub(super)")]
    pub(crate) enum Error {
        #[snafu(display("Invalid percentage specified: {} is greater than 100", percentage))]
        InvalidPercentage { percentage: u8 },

        #[snafu(context(false), display("{}", source))]
        Repo { source: crate::repo::Error },

        #[snafu(display("Failed to download and write target '{}': {}", target, source))]
        TargetDownload { target: String, source: io::Error },

        #[snafu(display("Missing target: {}", target))]
        TargetMissing { target: String },

        #[snafu(display("Unable to create thread pool: {}", source))]
        ThreadPool { source: rayon::ThreadPoolBuildError },
    }
}
pub(crate) use error::Error;
