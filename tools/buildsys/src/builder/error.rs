use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(super)")]
pub enum Error {
    #[snafu(display("Failed to execute command: {}", source))]
    CommandExecution { source: std::io::Error },

    #[snafu(display("Failed to build package '{}':\n{}", package, output,))]
    PackageBuild { package: String, output: String },

    #[snafu(display("Failed to build image with '{}':\n{}", packages, output,))]
    ImageBuild { packages: String, output: String },

    #[snafu(display("Missing environment variable '{}'", var))]
    Environment {
        var: String,
        source: std::env::VarError,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
