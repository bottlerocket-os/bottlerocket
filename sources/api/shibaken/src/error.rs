use crate::warmpool::error::WarmPoolCheckError;
use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(super)))]
pub(crate) enum Error {
    #[snafu(display("IMDS request failed: {}", source))]
    ImdsRequest { source: imdsclient::Error },

    #[snafu(display("IMDS client failed: {}", source))]
    ImdsClient { source: imdsclient::Error },

    #[snafu(display(
        "IMDS client failed: Response '404' while fetching '{}' from '{}'",
        target,
        target_type,
    ))]
    ImdsData { target: String, target_type: String },

    #[snafu(display("Logger setup error: {}", source))]
    Logger { source: log::SetLoggerError },

    #[snafu(display("Invalid log level '{}'", log_level_str))]
    LogLevel {
        log_level_str: String,
        source: log::ParseLevelError,
    },

    #[snafu(display("Error serializing to JSON: {}", source))]
    SerializeJson { source: serde_json::error::Error },

    #[snafu(display("Failed to check autoscaling warm pool: {}", source))]
    WarmPoolCheckFailed { source: WarmPoolCheckError },
}

pub(crate) type Result<T> = std::result::Result<T, Error>;
