use std::path::PathBuf;

use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(super)))]
pub(crate) enum Error {
    #[snafu(display("Variant {} could not be found.", variant))]
    VariantNotFound {
        source: std::io::Error,
        variant: String,
    },

    #[snafu(display("Repo root '{:?}' in invalid.", path))]
    InvalidRepoRoot {
        source: std::io::Error,
        path: PathBuf,
    },

    #[snafu(display("Package '{:?}' was not found.", package))]
    PackageNotFound { package: String },

    #[snafu(display("Output directory '{:?}' could not be created: {}", path, source))]
    InvalidOutputDir {
        source: std::io::Error,
        path: PathBuf,
    },

    #[snafu(display("Unable to write output to '{:?}': {}", path, source))]
    OutputFileWrite {
        source: std::io::Error,
        path: PathBuf,
    },

    #[snafu(display("Error reading '{:?}': {}", path, source))]
    CargoReadFailure {
        source: std::io::Error,
        path: PathBuf,
    },

    #[snafu(display("Error parsing '{:?}': {}", path, source))]
    CargoParseFailure {
        source: toml::de::Error,
        path: PathBuf,
    },
}
