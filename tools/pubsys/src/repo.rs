//! The repo module owns the 'repo' subcommand and controls the process of building a repository.

mod transport;

use crate::config::{InfraConfig, RepoExpirationPolicy, SigningKeyConfig};
use crate::{friendly_version, Args};
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use log::{debug, info, trace};
use parse_datetime::parse_datetime;
use semver::Version;
use snafu::{ensure, OptionExt, ResultExt};
use std::convert::TryInto;
use std::fs::{self, File};
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use tempfile::{tempdir, NamedTempFile};
use tough::{
    editor::signed::PathExists,
    editor::RepositoryEditor,
    key_source::{KeySource, LocalKeySource},
    schema::Target,
    ExpirationEnforcement, Limits, Repository, Settings,
};
use tough_kms::{KmsKeySource, KmsSigningAlgorithm};
use tough_ssm::SsmKeySource;
use transport::RepoTransport;
use update_metadata::{Images, Manifest, Release, UpdateWaves};
use url::Url;

lazy_static! {
    static ref DEFAULT_START_TIME: DateTime<Utc> = Utc::now();
}

/// Builds Bottlerocket repos using latest build artifacts
#[derive(Debug, StructOpt)]
#[structopt(setting = clap::AppSettings::DeriveDisplayOrder)]
pub(crate) struct RepoArgs {
    // Metadata about the update
    #[structopt(long)]
    /// Use this named repo from Infra.toml
    repo: String,
    #[structopt(long)]
    /// The architecture of the repo and the update being added
    arch: String,
    #[structopt(long, parse(try_from_str=friendly_version))]
    /// The version of the update being added
    version: Version,
    #[structopt(long)]
    /// The variant of the update being added
    variant: String,

    // The images to add in this update
    #[structopt(long, parse(from_os_str))]
    /// Path to the image containing the boot partition
    boot_image: PathBuf,
    #[structopt(long, parse(from_os_str))]
    /// Path to the image containing the root partition
    root_image: PathBuf,
    #[structopt(long, parse(from_os_str))]
    /// Path to the image containing the verity hashes
    hash_image: PathBuf,

    // Optionally add other files to the repo
    #[structopt(long = "link-target", parse(from_os_str))]
    /// Optional paths to add as targets and symlink into repo
    link_targets: Vec<PathBuf>,
    #[structopt(long = "copy-target", parse(from_os_str))]
    /// Optional paths to add as targets and copy into repo
    copy_targets: Vec<PathBuf>,

    // Policies that pubsys interprets to set repo parameters
    #[structopt(long, parse(from_os_str))]
    /// Path to file that defines when repo metadata should expire
    repo_expiration_policy_path: PathBuf,

    // Policies that pubsys passes on to other tools
    #[structopt(long, parse(from_os_str))]
    /// Path to Release.toml
    release_config_path: PathBuf,
    #[structopt(long, parse(from_os_str))]
    /// Path to file that defines when this update will become available
    wave_policy_path: PathBuf,

    #[structopt(long, parse(try_from_str = parse_datetime))]
    /// When the waves and expiration timer will start; RFC3339 date or "in X hours/days/weeks"
    release_start_time: Option<DateTime<Utc>>,

    #[structopt(long)]
    /// Use this named key from Infra.toml
    signing_key: String,

    #[structopt(long, parse(from_os_str))]
    /// Where to store the created repo
    outdir: PathBuf,
}

/// Adds update, migrations, and waves to the Manifest
fn update_manifest(repo_args: &RepoArgs, manifest: &mut Manifest) -> Result<()> {
    // Add update   =^..^=   =^..^=   =^..^=   =^..^=

    let filename = |path: &PathBuf| -> Result<String> {
        Ok(path
            .file_name()
            .context(error::InvalidImagePath { path })?
            .to_str()
            .context(error::NonUtf8Path { path })?
            .to_string())
    };

    let images = Images {
        boot: filename(&repo_args.boot_image)?,
        root: filename(&repo_args.root_image)?,
        hash: filename(&repo_args.hash_image)?,
    };

    info!(
        "Adding update to manifest for version: {}, arch: {}, variant: {}",
        repo_args.version, repo_args.arch, repo_args.variant
    );
    manifest
        .add_update(
            repo_args.version.clone(),
            None,
            repo_args.arch.clone(),
            repo_args.variant.clone(),
            images,
        )
        .context(error::AddUpdate)?;

    // Add migrations   =^..^=   =^..^=   =^..^=   =^..^=

    info!(
        "Using release config from path: {}",
        repo_args.release_config_path.display()
    );
    let release =
        Release::from_path(&repo_args.release_config_path).context(error::UpdateMetadataRead {
            path: &repo_args.release_config_path,
        })?;
    trace!(
        "Adding migrations to manifest for versions: {:#?}",
        release
            .migrations
            .keys()
            .map(|(from, to)| format!("({}, {})", from, to))
            .collect::<Vec<String>>()
    );
    // Replace the manifest 'migrations' section with the new data
    manifest.migrations = release.migrations;

    // Add update waves   =^..^=   =^..^=   =^..^=   =^..^=

    let wave_start_time = repo_args.release_start_time.unwrap_or(*DEFAULT_START_TIME);
    info!(
        "Using wave policy from path: {}",
        repo_args.wave_policy_path.display()
    );
    info!(
        "Offsets from that file will be added to the release start time of: {}",
        wave_start_time
    );
    let waves =
        UpdateWaves::from_path(&repo_args.wave_policy_path).context(error::UpdateMetadataRead {
            path: &repo_args.wave_policy_path,
        })?;
    manifest
        .set_waves(
            repo_args.variant.clone(),
            repo_args.arch.clone(),
            repo_args.version.clone(),
            wave_start_time,
            &waves,
        )
        .context(error::SetWaves {
            wave_policy_path: &repo_args.wave_policy_path,
        })?;

    Ok(())
}

/// Adds targets, expirations, and version to the RepositoryEditor
fn update_editor<'a, P>(
    repo_args: &'a RepoArgs,
    editor: &mut RepositoryEditor<'a, RepoTransport>,
    targets: impl Iterator<Item = &'a PathBuf>,
    manifest_path: P,
) -> Result<()>
where
    P: AsRef<Path>,
{
    // Add targets   =^..^=   =^..^=   =^..^=   =^..^=

    for target_path in targets {
        debug!("Adding target from path: {}", target_path.display());
        editor
            .add_target_path(&target_path)
            .context(error::AddTarget { path: &target_path })?;
    }

    let manifest_target = Target::from_path(&manifest_path).context(error::BuildTarget {
        path: manifest_path.as_ref(),
    })?;
    debug!("Adding target for manifest.json");
    editor.add_target("manifest.json", manifest_target).context(error::AddTarget { path: "manifest.json" })?;

    // Add expirations   =^..^=   =^..^=   =^..^=   =^..^=

    info!(
        "Using repo expiration policy from path: {}",
        repo_args.repo_expiration_policy_path.display()
    );
    let expiration = RepoExpirationPolicy::from_path(&repo_args.repo_expiration_policy_path)
        .context(error::Config)?;

    let expiration_start_time = repo_args.release_start_time.unwrap_or(*DEFAULT_START_TIME);
    let snapshot_expiration = expiration_start_time + expiration.snapshot_expiration;
    let targets_expiration = expiration_start_time + expiration.targets_expiration;
    let timestamp_expiration = expiration_start_time + expiration.timestamp_expiration;
    info!(
        "Repo expiration times:\n\tsnapshot:  {}\n\ttargets:   {}\n\ttimestamp: {}",
        snapshot_expiration, targets_expiration, timestamp_expiration
    );
    editor
        .snapshot_expires(snapshot_expiration)
        .targets_expires(targets_expiration)
        .context(error::SetTargetsExpiration {
            expiration: targets_expiration,
        })?
        .timestamp_expires(timestamp_expiration);

    // Add version   =^..^=   =^..^=   =^..^=   =^..^=

    let seconds = Utc::now().timestamp();
    let unsigned_seconds = seconds.try_into().expect("System clock before 1970??");
    let version = NonZeroU64::new(unsigned_seconds).expect("System clock exactly 1970??");
    debug!("Repo version: {}", version);
    editor
        .snapshot_version(version)
        .targets_version(version)
        .context(error::SetTargetsVersion { version })?
        .timestamp_version(version);

    Ok(())
}

/// If the infra config has a repo section defined for the given repo, and it has metadata base and
/// targets URLs defined, returns those URLs, otherwise None.
fn repo_urls<'a>(
    repo_args: &RepoArgs,
    infra_config: &'a InfraConfig,
) -> Result<Option<(Url, &'a Url)>> {
    let repo_config = infra_config
        .repo
        .as_ref()
        .context(error::MissingConfig {
            missing: "repo section",
        })?
        .get(&repo_args.repo)
        .context(error::MissingConfig {
            missing: format!("definition for repo {}", &repo_args.repo),
        })?;

    // Check if both URLs are set
    if let Some(metadata_base_url) = repo_config.metadata_base_url.as_ref() {
        if let Some(targets_url) = repo_config.targets_url.as_ref() {
            let base_slash = if metadata_base_url.as_str().ends_with('/') {
                ""
            } else {
                "/"
            };
            let metadata_url_str = format!(
                "{}{}{}/{}",
                metadata_base_url, base_slash, repo_args.variant, repo_args.arch
            );
            let metadata_url = Url::parse(&metadata_url_str).context(error::ParseUrl {
                input: &metadata_url_str,
            })?;

            debug!("Using metadata url: {}", metadata_url);
            return Ok(Some((metadata_url, targets_url)));
        }
    }

    Ok(None)
}

/// Builds an editor and manifest; will start from an existing repo if one is specified in the
/// configuration.  Returns Err if we fail to read from the repo.  Returns Ok(None) if we detect
/// that the repo does not exist.
fn load_editor_and_manifest<'a, P>(
    root_role_path: P,
    transport: &'a RepoTransport,
    datastore: &'a Path,
    metadata_url: &'a Url,
    targets_url: &'a Url,
) -> Result<Option<(RepositoryEditor<'a, RepoTransport>, Manifest)>>
where
    P: AsRef<Path>,
{
    let root_role_path = root_role_path.as_ref();

    // Create a temporary directory where the TUF client can store metadata
    let settings = Settings {
        root: File::open(root_role_path).context(error::File {
            path: root_role_path,
        })?,
        datastore,
        metadata_base_url: metadata_url.as_str(),
        targets_base_url: targets_url.as_str(),
        limits: Limits::default(),
        expiration_enforcement: ExpirationEnforcement::Safe,
    };

    // Try to load the repo...
    match Repository::load(transport, settings) {
        // If we load it successfully, build an editor and manifest from it.
        Ok(repo) => {
            let reader = repo
                .read_target("manifest.json")
                .context(error::ReadTarget {
                    target: "manifest.json",
                })?
                .with_context(|| error::NoManifest {
                    metadata_url: metadata_url.clone(),
                })?;
            let manifest = serde_json::from_reader(reader).context(error::InvalidJson {
                path: "manifest.json",
            })?;

            let editor =
                RepositoryEditor::from_repo(root_role_path, repo).context(error::EditorFromRepo)?;

            Ok(Some((editor, manifest)))
        }
        // If we fail to load, but we only failed because the repo doesn't exist yet, then start
        // fresh by signalling that there is no known repo.  Otherwise, fail hard.
        Err(e) => {
            if transport.repo_not_found.get() {
                Ok(None)
            } else {
                Err(e).with_context(|| error::RepoLoad {
                    metadata_base_url: metadata_url.clone(),
                })?
            }
        }
    }
}

/// Common entrypoint from main()
pub(crate) fn run(args: &Args, repo_args: &RepoArgs) -> Result<()> {
    let metadata_out_dir = repo_args
        .outdir
        .join(&repo_args.variant)
        .join(&repo_args.arch);
    let targets_out_dir = repo_args.outdir.join("targets");

    // If the given metadata directory exists, throw an error.  We dont want to overwrite a user's
    // existing repository.  (The targets directory is shared, so it's fine if that exists.)
    ensure!(
        !Path::exists(&metadata_out_dir),
        error::RepoExists {
            path: metadata_out_dir
        }
    );

    // Build repo   =^..^=   =^..^=   =^..^=   =^..^=

    info!(
        "Using infra config from path: {}",
        args.infra_config_path.display()
    );
    let infra_config = InfraConfig::from_path(&args.infra_config_path).context(error::Config)?;
    trace!("Parsed infra config: {:?}", infra_config);
    let root_role_path = infra_config
        .root_role_path
        .as_ref()
        .context(error::MissingConfig {
            missing: "root_role_path",
        })?;

    // Build a repo editor and manifest, from an existing repo if available, otherwise fresh
    let maybe_urls = repo_urls(&repo_args, &infra_config)?;
    let workdir = tempdir().context(error::TempDir)?;
    let transport = RepoTransport::default();
    let (mut editor, mut manifest) = if let Some((metadata_url, targets_url)) = maybe_urls.as_ref() {
        info!("Found metadata and target URLs, loading existing repository");
        match load_editor_and_manifest(root_role_path, &transport, workdir.path(), &metadata_url, &targets_url)? {
            Some((editor, manifest)) => (editor, manifest),
            None => {
                info!(
                    "Did not find repo at '{}', starting a new one",
                    metadata_url
                );
                (
                    RepositoryEditor::new(root_role_path).context(error::NewEditor)?,
                    Manifest::default(),
                )
            }
        }
    } else {
        info!("Did not find metadata and target URLs in infra config, creating a new repository");
        (
            RepositoryEditor::new(root_role_path).context(error::NewEditor)?,
            Manifest::default(),
        )
    };

    // Add update information to manifest
    update_manifest(&repo_args, &mut manifest)?;
    // Write manifest to tempfile so it can be copied in as target later
    let manifest_path = NamedTempFile::new()
        .context(error::TempFile)?
        .into_temp_path();
    update_metadata::write_file(&manifest_path, &manifest).context(error::ManifestWrite {
        path: &manifest_path,
    })?;

    // Add manifest and targets to editor
    let copy_targets = &repo_args.copy_targets;
    let link_targets = repo_args.link_targets.iter().chain(vec![
        &repo_args.boot_image,
        &repo_args.root_image,
        &repo_args.hash_image,
    ]);
    let all_targets = copy_targets.iter().chain(link_targets.clone());

    update_editor(&repo_args, &mut editor, all_targets, &manifest_path)?;

    // Sign repo   =^..^=   =^..^=   =^..^=   =^..^=

    let signing_key_config = infra_config
        .signing_keys
        .as_ref()
        .context(error::MissingConfig {
            missing: "signing_keys",
        })?
        .get(&repo_args.signing_key)
        .context(error::MissingConfig {
            missing: format!("profile {} in signing_keys", &repo_args.signing_key),
        })?;

    let key_source: Box<dyn KeySource> = match signing_key_config {
        SigningKeyConfig::file { path } => Box::new(LocalKeySource { path: path.clone() }),
        SigningKeyConfig::kms { key_id } => Box::new(KmsKeySource {
            profile: None,
            key_id: key_id.clone(),
            client: None,
            signing_algorithm: KmsSigningAlgorithm::RsassaPssSha256,
        }),
        SigningKeyConfig::ssm { parameter } => Box::new(SsmKeySource {
            profile: None,
            parameter_name: parameter.clone(),
            key_id: None,
        }),
    };

    let signed_repo = editor.sign(&[key_source]).context(error::RepoSign)?;

    // Write repo   =^..^=   =^..^=   =^..^=   =^..^=

    // Write targets first so we don't have invalid metadata if targets fail
    info!("Writing repo targets to: {}", targets_out_dir.display());
    fs::create_dir_all(&targets_out_dir).context(error::CreateDir {
        path: &targets_out_dir,
    })?;

    // Copy manifest with proper name instead of tempfile name
    debug!("Copying manifest.json into {}", targets_out_dir.display());
    signed_repo
        .copy_target(
            &manifest_path,
            &targets_out_dir,
            // We should never have matching manifests from different repos
            PathExists::Fail,
            Some("manifest.json"),
        )
        .context(error::CopyTarget {
            target: &manifest_path,
            path: &targets_out_dir,
        })?;

    // Copy / link any other user requested targets
    for copy_target in copy_targets {
        debug!(
            "Copying target '{}' into {}",
            copy_target.display(),
            targets_out_dir.display()
        );
        signed_repo
            .copy_target(copy_target, &targets_out_dir, PathExists::Skip, None)
            .context(error::CopyTarget {
                target: copy_target,
                path: &targets_out_dir,
            })?;
    }
    for link_target in link_targets {
        debug!(
            "Linking target '{}' into {}",
            link_target.display(),
            targets_out_dir.display()
        );
        signed_repo
            .link_target(link_target, &targets_out_dir, PathExists::Skip, None)
            .context(error::LinkTarget {
                target: link_target,
                path: &targets_out_dir,
            })?;
    }

    info!("Writing repo metadata to: {}", metadata_out_dir.display());
    fs::create_dir_all(&metadata_out_dir).context(error::CreateDir {
        path: &metadata_out_dir,
    })?;
    signed_repo
        .write(&metadata_out_dir)
        .context(error::RepoWrite {
            path: &repo_args.outdir,
        })?;

    Ok(())
}

mod error {
    use chrono::{DateTime, Utc};
    use snafu::Snafu;
    use std::io;
    use std::path::PathBuf;
    use url::Url;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(crate) enum Error {
        #[snafu(display("Failed to add new update to manifest: {}", source))]
        AddUpdate {
            source: update_metadata::error::Error,
        },

        #[snafu(display("Failed to add new target '{}' to repo: {}", path.display(), source))]
        AddTarget {
            path: PathBuf,
            source: tough::error::Error,
        },

        #[snafu(display("Failed to build target metadata from path '{}': {}", path.display(), source))]
        BuildTarget {
            path: PathBuf,
            source: tough::schema::Error,
        },

        #[snafu(display("Failed to copy target '{}' to '{}': {}", target.display(), path.display(), source))]
        CopyTarget {
            target: PathBuf,
            path: PathBuf,
            source: tough::error::Error,
        },

        #[snafu(display("Error reading config: {}", source))]
        Config { source: crate::config::Error },

        #[snafu(display("Failed to create directory '{}': {}", path.display(), source))]
        CreateDir { path: PathBuf, source: io::Error },

        #[snafu(display("Failed to create repo editor from given repo: {}", source))]
        EditorFromRepo { source: tough::error::Error },

        #[snafu(display("Failed to read '{}': {}", path.display(), source))]
        File { path: PathBuf, source: io::Error },

        #[snafu(display("Invalid path given for image file: '{}'", path.display()))]
        InvalidImagePath { path: PathBuf },

        #[snafu(display("Invalid config file at '{}': {}", path.display(), source))]
        InvalidJson {
            path: PathBuf,
            source: serde_json::Error,
        },

        #[snafu(display("Failed to symlink target '{}' to '{}': {}", target.display(), path.display(), source))]
        LinkTarget {
            target: PathBuf,
            path: PathBuf,
            source: tough::error::Error,
        },

        #[snafu(display("Failed to write Manifest to '{}': {}", path.display(), source))]
        ManifestWrite {
            path: PathBuf,
            source: update_metadata::error::Error,
        },

        #[snafu(display("Infra.toml is missing {}", missing))]
        MissingConfig { missing: String },

        #[snafu(display("Failed to create new repo editor: {}", source))]
        NewEditor { source: tough::error::Error },

        #[snafu(display("Repo does not have a manifest.json: {}", metadata_url))]
        NoManifest { metadata_url: Url },

        #[snafu(display("Non-UTF8 path '{}' not supported", path.display()))]
        NonUtf8Path { path: PathBuf },

        #[snafu(display("Invalid URL '{}': {}", input, source))]
        ParseUrl {
            input: String,
            source: url::ParseError,
        },

        #[snafu(display("Failed to read target '{}' from repo: {}", target, source))]
        ReadTarget {
            target: String,
            source: tough::error::Error,
        },

        #[snafu(display("Repo exists at '{}' - remove it and try again", path.display()))]
        RepoExists { path: PathBuf },

        #[snafu(display("Could not fetch repo at '{}': {}", url, msg))]
        RepoFetch { url: Url, msg: String },

        #[snafu(display(
            "Failed to load repo from metadata URL '{}': {}",
            metadata_base_url,
            source
        ))]
        RepoLoad {
            metadata_base_url: Url,
            source: tough::error::Error,
        },

        #[snafu(display("Requested repository does not exist: '{}'", url))]
        RepoNotFound { url: Url },

        #[snafu(display("Failed to sign repository: {}", source))]
        RepoSign { source: tough::error::Error },

        #[snafu(display("Failed to write repository to {}: {}", path.display(), source))]
        RepoWrite {
            path: PathBuf,
            source: tough::error::Error,
        },

        #[snafu(display("Failed to set targets expiration to {}: {}", expiration, source))]
        SetTargetsExpiration {
            expiration: DateTime<Utc>,
            source: tough::error::Error,
        },

        #[snafu(display("Failed to set targets version to {}: {}", version, source))]
        SetTargetsVersion {
            version: u64,
            source: tough::error::Error,
        },

        #[snafu(display("Failed to set waves from '{}': {}", wave_policy_path.display(), source))]
        SetWaves {
            wave_policy_path: PathBuf,
            source: update_metadata::error::Error,
        },

        #[snafu(display("Failed to create tempdir: {}", source))]
        TempDir { source: io::Error },

        #[snafu(display("Failed to create temporary file: {}", source))]
        TempFile { source: io::Error },

        #[snafu(display("Failed to read update metadata '{}': {}", path.display(), source))]
        UpdateMetadataRead {
            path: PathBuf,
            source: update_metadata::error::Error,
        },
    }
}
pub(crate) use error::Error;
type Result<T> = std::result::Result<T, error::Error>;
