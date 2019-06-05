use snafu::Snafu;
use std::io;
use std::path::PathBuf;

use super::{serialization, ScalarError};

/// Possible errors from datastore operations.
#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum Error {
    #[snafu(display("defaults.toml is not valid TOML: {}", source))]
    DefaultsFormatting { source: toml::de::Error },

    #[snafu(display("defaults.toml is not a TOML table"))]
    DefaultsNotTable {},

    #[snafu(display("defaults.toml's metadata is not a TOML list of Metadata"))]
    DefaultsMetadataNotTable { source: toml::de::Error },

    #[snafu(display("Error serializing {}: {} ", given, source))]
    Serialization {
        given: String,
        source: serialization::Error,
    },

    #[snafu(display("Error serializing scalar {}: {} ", given, source))]
    SerializeScalar {
        given: String,
        source: ScalarError,
    },

    #[snafu(display("Key would traverse outside data store: {}", name))]
    PathTraversal { name: String },

    #[snafu(display("Reading key '{}' failed: {}", key, source))]
    KeyRead { key: String, source: io::Error },

    #[snafu(display("IO error on '{}': {}", path.display(), source))]
    Io { path: PathBuf, source: io::Error },

    #[snafu(display("Data store logic error: {}", msg))]
    Internal { msg: String },

    #[snafu(display("Data store integrity violation at {}: {}", path.display(), msg))]
    Corruption { msg: String, path: PathBuf },

    #[snafu(display("Error building data store path: {}", source))]
    Path { source: std::path::StripPrefixError },

    #[snafu(display("Error listing datastore keys: {}", source))]
    ListKeys { source: walkdir::Error },

    #[snafu(display("Listed key '{}' not found on disk", key))]
    ListedKeyNotPresent {
        key: String,
    },

    // Showing the full regex in an error is ugly because of ?x and the regex's formatting;
    // see datastore::key::{DATA_KEY,METADATA_KEY}
    #[snafu(display("Key name '{}' has invalid format, should be 1 or more dot-separated [a-zA-Z0-9_-]+", name))]
    InvalidKey { name: String },

    #[snafu(display("Key name beyond maximum length {}: {}", name, max))]
    KeyTooLong { name: String, max: usize },
}

pub type Result<T> = std::result::Result<T, Error>;
