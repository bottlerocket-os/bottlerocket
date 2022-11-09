use crate::error;
use signpost::Error as SignpostError;
use snafu::Snafu;
use std::path::PathBuf;
use std::process::{Command, Output};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(super)))]
pub(super) enum Error {
    #[snafu(display("'{}' failed - stderr: {}",
                        bin_path, String::from_utf8_lossy(&output.stderr)))]
    CommandFailure { bin_path: String, output: Output },

    #[snafu(display("Failed to execute '{:?}': {}", command, source))]
    ExecutionFailure {
        command: Command,
        source: std::io::Error,
    },

    #[snafu(display("Kexec load syscalls are disabled, please make sure the value of `kernel.kexec_load_disabled` is 0"))]
    KexecLoadDisabled,

    #[snafu(display("Failed to load partitions state: {}", source))]
    LoadState { source: SignpostError },

    #[snafu(display("Failed to setup logger: {}", source))]
    Logger { source: log::SetLoggerError },

    #[snafu(display("Invalid log level '{}'", log_level))]
    LogLevel {
        log_level: String,
        source: log::ParseLevelError,
    },

    #[snafu(display("Failed to create mount '{}': '{}'", path, source))]
    Mount { path: String, source: nix::Error },

    #[snafu(display("Failed to delete file '{}': '{}'", path, source))]
    RemoveFile {
        path: String,
        source: std::io::Error,
    },

    #[snafu(display("Failed to read from file '{}': {}", path.display(), source))]
    ReadFile {
        source: std::io::Error,
        path: PathBuf,
    },

    #[snafu(display("Failed to setup mount '{}': '{}'", path, source))]
    SetupMount { path: String, source: nix::Error },

    #[snafu(display("Failed to write to file '{}': {}", path.display(), source))]
    WriteFile {
        source: std::io::Error,
        path: PathBuf,
    },

    #[snafu(display("Failed to retrieve settings: {}", source))]
    RetrieveSettings { source: schnauzer::Error },

    #[snafu(display("Failed to convert usize to u32: {}", source))]
    UsizeToU32 { source: std::num::TryFromIntError },

    #[snafu(display("Encountered unsigned 32-bit integer overflow when calculating checksum for boot config file"))]
    AddU32Overflow,

    #[snafu(display("Failed to write initrd image file: {}", source))]
    WriteInitrd { source: std::io::Error },

    #[snafu(display("Error serializing `BootSettings` to JSON: {}", source))]
    OutputJson { source: serde_json::error::Error },

    #[snafu(display("Failed to deserialize `BootSettings` from JSON value: {}", source))]
    BootSettingsFromJsonValue { source: serde_json::error::Error },

    #[snafu(display(
        "Invalid boot config file, expected key-value, or key entries for each line"
    ))]
    InvalidBootConfig,

    #[snafu(display("Failed to parse boot config key: {}", source))]
    ParseBootConfigKey {
        source: model::modeled_types::error::Error,
    },

    #[snafu(display("Invalid boot config value '{}'. Boot config values may only contain ASCII printable characters except for delimiters such as ';', '\n', ',', '#', and '}}'", input))]
    InvalidBootConfigValue { input: String },

    #[snafu(display("Failed to parse boot config value: {}", source))]
    ParseBootConfigValue {
        source: model::modeled_types::error::Error,
    },

    #[snafu(display("Unsupported boot config key '{}'. `BootSettings` currently only supports boot configuration for 'kernel' and 'init'", key))]
    UnsupportedBootConfigKey { key: String },

    #[snafu(display(
        "`BootSettings` does not support `kernel` and `init` as parent keys unless the values are null"
    ))]
    ParentBootConfigKey,

    #[snafu(display(
        "Encountered unbalanced quotes when processing array elements in '{}'",
        input
    ))]
    UnbalancedQuotes { input: String },

    #[snafu(display("Expected an comma between array elements, encountered '{}'", input))]
    ExpectedArrayComma { input: String },
}

pub(crate) type Result<T> = std::result::Result<T, error::Error>;
