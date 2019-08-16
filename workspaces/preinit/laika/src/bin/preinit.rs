/*!
# Introduction

laika is a preinit agent that mounts the necessary filesystems for setting up Thar.

After the filesystems are mounted, laika then invokes the init process.
*/

use filetime::FileTime;
use snafu::{IntoError, ResultExt};
use std::os::unix::process::CommandExt;
use std::process::Command;
use sys_mount::{Mount, MountFlags};

type Result<T> = std::result::Result<T, error::LaikaError>;

mod error {
    use snafu::Snafu;

    /// Potential errors during pre-init process.
    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(super) enum LaikaError {
        #[snafu(display("Failed to mount {} to {} : {}", device, target, source))]
        MountFailed {
            device: String,
            target: String,
            source: std::io::Error,
        },

        #[snafu(display("Failed to create directory {}: {}", directory, source))]
        CreateDirectoryFailed {
            directory: String,
            source: std::io::Error,
        },

        #[snafu(display("Failed to set timestamp for {} to {}: {}", path, time, source))]
        ModifyFileTime {
            path: String,
            time: filetime::FileTime,
            source: std::io::Error,
        },

        #[snafu(display("Failed to execute {}: {}", path, source))]
        InitExecFailed {
            path: String,
            source: std::io::Error,
        },
    }
}

fn main() -> Result<()> {
    // Mount flag consts
    const NOATIME: MountFlags = MountFlags::NOATIME;
    const NOSUID: MountFlags = MountFlags::NOSUID;
    const NODEV: MountFlags = MountFlags::NODEV;
    const NOEXEC: MountFlags = MountFlags::NOEXEC;

    // Try and mount tmpfs directories
    for target in vec![
        ("/etc", NOATIME | NOSUID | NODEV | NOEXEC),
        ("/var", NOATIME | NOSUID | NODEV | NOEXEC),
        ("/opt", NOATIME | NOSUID | NODEV),
    ] {
        Mount::new("tmpfs", target.0, "tmpfs", target.1, Some("mode=0755")).context(
            error::MountFailed {
                device: "tmpfs",
                target: target.0,
            },
        )?;
    }

    // Create overlayfs directories
    for dir in vec!["/opt/cni", "/opt/upper", "/opt/work"] {
        std::fs::create_dir_all(dir).context(error::CreateDirectoryFailed { directory: dir })?;
    }

    // Try and mount overlayfs
    Mount::new(
        "overlay",
        "/opt/cni",
        "overlay",
        MountFlags::empty(),
        Some("lowerdir=/usr/libexec/cni,upperdir=/opt/upper,workdir=/opt/work"),
    )
    .context(error::MountFailed {
        device: "overlay",
        target: "/opt/cni",
    })?;

    let unix_epoch = FileTime::zero();

    // Set the file modification times for /etc, /var, /opt to the unix epoch time to ensure that
    // systemd detect these directories as 'outdated/uninitialized' and perform all the initialization
    // it needs to do at boot time (e.g. systemd-tmpfiles)
    for dir in vec!["/etc", "/var", "/opt"] {
        filetime::set_file_mtime(dir, unix_epoch).context(error::ModifyFileTime {
            path: dir,
            time: unix_epoch,
        })?;
    }

    // Start the init process
    let err = Command::new("/sbin/init").exec();

    // If the exec call succeeded, it should not have returned.
    Err(error::InitExecFailed { path: "/sbin/init" }.into_error(err))
}
