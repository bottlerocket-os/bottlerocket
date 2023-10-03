use snafu::{OptionExt, ResultExt};
use std::os::unix::fs::FileTypeExt;
use std::path::Path;
use std::process::Command;
use std::process::ExitCode;
use std::{fs, str};

use argh::FromArgs;

use self::error::{Error, Result};
use xfscli::{perform_xfs_repair, XfsRepairResponseCode, BLKID, MOUNT, UMOUNT};

mod error;

// This CLI is rewritten in Rust for fsck.xfs shell script
// https://git.kernel.org/pub/scm/fs/xfs/xfsprogs-dev.git/tree/fsck/xfs_fsck.sh
// since there isn't a shell on Bottlerocket hosts.

const REPAIR_MOUNT_DIR: &str = "/tmp/repair_mnt";
struct KernelParams {
    root: String,
    root_flags: String,
}

/// check and repair a XFS file system
#[derive(FromArgs)]
struct Args {
    /// auto repair the file system
    #[argh(switch, short = 'a')]
    auto_repair: bool,

    /// auto check all file systems
    #[argh(switch, short = 'A')]
    auto_check: bool,

    /// check the root filesystem as well when -A flag is set
    #[argh(switch, short = 'p')]
    check_root_fs: bool,

    /// force repair filesystem
    #[argh(switch, short = 'f')]
    force: bool,

    /// repair filesystem automatically
    #[argh(switch, short = 'y')]
    repair: bool,

    /// mount-point | block-device
    #[argh(positional)]
    target: Option<String>,
}

fn main() -> ExitCode {
    let args: Args = argh::from_env();
    match run(args) {
        Ok(exit_code) => exit_code,
        Err(e) => {
            eprintln!("{:?}", e);
            e.exit_code()
        }
    }
}

fn run(args: Args) -> Result<ExitCode> {
    let auto = args.auto_check || args.auto_repair || args.check_root_fs;
    let force = args.force;
    let repair = args.repair;

    // This is diverted behavior from xfs shell script.
    // In fsck no filesystems are specified on the command line,
    // and the -A option is not specified,
    // fsck will default to checking filesystems in /etc/fstab serially
    // Hence device name is not a mandatory argument. But as there isn't a /etc/fstab
    // file in Bottlerocket we have to specify the device.
    let target = &args.target.as_ref().context(error::ParseTargetSnafu)?;

    if !Path::new(target).exists() {
        return Err(Error::DeviceNotFound {
            target: target.to_string(),
        });
    }

    if force {
        // Preform xfs_repair
        println!("Performing XFS repair..");
        let mut repair_exit_code =
            perform_xfs_repair(vec!["-e", target]).context(error::RepairCommandExecutionSnafu)?;

        // Clear the log, mount and unmount the XFS file system
        // before xfs_repair
        if repair_exit_code == XfsRepairResponseCode::DirtyLogs && repair {
            println!("Replaying log for {}", target);

            // Using random directory (/tmp/repair_mntxxxx) instead of a targeted directory (/tmp/repair_mnt)
            let mnt_dir = tempfile::Builder::new()
                .prefix(REPAIR_MOUNT_DIR)
                .tempdir()
                .context(error::MakeDirSnafu)?;

            let mnt_dir_path = mnt_dir.path().to_str().context(error::TempDirPathSnafu)?;

            let kernel_params = get_root_and_rootflags(target)?;

            let basename_dev = Path::new(target)
                .file_name()
                .context(error::FindBasenameSnafu {
                    target: "device".to_string(),
                })?;

            let basename_root =
                Path::new(&kernel_params.root)
                    .file_name()
                    .context(error::FindBasenameSnafu {
                        target: "root".to_string(),
                    })?;

            let mut mount_args = vec![target.as_str(), mnt_dir_path];
            if basename_dev == basename_root && !kernel_params.root_flags.is_empty() {
                mount_args.push(kernel_params.root_flags.as_str());
            }

            let mount_status = Command::new(MOUNT).args(mount_args).status().context(
                error::CommandFailureSnafu {
                    command: "mount".to_string(),
                },
            )?;

            if !mount_status.success() {
                return Err(Error::Mount);
            }

            Command::new(UMOUNT).arg(mnt_dir_path).output().context(
                error::CommandFailureSnafu {
                    command: "umount".to_string(),
                },
            )?;

            println!("Performing XFS repair again after cleaning the log..");
            repair_exit_code = perform_xfs_repair(vec!["-e", target])
                .context(error::RepairCommandExecutionSnafu)?;

            mnt_dir.close().context(error::DeleteMountDirectorySnafu)?;
        }

        let metadata_repaired = match repair_exit_code {
            XfsRepairResponseCode::Ok => Ok(false),
            XfsRepairResponseCode::RepairFailure => Err(Error::RepairFailure),
            XfsRepairResponseCode::DirtyLogs => Err(Error::DirtyLogs),
            XfsRepairResponseCode::MetadataRepair => Ok(true),
            XfsRepairResponseCode::CommandUnavailable => Err(Error::CommandUnavailable {
                cli: "fsck".to_string(),
                command: "xfs_repair".to_string(),
            }),
            XfsRepairResponseCode::Unknown(code) => Err(Error::UnrecognizedExitCode { code }),
        }?;

        return Ok(if metadata_repaired {
            ExitCode::from(1)
        } else {
            ExitCode::SUCCESS
        });
    }

    if auto {
        println!("fsck.xfs : XFS file system.");
    } else {
        println!(
            "If you wish to check the consistency of an XFS filesystem or \
        repair a damaged filesystem, see xfs_repair(8)."
        );
    }

    Ok(ExitCode::SUCCESS)
}

// Get the kernel parameters by reading /proc/cmdline file
fn get_root_and_rootflags(target: &String) -> Result<KernelParams> {
    let params = fs::read_to_string("/proc/cmdline").context(error::FileReadSnafu)?;
    let mut root = String::new();
    let mut root_flags = String::new();
    for param in params.split(' ') {
        if param.starts_with("root=") {
            root = param
                .strip_prefix("root=")
                .context(error::ReadKernelParamsSnafu {
                    param: "root".to_string(),
                })?
                .to_string();
        }
        if param.starts_with("rootflags=") {
            root_flags.push_str("-o ");
            root_flags.push_str(param.strip_prefix("rootflags=").context(
                error::ReadKernelParamsSnafu {
                    param: "rootflags".to_string(),
                },
            )?);
        }
    }
    let path = Path::new(&root);
    let metadata = path.metadata().context(error::PathMetadataSnafu)?;
    let file_type = metadata.file_type();

    if !file_type.is_block_device() {
        let blkid_result = Command::new(BLKID)
            .args(["-t", &root, "-o", target])
            .output()
            .context(error::CommandFailureSnafu {
                command: "blkid".to_string(),
            })?;
        root = String::from_utf8(blkid_result.stdout).context(error::FromUtf8Snafu {
            command: "blkid".to_string(),
        })?;
    };

    Ok(KernelParams { root, root_flags })
}
