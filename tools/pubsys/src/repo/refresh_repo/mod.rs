//! The refresh_repo module owns the 'refresh-repo' subcommand and provide methods for
//! refreshing and re-signing the metadata files of a given TUF repository.

use crate::repo::{
    error as repo_error, get_signing_key_source, repo_urls, set_expirations, set_versions,
};
use crate::Args;
use chrono::{DateTime, Utc};
use clap::Parser;
use lazy_static::lazy_static;
use log::{info, trace};
use pubsys_config::{InfraConfig, RepoExpirationPolicy};
use snafu::{ensure, OptionExt, ResultExt};
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use tough::editor::RepositoryEditor;
use tough::key_source::{KeySource, LocalKeySource};
use tough::{ExpirationEnforcement, RepositoryLoader};
use url::Url;

lazy_static! {
    static ref EXPIRATION_START_TIME: DateTime<Utc> = Utc::now();
}

/// Refreshes and re-sign TUF repositories' non-root metadata files with new expiration dates
#[derive(Debug, Parser)]
pub(crate) struct RefreshRepoArgs {
    #[arg(long)]
    /// Use this named repo infrastructure from Infra.toml
    repo: String,

    #[arg(long)]
    /// The architecture of the repo being refreshed and re-signed
    arch: String,
    #[arg(long)]
    /// The variant of the repo being refreshed and re-signed
    variant: String,

    #[arg(long)]
    /// Path to root.json for this repo
    root_role_path: PathBuf,

    #[arg(long)]
    /// If we generated a local key, we'll find it here; used if Infra.toml has no key defined
    default_key_path: PathBuf,

    #[arg(long)]
    /// Path to file that defines when repo non-root metadata should expire
    repo_expiration_policy_path: PathBuf,

    #[arg(long)]
    /// Where to store the refresh/re-signed repository (just the metadata files)
    outdir: PathBuf,

    #[arg(long)]
    /// If this flag is set, repositories will succeed in loading and be refreshed even if they have
    /// expired metadata files.
    unsafe_refresh: bool,
}

fn refresh_repo(
    root_role_path: &PathBuf,
    metadata_out_dir: &PathBuf,
    metadata_url: &Url,
    targets_url: &Url,
    key_source: Box<dyn KeySource>,
    expiration: &RepoExpirationPolicy,
    unsafe_refresh: bool,
) -> Result<(), Error> {
    // If the given metadata directory exists, throw an error.  We don't want to overwrite a user's
    // existing repository.
    ensure!(
        !Path::exists(metadata_out_dir),
        repo_error::RepoExistsSnafu {
            path: metadata_out_dir
        }
    );

    let expiration_enforcement = if unsafe_refresh {
        ExpirationEnforcement::Unsafe
    } else {
        ExpirationEnforcement::Safe
    };

    // Load the repository and get the repo editor for it
    let repo = RepositoryLoader::new(
        File::open(root_role_path).context(repo_error::FileSnafu {
            path: root_role_path,
        })?,
        metadata_url.clone(),
        targets_url.clone(),
    )
    .expiration_enforcement(expiration_enforcement)
    .load()
    .context(repo_error::RepoLoadSnafu {
        metadata_base_url: metadata_url.clone(),
    })?;
    let mut repo_editor = RepositoryEditor::from_repo(root_role_path, repo)
        .context(repo_error::EditorFromRepoSnafu)?;
    info!("Loaded TUF repo: {}", metadata_url);

    // Refresh the expiration dates of all non-root metadata files
    set_expirations(&mut repo_editor, expiration, *EXPIRATION_START_TIME)?;

    // Refresh the versions of all non-root metadata files
    set_versions(&mut repo_editor)?;

    // Sign the repository
    let signed_repo = repo_editor
        .sign(&[key_source])
        .context(repo_error::RepoSignSnafu)?;

    // Write out the metadata files for the repository
    info!("Writing repo metadata to: {}", metadata_out_dir.display());
    fs::create_dir_all(metadata_out_dir).context(repo_error::CreateDirSnafu {
        path: &metadata_out_dir,
    })?;
    signed_repo
        .write(metadata_out_dir)
        .context(repo_error::RepoWriteSnafu {
            path: &metadata_out_dir,
        })?;

    Ok(())
}

/// Common entrypoint from main()
pub(crate) fn run(args: &Args, refresh_repo_args: &RefreshRepoArgs) -> Result<(), Error> {
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
        .get(&refresh_repo_args.repo)
        .context(repo_error::MissingConfigSnafu {
            missing: format!("definition for repo {}", &refresh_repo_args.repo),
        })?;

    // Check if we have a signing key defined in Infra.toml; if not, we'll fall back to the
    // generated local key.
    let signing_key_config = repo_config.signing_keys.as_ref();

    let key_source = if let Some(signing_key_config) = signing_key_config {
        get_signing_key_source(signing_key_config)?
    } else {
        ensure!(
            refresh_repo_args.default_key_path.exists(),
            repo_error::MissingConfigSnafu {
                missing: "signing_keys in repo config, and we found no local key",
            }
        );
        Box::new(LocalKeySource {
            path: refresh_repo_args.default_key_path.clone(),
        })
    };

    // Get the expiration policy
    info!(
        "Using repo expiration policy from path: {}",
        refresh_repo_args.repo_expiration_policy_path.display()
    );
    let expiration =
        RepoExpirationPolicy::from_path(&refresh_repo_args.repo_expiration_policy_path)
            .context(repo_error::ConfigSnafu)?;

    let repo_urls = repo_urls(
        repo_config,
        &refresh_repo_args.variant,
        &refresh_repo_args.arch,
    )?
    .context(repo_error::MissingRepoUrlsSnafu {
        repo: &refresh_repo_args.repo,
    })?;
    refresh_repo(
        &refresh_repo_args.root_role_path,
        &refresh_repo_args
            .outdir
            .join(&refresh_repo_args.variant)
            .join(&refresh_repo_args.arch),
        &repo_urls.0,
        repo_urls.1,
        key_source,
        &expiration,
        refresh_repo_args.unsafe_refresh,
    )?;

    Ok(())
}

mod error {
    use snafu::Snafu;
    use url::Url;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(crate) enum Error {
        #[snafu(context(false), display("{}", source))]
        Repo {
            #[snafu(source(from(crate::repo::Error, Box::new)))]
            source: Box<crate::repo::Error>,
        },

        #[snafu(display("Failed to refresh & re-sign metadata for: {:#?}", list_of_urls))]
        RepoRefresh { list_of_urls: Vec<Url> },
    }
}
pub(crate) use error::Error;
