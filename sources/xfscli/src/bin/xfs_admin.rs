use snafu::{OptionExt, ResultExt};
use std::process::Command;
use std::process::ExitCode;
use std::str;

use argh::FromArgs;

use xfscli::error::{self, Error, Result};
use xfscli::{get_mount_point, perform_xfs_repair, query_xfs_db, XFS_IO};

// This CLI is rewritten in Rust for xfs_admin shell script
// https://git.kernel.org/pub/scm/fs/xfs/xfsprogs-dev.git/tree/db/xfs_admin.sh
// since there isn't a shell on Bottlerocket hosts.

/// change parameters of an XFS filesystem
#[derive(FromArgs)]
struct Args {
    /// enables unwritten extent support on a filesystem that is disabled
    #[argh(switch, short = 'e')]
    extent_flag: bool,

    /// specifies that the filesystem image to be processed is stored in a regular file at device
    #[argh(switch, short = 'f')]
    filesystem_image: bool,

    /// prints the current filesystem label
    #[argh(switch, short = 'l')]
    filesystem_label: bool,

    /// prints the current filesystem UUID
    #[argh(switch, short = 'u')]
    filesystem_uuid: bool,

    /// set the filesystem label to label
    #[argh(option, short = 'L')]
    label: Option<String>,

    /// enable (1) or disable (0) lazy-counters in the filesystem
    #[argh(option, short = 'c')]
    lazy_counters: Option<usize>,

    /// enables version 2 log format
    #[argh(switch, short = 'j')]
    log_version_2: bool,

    /// filesystem's external log location
    #[argh(option)]
    log_location: Option<String>,

    /// enable 32bit project identifier support
    #[argh(switch, short = 'p')]
    project_identifier_32_bit: bool,

    /// specifies the device special file where the filesystem's realtime section resides
    #[argh(option, short = 'r')]
    realtime_device: Option<String>,

    /// set the UUID of the filesystem to uuid
    #[argh(option, short = 'U')]
    uuid: Option<String>,

    /// add or remove features on an existing V5 filesystem
    #[argh(option, short = 'O')]
    v5_feature: Option<String>,

    /// prints the version number and exits.
    #[argh(switch, short = 'V')]
    version: bool,

    /// block-device
    #[argh(positional)]
    device: Option<String>,
}

impl Args {
    // Filesystem should not be mounted with args in this function
    // For more reference check require_offline variable in
    // https://git.kernel.org/pub/scm/fs/xfs/xfsprogs-dev.git/tree/db/xfs_admin.sh
    fn is_required_offline(&self) -> bool {
        self.extent_flag
            || self.filesystem_image
            || self.log_version_2
            || self.project_identifier_32_bit
            || self.uuid.is_some()
            || self.lazy_counters.is_some()
            || self.v5_feature.is_some()
            || self.realtime_device.is_some()
    }
}

fn main() -> ExitCode {
    let args: Args = argh::from_env();
    match run(args) {
        Ok(exit_code) => exit_code,
        Err(e) => {
            eprintln!("{}", e);
            e.exit_code()
        }
    }
}

fn run(args: Args) -> Result<ExitCode> {
    // Return the version for arg version as true
    if args.version {
        let xfs_db_response = query_xfs_db(vec!["-p", "xfs_admin", "-V"])?;
        return Ok(ExitCode::from(xfs_db_response as u8));
    }

    let device = &args.device.as_ref().context(error::ParseTargetSnafu)?;

    // Collect all the arguments for XFS_DB, XFS_IO and XFS_REPAIR commands
    // by parsing the CLI inputs.
    let mut db_opts: Vec<&str> = vec![];
    let mut io_opts: Vec<&str> = vec![];
    let mut repair_opts: Vec<&str> = vec![];

    let lazycount_arg;
    if let Some(lazy_counters) = &args.lazy_counters {
        lazycount_arg = format!("lazycount={}", lazy_counters);
        repair_opts.append(&mut vec!["-c", lazycount_arg.as_str()]);
    }

    if args.extent_flag {
        db_opts.append(&mut vec!["-c", "version extflg"]);
    }

    if args.filesystem_image {
        db_opts.push("-f");
    }

    if args.log_version_2 {
        db_opts.append(&mut vec!["-c", "version log2"]);
    }

    if args.filesystem_label {
        db_opts.append(&mut vec!["-r", "-c", "label"]);
        io_opts.append(&mut vec!["-r", "-c", "label"]);
    }

    let label_arg_db;
    let label_arg_io;
    if let Some(label) = &args.label {
        label_arg_db = format!("label {}", label);
        db_opts.append(&mut vec!["-c", label_arg_db.as_str()]);

        label_arg_io = match label.as_str() {
            "--" => format!("label {}", label),
            _ => {
                format!("label -s {}", label)
            }
        };
        io_opts.append(&mut vec!["-c", label_arg_io.as_str()]);
    }

    if let Some(v5_feature) = &args.v5_feature {
        repair_opts.append(&mut vec!["-c", v5_feature.as_str()]);
    }

    if args.project_identifier_32_bit {
        db_opts.append(&mut vec!["-c", "version projid32bit"]);
    }

    if args.filesystem_uuid {
        db_opts.append(&mut vec!["-r", "-c", "uuid"]);
        io_opts.append(&mut vec!["-r", "-c", "fsuuid"]);
    }

    let uuid_arg;
    if let Some(uuid) = &args.uuid {
        uuid_arg = format!("uuid {}", uuid);
        db_opts.append(&mut vec!["-c", uuid_arg.as_str()]);
    }

    // Exit if filesystem is mounted and require offline for queried arguments
    if let Ok(mpt) = get_mount_point(device) {
        // filesystem is mounted
        if args.is_required_offline() {
            return Err(Error::MountedFilesystem { mount_point: mpt });
        }

        if !io_opts.is_empty() {
            let mut xfs_io = Command::new(XFS_IO);
            xfs_io.args(["-p", "xfs_admin"]);
            xfs_io.args(io_opts);
            xfs_io.arg(mpt.as_str().trim());

            let xfs_io_result = xfs_io.output().context(error::CommandFailureSnafu {
                command: "xfs_io".to_string(),
            })?;

            print!(
                "{}",
                String::from_utf8(xfs_io_result.stdout).context(error::FromUtf8Snafu {
                    command: "xfs io".to_string(),
                })?,
            );
            let xfs_io_result =
                xfs_io_result
                    .status
                    .code()
                    .context(error::ParseStatusCodeSnafu {
                        command: "xfs_io".to_string(),
                    })?;
            return Ok(ExitCode::from(xfs_io_result as u8));
        }
    }

    let mut status = 0;
    if !db_opts.is_empty() {
        let mut xfs_db_args = vec!["-x", "-p", "xfs_admin"];

        let log_location_arg;
        if let Some(log_location) = &args.log_location {
            log_location_arg = format!("-l {}", log_location);
            xfs_db_args.push(log_location_arg.as_str());
        }

        xfs_db_args.append(&mut db_opts);
        xfs_db_args.push(device);

        status = query_xfs_db(xfs_db_args)?;
    }

    if !repair_opts.is_empty() {
        print!("Running xfs_repair to upgrade filesystem.");
        let mut repair_args: Vec<&str> = vec![];

        let log_location_arg;
        if let Some(log_location) = &args.log_location {
            log_location_arg = format!("-l {}", log_location);
            repair_args.push(log_location_arg.as_str());
        }

        let realtime_device_arg;
        if let Some(realtime_device) = &args.realtime_device {
            realtime_device_arg = format!("-r {}", realtime_device);
            repair_args.push(realtime_device_arg.as_str());
        }

        repair_args.append(&mut repair_opts);
        repair_args.push(device.as_str());

        status += perform_xfs_repair(repair_args)?.exit_code();
    }
    Ok(ExitCode::from(status as u8))
}
