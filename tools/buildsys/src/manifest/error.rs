use snafu::Snafu;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum Error {
    #[snafu(display("Failed to read manifest file '{}': {}", path.display(), source))]
    ManifestFileRead { path: PathBuf, source: io::Error },

    #[snafu(display("Failed to load manifest file '{}': {}", path.display(), source))]
    ManifestFileLoad {
        path: PathBuf,
        source: toml::de::Error,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
