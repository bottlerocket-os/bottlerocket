use crate::datastore::{self, deserialization, serialization};
use nix::unistd::Gid;
use snafu::Snafu;
use std::io;
use std::path::PathBuf;

// We want server (router/handler) and controller errors together so it's easy to define response
// error codes for all the high-level types of errors that could happen during a request.
#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(super)")]
pub enum Error {
    // Systemd Notification errors
    #[snafu(display("Systemd notify error: {}", source))]
    SystemdNotify { source: std::io::Error },

    #[snafu(display("Failed to send systemd status notification"))]
    SystemdNotifyStatus,

    // =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

    // Set file metadata errors
    #[snafu(display(
        "Failed to set file permissions on the API socket to {:o}: {}",
        mode,
        source
    ))]
    SetPermissions { source: std::io::Error, mode: u32 },

    #[snafu(display("Failed to set group owner on the API socket to {}: {}", gid, source))]
    SetGroup { source: nix::Error, gid: Gid },

    // =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

    // Server errors
    #[snafu(display("Missing required input '{}'", input))]
    MissingInput { input: String },

    #[snafu(display("Input '{}' cannot be empty", input))]
    EmptyInput { input: String },

    #[snafu(display("Another thread poisoned the data store lock by panicking"))]
    DataStoreLock,

    #[snafu(display("Unable to serialize response: {}", source))]
    ResponseSerialization { source: serde_json::Error },

    #[snafu(display("Unable to bind to {}: {}", path.display(), source))]
    BindSocket { path: PathBuf, source: io::Error },

    #[snafu(display("Unable to start server: {}", source))]
    ServerStart { source: io::Error },

    #[snafu(display("Tried to commit with no pending changes"))]
    CommitWithNoPending,

    // =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

    // Controller errors
    #[snafu(display("Found no '{}' in datastore", prefix))]
    MissingData { prefix: String },

    #[snafu(display("Found no '{}' in datastore", requested))]
    ListKeys { requested: String },

    #[snafu(display("Listed key '{}' not found on disk", key))]
    ListedKeyNotPresent { key: String },

    #[snafu(display("Data store error during {}: {}", op, source))]
    DataStore {
        op: String,
        source: datastore::Error,
    },

    #[snafu(display("Error deserializing {}: {} ", given, source))]
    Deserialization {
        given: String,
        source: deserialization::Error,
    },

    #[snafu(display("Error serializing {}: {} ", given, source))]
    DataStoreSerialization {
        given: String,
        source: serialization::Error,
    },

    #[snafu(display("Error serializing {}: {} ", given, source))]
    CommandSerialization {
        given: String,
        source: serde_json::Error,
    },

    #[snafu(display("Unable to make {} key '{}': {}", key_type, name, source))]
    NewKey {
        key_type: String,
        name: String,
        source: datastore::Error,
    },

    #[snafu(display("Metadata '{}' is not valid JSON: {}", key, source))]
    InvalidMetadata {
        key: String,
        source: serde_json::Error,
    },

    #[snafu(display("Unable to start config applier: {} ", source))]
    ConfigApplierStart { source: io::Error },

    #[snafu(display("Unable to use config applier, couldn't get stdin"))]
    ConfigApplierStdin {},

    #[snafu(display("Unable to send input to config applier: {}", source))]
    ConfigApplierWrite { source: io::Error },
}

pub type Result<T> = std::result::Result<T, Error>;
