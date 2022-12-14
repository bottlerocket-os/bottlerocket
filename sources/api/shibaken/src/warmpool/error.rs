use snafu::Snafu;
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, WarmPoolCheckError>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(super)))]
pub enum WarmPoolCheckError {
    #[snafu(display("Command '{}' with args '{:?}' failed: {}", command, args, source))]
    Command {
        command: String,
        args: Vec<String>,
        source: std::io::Error,
    },

    #[snafu(display("Failed to parse config file {}: {}", path.display(), source))]
    ConfigParse {
        path: PathBuf,
        source: toml::de::Error,
    },

    #[snafu(display("Failed to read config file {}: {}", path.display(), source))]
    ConfigRead {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("IMDS request failed: {}", source))]
    ImdsRequest { source: imdsclient::Error },

    #[snafu(display("Logger setup error: {}", source))]
    Logger { source: log::SetLoggerError },
}
