#![allow(clippy::default_trait_access)]

use data_store_version::Version as DataVersion;
use snafu::{Backtrace, Snafu};
use std::path::PathBuf;

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")]
pub(crate) enum Error {
    #[snafu(display("Bad bound field: {}", bound_str))]
    BadBound {
        backtrace: Backtrace,
        source: std::num::ParseIntError,
        bound_str: String,
    },

    #[snafu(display("Invalid bound start: {}", key))]
    BadBoundKey {
        source: std::num::ParseIntError,
        key: String,
        backtrace: Backtrace,
    },

    #[snafu(display("Could not parse datastore version: {}", key))]
    BadDataVersion {
        backtrace: Backtrace,
        key: String,
        source: data_store_version::error::Error,
    },

    #[snafu(display("Could not parse image version: {} - {}", key, value))]
    BadMapVersion {
        backtrace: Backtrace,
        key: String,
        value: String,
    },

    #[snafu(display("Migration {} matches regex but missing version", name))]
    BadRegexVersion { name: String },

    #[snafu(display("Migration {} matches regex but missing name", name))]
    BadRegexName { name: String },

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

    #[snafu(display("Failed to write config file {}: {}", path.display(), source))]
    ConfigWrite {
        path: PathBuf,
        source: std::io::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to create metadata cache directory: {}", source))]
    CreateMetadataCache {
        source: std::io::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to create directory: {:?}", path))]
    DirCreate {
        backtrace: Backtrace,
        source: std::io::Error,
        path: PathBuf,
    },

    #[snafu(display("Duplicate key ID: {}", keyid))]
    DuplicateKeyId { backtrace: Backtrace, keyid: u32 },

    #[snafu(display("Duplicate version key: {}", key))]
    DuplicateVersionKey { backtrace: Backtrace, key: String },

    #[snafu(display("Migration '{}' contains invalid version: {}", name, source))]
    InvalidMigrationVersion {
        name: String,
        source: data_store_version::error::Error,
    },

    #[snafu(display("Logger setup error: {}", source))]
    Logger { source: simplelog::TermLogError },

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

    #[snafu(display("Failed to read manifest file {}: {}", path.display(), source))]
    ManifestRead {
        path: PathBuf,
        source: std::io::Error,
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

    #[snafu(display(
        "Migration {} given for {} but name implies it is for {}",
        name,
        to,
        version
    ))]
    MigrationInvalidTarget {
        backtrace: Backtrace,
        name: String,
        to: DataVersion,
        version: DataVersion,
    },

    #[snafu(display(
        "Migration name invalid; must follow format 'migrate_${{TO_VERSION}}_${{NAME}}'"
    ))]
    MigrationNaming { backtrace: Backtrace },

    #[snafu(display("Migration not found in image: {:?}", name))]
    MigrationNotLocal { backtrace: Backtrace, name: PathBuf },

    #[snafu(display("Unable to get mutable reference to ({},{}) migrations", from, to))]
    MigrationMutable {
        backtrace: Backtrace,
        from: DataVersion,
        to: DataVersion,
    },

    #[snafu(display("Migration ({},{}) not present in manifest", from, to))]
    MigrationNotPresent {
        backtrace: Backtrace,
        from: DataVersion,
        to: DataVersion,
    },

    #[snafu(display("Missing datastore version in metadata: {:?}", version))]
    MissingDataVersion {
        backtrace: Backtrace,
        version: DataVersion,
    },

    #[snafu(display(
        "Reached end of migration chain at {} but target is {}",
        current,
        target
    ))]
    MissingMigration {
        backtrace: Backtrace,
        current: DataVersion,
        target: DataVersion,
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

    #[snafu(display("Failed to determine VERSION_ID from /etc/os-release"))]
    VersionIdNotFound { backtrace: Backtrace },

    #[snafu(display("Failed to parse VERSION_ID from /etc/os-release: {}", line))]
    VersionIdParse {
        source: semver::SemVerError,
        backtrace: Backtrace,
        line: String,
    },

    #[snafu(display("Failed to read /etc/os-release: {}", source))]
    VersionIdRead {
        source: std::io::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("--start-time <time> required to add wave to update"))]
    WaveStartArg { backtrace: Backtrace },

    #[snafu(display("Waves are not ordered: bound {} occurs before bound {}", next, wave))]
    WavesUnordered { wave: u32, next: u32 },

    #[snafu(display("Failed writing update data to disk: {}", source))]
    WriteUpdate {
        source: std::io::Error,
        backtrace: Backtrace,
    },
}
