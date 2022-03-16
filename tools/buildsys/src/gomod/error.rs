use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(super)))]
pub(crate) enum Error {
    #[snafu(display("Failed to start command: {}", source))]
    CommandStart { source: std::io::Error },

    #[snafu(display("Failed to execute command: 'docker {}'", args))]
    DockerExecution { args: String },

    #[snafu(display("input is required"))]
    InputFile,

    #[snafu(display("input must be a file and end with .tar.gz"))]
    InputFileBad,

    #[snafu(display("mod-dir is required"))]
    ModDir,

    #[snafu(display("output-dir is required"))]
    OutputDir,

    #[snafu(display("output_dir must not exist or be a directory"))]
    OutputDirBad,

    #[snafu(display("Missing environment variable '{}'", var))]
    Environment {
        var: String,
        source: std::env::VarError,
    },
}

pub(super) type Result<T> = std::result::Result<T, Error>;
