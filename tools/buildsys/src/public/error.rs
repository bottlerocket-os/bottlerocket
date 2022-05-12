use crate::ReadmeSource;
use snafu::Snafu;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(super)))]
pub enum Error {
    #[snafu(display("Unable to create the 'README.md' file: {}", source))]
    ReadmeCreate { source: std::io::Error },

    #[snafu(display("Unable to generate the 'README.md' file contents: {}", error))]
    ReadmeGenerate { error: String },

    #[snafu(display("Unable to open '{}': {}", file, source))]
    ReadmeSourceOpen {
        file: ReadmeSource,
        source: std::io::Error,
    },

    #[snafu(display("Unable to open 'README.tpl': {}", source))]
    ReadmeTemplateOpen { source: std::io::Error },

    #[snafu(display("Unable to write to the 'README.md' file: {}", source))]
    ReadmeWrite { source: std::io::Error },

    #[snafu(display(
        "The 'VARIANT' environment variable is missing or unable to be read: {}",
        source
    ))]
    VariantEnv { source: std::env::VarError },

    #[snafu(display("The '{}' segment of the variant '{}' is missing", part_name, variant))]
    VariantPart { part_name: String, variant: String },

    #[snafu(display("The '{}' segment of the variant '{}' is empty", part_name, variant))]
    VariantPartEmpty { part_name: String, variant: String },
}
