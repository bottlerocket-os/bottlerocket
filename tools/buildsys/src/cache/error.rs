use snafu::Snafu;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(super)))]
#[allow(clippy::enum_variant_names)]
pub(crate) enum Error {
    #[snafu(display("Missing environment variable '{}'", var))]
    Environment {
        var: String,
        source: std::env::VarError,
    },

    #[snafu(display("Bad file name '{}'", path.display()))]
    ExternalFileName { path: PathBuf },

    #[snafu(display("Bad file url '{}': {}", url, source))]
    ExternalFileUrl {
        url: String,
        source: url::ParseError,
    },

    #[snafu(display("Failed to request '{}': {}", url, source))]
    ExternalFileRequest { url: String, source: reqwest::Error },

    #[snafu(display("Failed to fetch '{}': {}", url, status))]
    ExternalFileFetch {
        url: String,
        status: reqwest::StatusCode,
    },

    #[snafu(display("Failed to open file '{}': {}", path.display(), source))]
    ExternalFileOpen { path: PathBuf, source: io::Error },

    #[snafu(display("Failed to write file '{}': {}", path.display(), source))]
    ExternalFileSave {
        path: PathBuf,
        source: reqwest::Error,
    },

    #[snafu(display("Failed to load file '{}': {}", path.display(), source))]
    ExternalFileLoad { path: PathBuf, source: io::Error },

    #[snafu(display("Failed to verify file '{}' with hash '{}'", path.display(), hash))]
    ExternalFileVerify { path: PathBuf, hash: String },

    #[snafu(display("Failed to rename file '{}': {}", path.display(), source))]
    ExternalFileRename { path: PathBuf, source: io::Error },

    #[snafu(display("Failed to delete file '{}': {}", path.display(), source))]
    ExternalFileDelete { path: PathBuf, source: io::Error },
}

pub(super) type Result<T> = std::result::Result<T, Error>;
