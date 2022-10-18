use snafu::Snafu;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(super)))]
pub(super) enum Error {
    #[snafu(display("Failed to read manifest file '{}': {}", path.display(), source))]
    ManifestFileRead { path: PathBuf, source: io::Error },

    #[snafu(display("Failed to load manifest file '{}': {}", path.display(), source))]
    ManifestFileLoad {
        path: PathBuf,
        source: toml::de::Error,
    },

    #[snafu(display("Failed to parse image feature '{}'", what))]
    ParseImageFeature { what: String },

    #[snafu(display("Invalid image size {}; must be between 1 and 1024", value))]
    InvalidImageSize { value: i32 },
}
