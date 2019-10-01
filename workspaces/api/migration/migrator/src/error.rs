//! This module owns the error type used by the migrator.

use data_store_version::error::Error as VersionError;
use data_store_version::{Version, VersionComponent};
use snafu::Snafu;
use std::io;
use std::path::PathBuf;
use std::process::{Command, Output};

/// Error contains the errors that can happen during migration.
#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")]
pub(crate) enum Error {
    #[snafu(display("Tracing setup error: {}", source))]
    Logger { source: tracing::dispatcher::SetGlobalDefaultError },
    
    #[snafu(display("Failed to parse provided directive: {}", source))]
    TracingDirectiveParse {
        source: tracing_subscriber::filter::LevelParseError,
    },

    #[snafu(display("Internal error: {}", msg))]
    Internal { msg: String },

    #[snafu(display("Unable to create version from data store path '{}': {}", path.display(), source))]
    VersionFromDataStorePath { path: PathBuf, source: VersionError },

    #[snafu(display("Can only migrate minor versions; major version '{}' requested, major version of given data store is '{}'", given, found))]
    MajorVersionMismatch {
        given: VersionComponent,
        found: VersionComponent,
    },

    #[snafu(display("Unable to open data store directory '{}': {}", path.display(), source))]
    DataStoreDirOpen { path: PathBuf, source: nix::Error },

    #[snafu(display("Data store link '{}' points to /", path.display()))]
    DataStoreLinkToRoot { path: PathBuf },

    #[snafu(display("Data store path '{}' contains invalid version: {}", path.display(), source))]
    InvalidDataStoreVersion {
        path: PathBuf,
        #[snafu(source(from(Error, Box::new)))]
        source: Box<Error>,
    },

    #[snafu(display("Migration '{}' contains invalid version: {}", path.display(), source))]
    InvalidMigrationVersion {
        path: PathBuf,
        #[snafu(source(from(VersionError, Box::new)))]
        source: Box<VersionError>,
    },

    #[snafu(display("Data store for new version {} already exists at {}", version, path.display()))]
    NewVersionAlreadyExists { version: Version, path: PathBuf },

    #[snafu(display("Failed copying data store to work location: {}", source))]
    DataStoreCopy { source: fs_extra::error::Error },

    #[snafu(display("Unable to start migration command {:?} - {}", command, source))]
    StartMigration { command: Command, source: io::Error },

    #[snafu(display("Migration returned '{}' - stderr: {}",
                    output.status.code()
                        .map(|i| i.to_string()).unwrap_or_else(|| "signal".to_string()),
                    std::str::from_utf8(&output.stderr)
                        .unwrap_or_else(|_e| "<invalid UTF-8>")))]
    MigrationFailure { output: Output },

    #[snafu(display("Failed to create symlink for new version at {}: {}", path.display(), source))]
    LinkCreate { path: PathBuf, source: io::Error },

    #[snafu(display("Failed to swap symlink at {} to new version: {}", link.display(), source))]
    LinkSwap { link: PathBuf, source: io::Error },

    #[snafu(display("Failed listing migration directory '{}': {}", dir.display(), source))]
    ListMigrations { dir: PathBuf, source: io::Error },

    #[snafu(display("Failed reading migration directory entry: {}", source))]
    ReadMigrationEntry { source: io::Error },

    #[snafu(display("Failed reading metadata of '{}': {}", path.display(), source))]
    PathMetadata { path: PathBuf, source: io::Error },

    #[snafu(display("Migration path '{}' contains invalid UTF-8", path.display()))]
    MigrationNameNotUTF8 { path: PathBuf },
}

/// Result alias containing our Error type.
pub(crate) type Result<T> = std::result::Result<T, Error>;
