#![allow(clippy::default_trait_access)]

use semver::Version;
use snafu::{Backtrace, Snafu};
use std::path::PathBuf;
use update_metadata::error::Error as update_metadata_error;

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")]
pub(crate) enum Error {
    #[snafu(display(
        "Failed to convert '{}' from FriendlyVersion to semver::Version: {}",
        version_str,
        source
    ))]
    BadVersion {
        version_str: String,
        source: semver::SemVerError,
    },

    #[snafu(display("Bad version string '{}' in config: {}", version_str, source))]
    BadVersionConfig {
        version_str: String,
        source: model::modeled_types::error::Error,
    },

    #[snafu(display("Failed to parse config file {}: {}", path.display(), source))]
    ConfigParse {
        path: PathBuf,
        source: toml::de::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to read config file {}: {}", path.display(), source))]
    ConfigRead {
        path: PathBuf,
        source: std::io::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to serialize config file {}: {}", path.display(), source))]
    ConfigSerialize {
        path: PathBuf,
        source: toml::ser::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to create metadata cache directory '{}': {}", path, source))]
    CreateMetadataCache {
        path: &'static str,
        source: std::io::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to create a tempdir for tough datastore: {}", source))]
    CreateTempDir {
        source: std::io::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to create directory: {:?}", path))]
    DirCreate {
        backtrace: Backtrace,
        source: std::io::Error,
        path: PathBuf,
    },

    #[snafu(display("Logger setup error: {}", source))]
    Logger { source: log::SetLoggerError },

    #[snafu(display("Could not mark inactive partition for boot: {}", source))]
    InactivePartitionUpgrade { source: signpost::Error },

    #[snafu(display("Failed to attach image to loop device"))]
    LoopAttachFailed {
        backtrace: Backtrace,
        source: std::io::Error,
    },

    #[snafu(display("Failed to open loop device control"))]
    LoopControlFailed {
        backtrace: Backtrace,
        source: std::io::Error,
    },

    #[snafu(display("Failed to find free loop device"))]
    LoopFindFailed {
        backtrace: Backtrace,
        source: std::io::Error,
    },

    #[snafu(display("Could not determine loop device path"))]
    LoopNameFailed { backtrace: Backtrace },

    #[snafu(display("Failed to decode LZ4-compressed target {}: {}", target, source))]
    Lz4Decode {
        target: String,
        source: std::io::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to parse updates manifest: {}", source))]
    ManifestParse {
        source: serde_json::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Metadata error: {}", source))]
    Metadata {
        source: tough::error::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to copy migration from image: {}", name))]
    MigrationCopyFailed {
        backtrace: Backtrace,
        source: std::io::Error,
        name: String,
    },

    #[snafu(display("Migration not found in image: {:?}", name))]
    MigrationNotLocal { backtrace: Backtrace, name: PathBuf },

    #[snafu(display("Migration ({},{}) not present in manifest", from, to))]
    MigrationNotPresent {
        backtrace: Backtrace,
        from: Version,
        to: Version,
    },

    #[snafu(display("Missing version in metadata: {}", version))]
    MissingVersion {
        backtrace: Backtrace,
        version: String,
    },

    #[snafu(display("Temporary image mount failed"))]
    MountFailed {
        backtrace: Backtrace,
        source: std::io::Error,
    },

    #[snafu(display("No update available"))]
    NoUpdate { backtrace: Backtrace },

    #[snafu(display("Failed to open partition {}: {}", path.display(), source))]
    OpenPartition {
        path: PathBuf,
        source: std::io::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to open trusted root metadata file {}: {}", path.display(), source))]
    OpenRoot {
        path: PathBuf,
        source: std::io::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to read OS disk partition table: {}", source))]
    PartitionTableRead {
        // signpost::Error triggers clippy::large_enum_variant
        #[snafu(source(from(signpost::Error, Box::new)))]
        source: Box<signpost::Error>,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to modify OS disk partition table: {}", source))]
    PartitionTableWrite {
        // signpost::Error triggers clippy::large_enum_variant
        #[snafu(source(from(signpost::Error, Box::new)))]
        source: Box<signpost::Error>,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to reboot: {}", source))]
    RebootFailure {
        source: std::io::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to parse release metadata file '{}': {}", path.display(), source))]
    ReleaseParse {
        path: PathBuf,
        source: toml::de::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Unable to get OS version: {}", source))]
    ReleaseVersion { source: bottlerocket_release::Error },

    #[snafu(display("Failed setting permissions of '{}': {}", path.display(), source))]
    SetPermissions {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("Target not found: {}", target))]
    TargetNotFound {
        target: String,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to create tmpfile for root mount"))]
    TmpFileCreate {
        backtrace: Backtrace,
        source: std::io::Error,
    },

    #[snafu(display("2Borrow2Fast"))]
    TransportBorrow {
        backtrace: Backtrace,
        source: std::cell::BorrowMutError,
    },

    #[snafu(display("No update available"))]
    UpdateNotAvailable { backtrace: Backtrace },

    #[snafu(display("Update {} exists but wave in the future", version))]
    UpdateNotReady {
        backtrace: Backtrace,
        version: Version,
    },

    #[snafu(display("Failed to serialize update information: {}", source))]
    UpdateSerialize {
        source: serde_json::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Update in the incorrect state"))]
    UpdateState { backtrace: Backtrace },

    #[snafu(display("Target partition is unrecognized: {}", partition))]
    UnknownPartition {
        partition: String,
        backtrace: Backtrace,
    },

    #[snafu(display("--wave-file <path> required to add waves to update"))]
    WaveFileArg { backtrace: Backtrace },

    #[snafu(display("Failed writing update data to disk: {}", source))]
    WriteUpdate {
        source: std::io::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("{}", source))]
    UpdateMetadata {
        source: update_metadata::error::Error,
    },

    #[snafu(display("Failed to set up signal handler: {}", source))]
    Signal {
        source: std::io::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to store manifest and migrations: {}", source))]
    RepoCacheMigrations { source: tough::error::Error },
}

impl std::convert::From<update_metadata::error::Error> for Error {
    fn from(e: update_metadata_error) -> Self {
        Error::UpdateMetadata { source: e }
    }
}
