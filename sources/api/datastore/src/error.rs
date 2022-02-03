use snafu::Snafu;
use std::io;
use std::path::PathBuf;

use super::{serialization, ScalarError};

/// Possible errors from datastore operations.
#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Error serializing {}: {} ", given, source))]
    Serialization {
        given: String,
        source: serialization::Error,
    },

    #[snafu(display("Error serializing scalar {}: {} ", given, source))]
    SerializeScalar { given: String, source: ScalarError },

    #[snafu(display("Key would traverse outside data store: {}", name))]
    PathTraversal { name: String },

    #[snafu(display("Reading key '{}' failed: {}", key, source))]
    KeyRead { key: String, source: io::Error },

    #[snafu(display("Removing key at '{}' failed: {}", path.display(), source))]
    DeleteKey { path: PathBuf, source: io::Error },

    #[snafu(display("IO error on '{}': {}", path.display(), source))]
    Io { path: PathBuf, source: io::Error },

    #[snafu(display("Can't handle non-Unicode file for {}: {}", context, file))]
    NonUnicodeFile { file: String, context: String },

    #[snafu(display("Data store logic error: {}", msg))]
    Internal { msg: String },

    #[snafu(display("Data store integrity violation at {}: {}", path.display(), msg))]
    Corruption { msg: String, path: PathBuf },

    #[snafu(display("Error building data store path: {}", source))]
    Path { source: std::path::StripPrefixError },

    #[snafu(display("Error listing datastore keys: {}", source))]
    ListKeys { source: walkdir::Error },

    #[snafu(display("Listed key '{}' not found on disk", key))]
    ListedKeyNotPresent { key: String },

    #[snafu(display(
        "Listed metadata '{}' for key '{}' not found on disk",
        meta_key,
        data_key
    ))]
    ListedMetaNotPresent { meta_key: String, data_key: String },

    #[snafu(display("Key name '{}' has invalid format: {}", name, msg))]
    InvalidKey { name: String, msg: String },

    #[snafu(display("Key name beyond maximum length {}: {}", name, max))]
    KeyTooLong { name: String, max: usize },
}

pub type Result<T> = std::result::Result<T, Error>;
