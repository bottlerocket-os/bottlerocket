//! This module owns the error type used by the migrator.

use semver::Version;
use snafu::Snafu;
use std::io;
use std::path::PathBuf;
use std::process::Output;

/// Error contains the errors that can happen during migration.
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub(crate) enum Error {
    #[snafu(display("Data store path '{}' contains invalid UTF-8", path.display()))]
    DataStorePathNotUTF8 { path: PathBuf },

    #[snafu(display("Unable to open data store directory '{}': {}", path.display(), source))]
    DataStoreDirOpen { path: PathBuf, source: nix::Error },

    #[snafu(display("Data store link '{}' points to /", path.display()))]
    DataStoreLinkToRoot { path: PathBuf },

    #[snafu(display("Unable to create URL from path '{}'", path.display()))]
    DirectoryUrl { path: PathBuf },

    #[snafu(display("Error finding migration: {}", source))]
    FindMigrations {
        source: update_metadata::error::Error,
    },

    #[snafu(display("Data store path '{}' contains invalid version: {}", path.display(), source))]
    InvalidDataStoreVersion {
        path: PathBuf,
        source: semver::Error,
    },

    #[snafu(display("Data store for new version {} already exists at {}", version, path.display()))]
    NewVersionAlreadyExists { version: Version, path: PathBuf },

    #[snafu(display("Unable to seal migration command: {}", source))]
    SealMigration { source: std::io::Error },

    #[snafu(display("Unable to start migration command: {}", source))]
    StartMigration { source: std::io::Error },

    #[snafu(display("Migration returned '{}' - stderr: {}",
                    output.status.code()
                        .map(|i| i.to_string()).unwrap_or_else(|| "signal".to_string()),
                    std::str::from_utf8(&output.stderr)
                        .unwrap_or("<invalid UTF-8>")))]
    MigrationFailure { output: Output },

    #[snafu(display("Failed to create symlink for new version at {}: {}", path.display(), source))]
    LinkCreate { path: PathBuf, source: io::Error },

    #[snafu(display("Failed to swap symlink at {} to new version: {}", link.display(), source))]
    LinkSwap { link: PathBuf, source: io::Error },

    #[snafu(display("Failed to read symlink at {} to find version: {}", link.display(), source))]
    LinkRead { link: PathBuf, source: io::Error },

    #[snafu(display("Invalid target name '{}': {}", target, source))]
    TargetName {
        target: String,
        #[snafu(source(from(tough::error::Error, Box::new)))]
        source: Box<tough::error::Error>,
    },

    #[snafu(display("Error loading migration '{}': {}", migration, source))]
    LoadMigration {
        migration: String,
        #[snafu(source(from(tough::error::Error, Box::new)))]
        source: Box<tough::error::Error>,
    },

    #[snafu(display("Failed to decode LZ4-compressed migration {}: {}", migration, source))]
    Lz4Decode {
        migration: String,
        source: std::io::Error,
    },

    #[snafu(display("Error loading manifest: {}", source))]
    ManifestLoad {
        #[snafu(source(from(tough::error::Error, Box::new)))]
        source: Box<tough::error::Error>,
    },

    #[snafu(display("Manifest not found in repository"))]
    ManifestNotFound,

    #[snafu(display("Error parsing manifest: {}", source))]
    ManifestParse {
        source: update_metadata::error::Error,
    },

    #[snafu(display("Migration '{}' not found", migration))]
    MigrationNotFound { migration: String },

    #[snafu(display("Failed to open trusted root metadata file {}: {}", path.display(), source))]
    OpenRoot {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("Failed to load TUF repo: {}", source))]
    RepoLoad {
        #[snafu(source(from(tough::error::Error, Box::new)))]
        source: Box<tough::error::Error>,
    },
}

/// Result alias containing our Error type.
pub(crate) type Result<T> = std::result::Result<T, Error>;
