use snafu::Snafu;
use std::process::ExitCode;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
#[repr(i32)]
pub enum Error {
    // Errors to map the errors defined in
    // https://git.kernel.org/pub/scm/fs/xfs/xfsprogs-dev.git/tree/fsck/xfs_fsck.sh
    #[snafu(display("xfs {} errror: {} was not found!", cli, command))]
    CommandUnavailable { cli: String, command: String },

    #[snafu(display("{} does not exist", target))]
    DeviceNotFound { target: String },

    #[snafu(display(
        "xfs fsck errror: The filesystem log is dirty, mount it to recover \
        the log. If that fails, refer to the section DIRTY LOGS in the \
        xfs_repair manual page."
    ))]
    DirtyLogs,

    #[snafu(display("Unable to create tempdir: {}", source))]
    MakeDir { source: std::io::Error },

    #[snafu(display("Unable to mount the directory to device"))]
    Mount,

    #[snafu(display("xfs fsck error: xfs_repair could not fix the filesystem."))]
    RepairFailure,

    #[snafu(display("xfs fsck errror: An unknown return code from xfs_repair {}", code))]
    UnrecognizedExitCode { code: i32 },

    // Diverted behaviour from script, we fail as error code 8
    // as we do not have etc/fstab file so device is mandatory argument
    #[snafu(display("Could not parse target block device",))]
    ParseTarget,

    // General errors
    #[snafu(display("Failed to run '{}' successfully {}", command, source))]
    CommandFailure {
        command: String,
        source: std::io::Error,
    },

    #[snafu(display(
        "Could not delete the temporary mount dir created to repair xfs filesystem {}",
        source
    ))]
    DeleteMountDirectory { source: std::io::Error },

    #[snafu(display("Unable to find {} base name", target))]
    FindBasename { target: String },

    #[snafu(display("Unable to read /proc/cmdline file {}.", source))]
    FileRead { source: std::io::Error },

    #[snafu(display("Failed to parse '{}' output: {}", command, source))]
    FromUtf8 {
        command: String,
        source: std::string::FromUtf8Error,
    },

    #[snafu(display("Unable to get path metadata"))]
    PathMetadata { source: std::io::Error },

    #[snafu(display("Unable to read {} from /proc/cmdline file.", param))]
    ReadKernelParams { param: String },

    #[snafu(display("Unable to get path of temporary dir."))]
    TempDirPath,

    #[snafu(display("Failed to run repair successfully. {}", source))]
    RepairCommandExecution { source: xfscli::error::Error },
}

impl Error {
    pub fn exit_code(&self) -> ExitCode {
        match self {
            // Return error codes as per script
            Error::CommandUnavailable { .. } => ExitCode::from(4),
            Error::DeviceNotFound { .. } => ExitCode::from(8),
            Error::DirtyLogs => ExitCode::from(4),
            Error::MakeDir { .. } => ExitCode::from(1),
            Error::Mount => ExitCode::from(1),
            Error::RepairFailure { .. } => ExitCode::from(4),
            Error::UnrecognizedExitCode { .. } => ExitCode::from(4),

            // Return 8 when device argument is not provided
            Error::ParseTarget => ExitCode::from(8),

            // Return 8 for general errors
            Error::CommandFailure { .. } => ExitCode::from(8),
            Error::DeleteMountDirectory { .. } => ExitCode::from(8),
            Error::FindBasename { .. } => ExitCode::from(8),
            Error::FileRead { .. } => ExitCode::from(8),
            Error::FromUtf8 { .. } => ExitCode::from(8),
            Error::PathMetadata { .. } => ExitCode::from(8),
            Error::ReadKernelParams { .. } => ExitCode::from(8),
            Error::RepairCommandExecution { .. } => ExitCode::from(8),
            Error::TempDirPath => ExitCode::from(8),
        }
    }
}
