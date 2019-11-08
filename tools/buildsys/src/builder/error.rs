use snafu::Snafu;
use std::path::PathBuf;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(super)")]
pub enum Error {
    #[snafu(display("Failed to execute command: {}", source))]
    CommandExecution { source: std::io::Error },

    #[snafu(display("Failed to change directory to '{}': {}", path.display(), source))]
    DirectoryChange {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("Missing environment variable '{}'", var))]
    Environment {
        var: String,
        source: std::env::VarError,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
