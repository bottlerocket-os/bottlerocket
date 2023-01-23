//! Contains the Error and Result types used by the migration helper functions and migrations.

use snafu::Snafu;
use std::path::PathBuf;

/// Error contains the errors that can happen in the migration helper functions and in migrations.
#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Unable to get system release data: {}", source))]
    BottlerocketRelease { source: bottlerocket_release::Error },

    #[snafu(display("Unable to get {:?} data for migration: {}", committed, source))]
    GetData {
        committed: datastore::Committed,
        #[snafu(source(from(datastore::Error, Box::new)))]
        source: Box<datastore::Error>,
    },

    #[snafu(display("Unable to get metadata for migration: {}", source))]
    GetMetadata {
        #[snafu(source(from(datastore::Error, Box::new)))]
        source: Box<datastore::Error>,
    },

    #[snafu(display("Unable to deserialize to Value from '{}': {}", input, source))]
    Deserialize {
        input: String,
        source: datastore::ScalarError,
    },

    #[snafu(display("Unable to serialize Value: {}", source))]
    Serialize { source: datastore::ScalarError },

    #[snafu(display("Unable to serialize release data: {}", source))]
    SerializeRelease {
        source: datastore::serialization::Error,
    },

    #[snafu(display("Unable to write to data store: {}", source))]
    DataStoreWrite {
        #[snafu(source(from(datastore::Error, Box::new)))]
        source: Box<datastore::Error>,
    },

    #[snafu(display("Unable to remove key '{}' from data store: {}", key, source))]
    DataStoreRemove {
        key: String,
        #[snafu(source(from(datastore::Error, Box::new)))]
        source: Box<datastore::Error>,
    },

    #[snafu(display("Migrated data failed validation: {}", msg))]
    Validation { msg: String },

    // Generic error variant for migration authors
    #[snafu(display("Migration returned error: {}", msg))]
    Migration { msg: String },

    // More specific error variants for migration authors to handle common cases
    #[snafu(display("Migration requires missing key: {}", key))]
    MissingData { key: String },

    #[snafu(display("Migration used invalid {:?} key '{}': {}", key_type, key, source))]
    InvalidKey {
        key_type: datastore::KeyType,
        key: String,
        #[snafu(source(from(datastore::Error, Box::new)))]
        source: Box<datastore::Error>,
    },

    #[snafu(display("Unable to list transactions in data store: {}", source))]
    ListTransactions {
        #[snafu(source(from(datastore::Error, Box::new)))]
        source: Box<datastore::Error>,
    },

    #[snafu(display("Unable to build handlebar template registry: {}", source))]
    BuildTemplateRegistry { source: schnauzer::error::Error },

    #[snafu(display("Unable to render template string '{}': {}", template, source))]
    RenderTemplate {
        template: String,
        #[snafu(source(from(handlebars::RenderError, Box::new)))]
        source: Box<handlebars::RenderError>,
    },

    #[snafu(display("'{}' is set to non-string value", setting))]
    NonStringSettingDataType { setting: String },

    #[snafu(display("Unable to deserialize datastore data: {}", source))]
    DeserializeDatastore {
        source: datastore::deserialization::Error,
    },

    #[snafu(display("Unable to create new key: {}", source))]
    NewKey { source: datastore::error::Error },

    #[snafu(display("Setting '{}' contains non-string item: {:?}", setting, data))]
    ReplaceListContents {
        setting: String,
        data: Vec<serde_json::Value>,
    },

    #[snafu(display(
        "Metadata '{}' for setting '{}' contains non-string item: {:?}",
        metadata,
        setting,
        data
    ))]
    ReplaceMetadataListContents {
        setting: String,
        metadata: String,
        data: Vec<serde_json::Value>,
    },

    #[snafu(display("Failed to delete file '{}': '{}'", path.display(), source))]
    RemoveFile {
        path: PathBuf,
        source: std::io::Error,
    },
}

/// Result alias containing our Error type.
pub type Result<T> = std::result::Result<T, Error>;
