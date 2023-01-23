use crate::status::{UpdateCommand, UpdateState};
use http::StatusCode;
use num_derive::{FromPrimitive, ToPrimitive};
use snafu::Snafu;
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Failed to create tempfile for writing status: {}", source))]
    CreateTempfile { source: std::io::Error },

    #[snafu(display("Failed to create update status file '{}': {}", path.display(), source))]
    CreateStatusFile {
        path: PathBuf,
        source: tempfile::PathPersistError,
    },

    #[snafu(display("Failed to access update status file '{}': {}", path.display(), source))]
    NoStatusFile {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("Failed to parse update status file '{}': {}", path.display(), source))]
    StatusParse {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[snafu(display("Failed to write update status file '{}': {}", path.display(), source))]
    StatusWrite {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[snafu(display("Failed to deserialize update info: {}", source))]
    UpdateInfo { source: serde_json::Error },

    #[snafu(display("Unable to get OS version: {}", source))]
    ReleaseVersion { source: bottlerocket_release::Error },

    #[snafu(display("Failed to access lockfile '{}': {}",  path.display(),source))]
    UpdateLockFile {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("Unable to obtain lock on lockfile '{}': {}", path.display(), source))]
    UpdateLockHeld {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("Error sending {} to {}: {}", method, uri, source))]
    APIRequest {
        method: String,
        uri: String,
        #[snafu(source(from(apiclient::Error, Box::new)))]
        source: Box<apiclient::Error>,
    },

    #[snafu(display("Error {} when sending {} to {}: {}", code, method, uri, response_body))]
    APIResponse {
        method: String,
        uri: String,
        code: StatusCode,
        response_body: String,
    },

    #[snafu(display("Error deserializing response as JSON from {}: {}", uri, source))]
    ResponseJson {
        uri: String,
        source: serde_json::Error,
    },

    #[snafu(display("Failed to read OS disk partition table: {}", source))]
    PartitionTableRead {
        // signpost::Error triggers clippy::large_enum_variant
        #[snafu(source(from(signpost::Error, Box::new)))]
        source: Box<signpost::Error>,
    },

    #[snafu(display("Failed to modify OS disk partition table: {}", source))]
    PartitionTableWrite {
        // signpost::Error triggers clippy::large_enum_variant
        #[snafu(source(from(signpost::Error, Box::new)))]
        source: Box<signpost::Error>,
    },

    #[snafu(display("Could not mark inactive partition for boot: {}", source))]
    InactivePartitionUpgrade { source: signpost::Error },

    #[snafu(display("No partition set is set to boot next"))]
    NoneSetToBoot,

    #[snafu(display("Failed to fork process"))]
    Fork {},

    #[snafu(display("Failed to start updog: {}", source))]
    Updog { source: std::io::Error },

    #[snafu(display("Failed to prepare the update with updog"))]
    PrepareUpdate,

    #[snafu(display("Failed to activate the update with updog"))]
    ActivateUpdate,

    #[snafu(display("Failed to deactivate the update with updog"))]
    DeactivateUpdate,

    #[snafu(display("Failed to start signpost: {}", source))]
    Signpost { source: std::io::Error },

    #[snafu(display("Failed to get setting '{}': {}", setting, source))]
    GetSetting {
        setting: String,
        source: serde_json::Error,
    },

    #[snafu(display("Failed to parse version string '{}' into semver version", version))]
    SemVer {
        version: String,
        source: semver::Error,
    },

    #[snafu(display("Invalid state transition from {:?} to {:?}", from, to))]
    InvalidStateTransition { from: UpdateState, to: UpdateState },

    #[snafu(display("Command '{:?}' not allowed when state is '{:?}'", command, state))]
    DisallowCommand {
        command: UpdateCommand,
        state: UpdateState,
    },

    #[snafu(display("Chosen update does not exist"))]
    UpdateDoesNotExist,

    #[snafu(display("Update version to query is not specified"))]
    UnspecifiedVersion,

    #[snafu(display("No update image applied to the inactive partition set"))]
    StagingPartition,

    #[snafu(display("No image information for the active partition set"))]
    ActivePartition,

    #[snafu(display("Logger setup error: {}", source))]
    Logger { source: log::SetLoggerError },

    #[snafu(display("Unable to create a tokio runtime: {}", source))]
    Runtime { source: std::io::Error },
}

/// Map errors to specific exit codes to return to caller
#[derive(FromPrimitive, ToPrimitive)]
pub enum TbuErrorStatus {
    OtherError = 1,
    UpdateLockHeld = 64,
    DisallowCommand = 65,
    UpdateDoesNotExist = 66,
    NoStagedImage = 67,
}
