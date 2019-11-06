#![allow(clippy::default_trait_access)]

use data_store_version::Version as DataVersion;
use snafu::{Backtrace, Snafu};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")]
pub enum Error {
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
    BadDataVersion { backtrace: Backtrace, key: String },

    #[snafu(display("Could not parse image version: {} - {}", key, value))]
    BadMapVersion {
        backtrace: Backtrace,
        key: String,
        value: String,
    },

    #[snafu(display("Duplicate key ID: {}", keyid))]
    DuplicateKeyId { backtrace: Backtrace, keyid: u64 },

    #[snafu(display("Duplicate version key: {}", key))]
    DuplicateVersionKey { backtrace: Backtrace, key: String },

    #[snafu(display("Failed to parse updates manifest: {}", source))]
    ManifestParse {
        source: serde_json::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Missing datastore version in metadata: {:?}", version))]
    MissingDataVersion {
        backtrace: Backtrace,
        version: DataVersion,
    },

    #[snafu(display("Image version missing datastore mapping: {}", version))]
    MissingMapping {
        backtrace: Backtrace,
        version: String,
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

    #[snafu(display("This host is not part of any wave"))]
    NoWave { backtrace: Backtrace },

    #[snafu(display("Failed to serialize update information: {}", source))]
    UpdateSerialize {
        source: serde_json::Error,
        backtrace: Backtrace,
    },
}
