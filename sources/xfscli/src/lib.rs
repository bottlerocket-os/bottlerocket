use error::Result;
use snafu::{OptionExt, ResultExt};
use std::process::Command;

pub mod error;
pub static BLKID: &str = "/usr/sbin/blkid";
pub static FINDMNT: &str = "/usr/bin/findmnt";
pub static MOUNT: &str = "/usr/bin/mount";
pub static UMOUNT: &str = "/usr/bin/umount";
pub static XFS_DB: &str = "/usr/sbin/xfs_db";
pub static XFS_IO: &str = "/usr/sbin/xfs_io";
pub static XFS_REPAIR: &str = "/usr/sbin/xfs_repair";
pub static XFS_SPACEMAN: &str = "/usr/sbin/xfs_spaceman";

/// Check if target is mounted and return mount point
pub fn get_mount_point(target: &String) -> Result<String> {
    let mountpt = Command::new(FINDMNT)
        .args(["-t", "xfs", "-f", "-n", "-o", "TARGET", target])
        .output()
        .context(error::CommandFailureSnafu {
            command: "findmnt".to_string(),
        })?;

    // All errors will be interpreted as failure in finding the mount point
    if mountpt.status.success() {
        Ok(
            String::from_utf8(mountpt.stdout).context(error::FromUtf8Snafu {
                command: "mount point".to_string(),
            })?,
        )
    } else {
        Err(error::Error::FindMount {
            target: String::from(target),
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]

pub enum XfsRepairResponseCode {
    CommandUnavailable,
    DirtyLogs,
    MetadataRepair,
    Ok,
    RepairFailure,
    Unknown(i32),
}

impl XfsRepairResponseCode {
    pub fn exit_code(self) -> i32 {
        match self {
            XfsRepairResponseCode::CommandUnavailable => 127,
            XfsRepairResponseCode::DirtyLogs => 2,
            XfsRepairResponseCode::MetadataRepair => 4,
            XfsRepairResponseCode::Ok => 0,
            XfsRepairResponseCode::RepairFailure => 1,
            XfsRepairResponseCode::Unknown(code) => code,
        }
    }
}

impl From<i32> for XfsRepairResponseCode {
    fn from(code: i32) -> Self {
        match code {
            0 => XfsRepairResponseCode::Ok,
            1 => XfsRepairResponseCode::RepairFailure,
            2 => XfsRepairResponseCode::DirtyLogs,
            4 => XfsRepairResponseCode::MetadataRepair,
            127 => XfsRepairResponseCode::CommandUnavailable,
            code => XfsRepairResponseCode::Unknown(code),
        }
    }
}

/// Perform xfs_repair for the given arguments
pub fn perform_xfs_repair(args: Vec<&str>) -> Result<XfsRepairResponseCode> {
    let xfs_repair_result =
        Command::new(XFS_REPAIR)
            .args(args)
            .status()
            .context(error::CommandFailureSnafu {
                command: "xfs_repair".to_string(),
            })?;

    let code = xfs_repair_result
        .code()
        .context(error::ParseStatusCodeSnafu {
            command: "xfs_repair".to_string(),
        })?;

    Ok(XfsRepairResponseCode::from(code))
}

/// Query xfs db to get required information
pub fn query_xfs_db(args: Vec<&str>) -> Result<i32> {
    let xfs_db_result =
        Command::new(XFS_DB)
            .args(args)
            .output()
            .context(error::CommandFailureSnafu {
                command: "xfs db".to_string(),
            })?;

    print!(
        "{}",
        String::from_utf8(xfs_db_result.stdout).context(error::FromUtf8Snafu {
            command: "xfs db".to_string(),
        })?
    );

    xfs_db_result
        .status
        .code()
        .context(error::ParseStatusCodeSnafu {
            command: "xfs_db".to_string(),
        })
}

/// Query xfs spaceman to get xfs information
pub fn query_xfs_spaceman(args: Vec<&str>) -> Result<i32> {
    let xfs_spaceman_result =
        Command::new(XFS_SPACEMAN)
            .args(args)
            .output()
            .context(error::CommandFailureSnafu {
                command: "xfs spaceman".to_string(),
            })?;

    print!(
        "{}",
        String::from_utf8(xfs_spaceman_result.stdout).context(error::FromUtf8Snafu {
            command: "xfs spaceman".to_string(),
        })?
    );

    xfs_spaceman_result
        .status
        .code()
        .context(error::ParseStatusCodeSnafu {
            command: "xfs_spaceman".to_string(),
        })
}
