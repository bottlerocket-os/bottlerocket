use snafu::Snafu;
use std::process::ExitCode;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
#[repr(i32)]
pub enum Error {
    #[snafu(display("Failed to run '{}' successfully {}", command, source))]
    CommandFailure {
        command: String,
        source: std::io::Error,
    },

    #[snafu(display("Unable to find the mount point for device: {}", target))]
    FindMount { target: String },

    #[snafu(display("Failed to parse '{}' output: {}", command, source))]
    FromUtf8 {
        command: String,
        source: std::string::FromUtf8Error,
    },

    #[snafu(display("{} filesystem is mounted", mount_point))]
    MountedFilesystem { mount_point: String },

    #[snafu(display("Failed to read '{}' status code", command))]
    ParseStatusCode { command: String },

    #[snafu(display("Could not parse target block device",))]
    ParseTarget,
}

impl Error {
    pub fn exit_code(&self) -> ExitCode {
        match self {
            Error::CommandFailure { .. } => ExitCode::from(8),
            Error::FindMount { .. } => ExitCode::from(8),
            Error::FromUtf8 { .. } => ExitCode::from(8),
            Error::MountedFilesystem { .. } => ExitCode::from(2),
            Error::ParseStatusCode { .. } => ExitCode::from(8),
            Error::ParseTarget => ExitCode::from(2),
        }
    }
}
