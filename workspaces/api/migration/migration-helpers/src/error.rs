//! Contains the Error and Result types used by the migration helper functions and migrations.

use snafu::Snafu;

use apiserver::datastore;

/// Error contains the errors that can happen in the migration helper functions and in migrations.
#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum Error {
    #[snafu(display("Unable to get {:?} data for migration: {}", committed, source))]
    GetData {
        committed: datastore::Committed,
        source: datastore::Error,
    },

    #[snafu(display("Unable to get metadata for migration: {}", source))]
    GetMetadata { source: datastore::Error },

    #[snafu(display("Unable to deserialize to Value from '{}': {}", input, source))]
    Deserialize {
        input: String,
        source: datastore::ScalarError,
    },

    #[snafu(display("Unable to serialize Value: {}", source))]
    Serialize { source: datastore::ScalarError },

    #[snafu(display("Unable to write to data store: {}", source))]
    DataStoreWrite { source: datastore::Error },

    #[snafu(display("Migrated data failed validation: {}", msg))]
    Validation { msg: String },

    // Generic error variant for migration authors
    #[snafu(display("Migration returned error: {}", msg))]
    Migration { msg: String },

    // More specific error variants for migration authors to handle common cases
    #[snafu(display("Migration requires missing key: {}", key))]
    MissingData { key: String },

    #[snafu(display("Migration used invalid key '{}': {}", key, source))]
    InvalidKey {
        key: String,
        source: datastore::Error,
    },
}

/// Result alias containing our Error type.
pub type Result<T> = std::result::Result<T, Error>;
