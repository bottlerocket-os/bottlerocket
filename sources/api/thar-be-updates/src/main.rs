/*!
# Introduction

thar-be-updates is a Bottlerocket update dispatcher that serves as an interface for the `apiserver` to issue update commands and monitor update status.

It models the Bottlerocket update process after a state machine and provides several update commands that modifies the update state.
It keeps track of the update state and other stateful update information in a update status file located at `/run/update-status`

Upon receiving a command not allowed by the update state, thar-be-updates exits immediately with an exit status indicating so.
Otherwise, thar-be-updates forks a child process to spawn the necessary process to do the work.
The parent process immediately returns back to the caller with an exit status of `0`.
The output and status of the command will be written to the update status file.
This allows the caller to synchronously call thar-be-updates without having to wait for a result to come back.

thar-be-updates uses a lockfile to control read/write access to the disks and the update status file.

*/

use fs2::FileExt;
use log::{debug, warn};
use nix::unistd::{fork, ForkResult};
use num_traits::cast::ToPrimitive;
use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};
use snafu::ensure;
use snafu::{OptionExt, ResultExt};
use std::fs::File;
use std::path::Path;
use std::process::{exit, Command};
use std::str::FromStr;
use std::{env, process};
use tempfile::NamedTempFile;
use thar_be_updates::error;
use thar_be_updates::error::{Error, Result, TbuErrorStatus};
use thar_be_updates::status::{
    get_update_status, UpdateCommand, UpdateState, UpdateStatus, UPDATE_LOCKFILE,
    UPDATE_STATUS_FILE,
};

const UPDATE_STATUS_DIR: &str = "/run/cache/thar-be-updates";

/// Stores the command line arguments
struct Args {
    subcommand: UpdateCommand,
    log_level: LevelFilter,
    socket_path: String,
}

/// Prints an usage message
fn usage() -> ! {
    let program_name = env::args().next().unwrap_or_else(|| "program".to_string());
    eprintln!(
        r"Usage: {}
            Subcommands:
                refresh     Query update repository, store the list of available updates,
                            and check if chosen version is available
                prepare     Download the chosen update and write the update image to the
                            inactive partition
                activate    Marks the inactive partition for boot
                deactivate  Reverts update activation by marking current active partition for boot

            Global options:
                    [ --socket-path PATH ]    Bottlerocket API socket path (default {})
                    [ --log-level trace|debug|info|warn|error ]  (default info)",
        program_name,
        constants::API_SOCKET,
    );
    process::exit(2);
}

/// Prints a more specific message before exiting through usage().
fn usage_msg<S: AsRef<str>>(msg: S) -> ! {
    eprintln!("{}\n", msg.as_ref());
    usage();
}

/// Parses the command line arguments
fn parse_args(args: std::env::Args) -> Args {
    let mut subcommand = None;
    let mut log_level = None;
    let mut socket_path = None;

    let mut iter = args.skip(1).peekable();
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "--log-level" => {
                let log_level_str = iter
                    .next()
                    .unwrap_or_else(|| usage_msg("Did not give argument to --log-level"));
                log_level = Some(LevelFilter::from_str(&log_level_str).unwrap_or_else(|_| {
                    usage_msg(format!("Invalid log level '{}'", log_level_str))
                }));
            }

            "--socket-path" => {
                socket_path = Some(
                    iter.next()
                        .unwrap_or_else(|| usage_msg("Did not give argument to --socket-path")),
                )
            }
            // Assume any arguments not prefixed with '-' is a subcommand
            s if !s.starts_with('-') => {
                if subcommand.is_some() {
                    usage();
                }
                subcommand =
                    Some(serde_plain::from_str::<UpdateCommand>(s).unwrap_or_else(|_| usage()));
            }
            _ => usage(),
        }
    }

    Args {
        subcommand: subcommand.unwrap_or_else(|| usage()),
        log_level: log_level.unwrap_or(LevelFilter::Info),
        socket_path: socket_path.unwrap_or_else(|| constants::API_SOCKET.to_string()),
    }
}

// Simple wrapper for locking
// Once we fork, the parent process and child process are going to have duplicate file descriptors
// that refer to the same lock. Once the parent returns and closes its copy of the lockfile fd,
// the child will still hold the lock. The lock is only released when all copies of the file descriptor are closed.
fn lock_exclusive(lockfile: &File) -> Result<()> {
    lockfile
        .try_lock_exclusive()
        .context(error::UpdateLockHeldSnafu {
            path: UPDATE_LOCKFILE,
        })?;
    debug!("Obtained exclusive lock");
    Ok(())
}

/// Initializes the update status and creates the update status file
fn initialize_update_status() -> Result<()> {
    let mut new_status = UpdateStatus::new();
    // Initialize active partition set information
    new_status.update_active_partition_info()?;
    write_update_status(&new_status)
}

/// Atomically writes out the update status to disk
fn write_update_status(update_status: &UpdateStatus) -> Result<()> {
    // Create the status file as a temporary file first and finish writing to it
    // before swapping the old status file out
    let status_file_tempfile =
        NamedTempFile::new_in(UPDATE_STATUS_DIR).context(error::CreateTempfileSnafu)?;
    serde_json::to_writer_pretty(&status_file_tempfile, update_status).context(
        error::StatusWriteSnafu {
            path: status_file_tempfile.path(),
        },
    )?;
    let tempfile_path = status_file_tempfile.into_temp_path();
    debug!("Updating status file in '{}'", UPDATE_STATUS_FILE);
    tempfile_path
        .persist(UPDATE_STATUS_FILE)
        .context(error::CreateStatusFileSnafu {
            path: UPDATE_STATUS_FILE,
        })?;
    Ok(())
}

/// This macros encapsulates the boilerplate code for dispatching the update command in a forked process
macro_rules! fork_and_return {
    ($child_process:block) => {
        match unsafe { fork() } {
            Ok(ForkResult::Parent { child, .. }) => {
                debug!("forked child pid: {}", child);
                // Exit immediately as the parent
                // Parent's lockfile fd will close but child will still have a duplicate fd
                exit(0);
            }
            Ok(ForkResult::Child) => $child_process,
            Err(e) => {
                eprintln!("{}", e);
                error::ForkSnafu.fail()
            }
        }
    };
}

/// Spawns updog process to get list of updates and check if any of them can be updated to.
/// Returns true if there is an available update, returns false otherwise.
fn refresh(status: &mut UpdateStatus, socket_path: &str) -> Result<bool> {
    fork_and_return!({
        debug!("Spawning 'updog whats'");
        let output = Command::new("updog")
            .args(["whats", "--all", "--json"])
            .output()
            .context(error::UpdogSnafu)?;
        status.set_recent_command_info(UpdateCommand::Refresh, &output);
        if !output.status.success() {
            warn!("Failed to check for updates with updog");
            return Ok(false);
        }
        let update_info: Vec<update_metadata::Update> =
            serde_json::from_slice(&output.stdout).context(error::UpdateInfoSnafu)?;
        status.update_available_updates(socket_path, update_info)
    })
}

/// Prepares the update by downloading and writing the update to the staging partition
fn prepare(status: &mut UpdateStatus) -> Result<()> {
    fork_and_return!({
        debug!("Spawning 'updog update-image'");
        let chosen_update = status
            .chosen_update()
            .context(error::UpdateDoesNotExistSnafu)?
            .clone();
        let output = Command::new("updog")
            .arg("update-image")
            .output()
            .context(error::UpdogSnafu)?;
        status.set_recent_command_info(UpdateCommand::Prepare, &output);
        if !output.status.success() {
            warn!("Failed to prepare the update with updog");
            return error::PrepareUpdateSnafu.fail();
        }
        status.set_staging_partition_image_info(chosen_update);
        Ok(())
    })
}

/// "Activates" the staged update by letting updog set up the appropriate boot flags
fn activate(status: &mut UpdateStatus) -> Result<()> {
    fork_and_return!({
        debug!("Spawning 'updog update-apply'");
        let output = Command::new("updog")
            .arg("update-apply")
            .output()
            .context(error::UpdogSnafu)?;
        status.set_recent_command_info(UpdateCommand::Activate, &output);
        if !output.status.success() {
            warn!("Failed to activate the update with updog");
            return error::ActivateUpdateSnafu.fail();
        }
        status.mark_staging_partition_next_to_boot()
    })
}

/// "Deactivates" the staged update by rolling back actions done by `activate_update`
fn deactivate(status: &mut UpdateStatus) -> Result<()> {
    fork_and_return!({
        debug!("Spawning 'updog update-revert'");
        let output = Command::new("updog")
            .arg("update-revert")
            .output()
            .context(error::UpdogSnafu)?;
        status.set_recent_command_info(UpdateCommand::Deactivate, &output);
        if !output.status.success() {
            warn!("Failed to deactivate the update with updog");
            return error::DeactivateUpdateSnafu.fail();
        }
        status.unmark_staging_partition_next_to_boot()
    })
}

/// Given the update command, this drives the update state machine.
fn drive_state_machine(
    update_status: &mut UpdateStatus,
    operation: &UpdateCommand,
    socket_path: &str,
) -> Result<()> {
    let new_state = match (operation, update_status.update_state()) {
        (UpdateCommand::Refresh, UpdateState::Idle)
        | (UpdateCommand::Refresh, UpdateState::Available) => {
            if refresh(update_status, socket_path)? {
                // Transitions state to `Available` if there is an available update
                UpdateState::Available
            } else {
                // Go to Idle otherwise
                UpdateState::Idle
            }
        }
        // Refreshing the list of updates is allowed under every update state
        (UpdateCommand::Refresh, _) => {
            refresh(update_status, socket_path)?;
            // No need to transition state here as we're already beyond `Available`
            update_status.update_state().to_owned()
        }
        // Preparing the update is allowed when the state is either `Available` or `Staged`
        (UpdateCommand::Prepare, UpdateState::Available)
        | (UpdateCommand::Prepare, UpdateState::Staged) => {
            // Make sure the chosen update exists
            ensure!(
                update_status.chosen_update().is_some(),
                error::UpdateDoesNotExistSnafu
            );
            prepare(update_status)?;
            // If we succeed in preparing the update, we transition to `Staged`
            UpdateState::Staged
        }
        // Activating the update is only allowed when the state is `Staged`
        (UpdateCommand::Activate, UpdateState::Staged) => {
            // Make sure there's an update image written to the inactive partition
            ensure!(
                update_status.staging_partition().is_some(),
                error::StagingPartitionSnafu
            );
            activate(update_status)?;
            // If we succeed in activating the update, we transition to `Ready`
            UpdateState::Ready
        }
        // Deactivating the update is only allowed when the state is `Ready`
        (UpdateCommand::Deactivate, UpdateState::Ready) => {
            // Make sure there's an update image written to the inactive partition
            ensure!(
                update_status.staging_partition().is_some(),
                error::StagingPartitionSnafu
            );
            deactivate(update_status)?;
            // If we succeed in deactivating the update, we transition to `Staged`
            UpdateState::Staged
        }
        // Everything else is disallowed
        _ => {
            return error::DisallowCommandSnafu {
                command: operation.clone(),
                state: update_status.update_state().to_owned(),
            }
            .fail();
        }
    };
    update_status.set_update_state(new_state);
    Ok(())
}

fn run() -> Result<()> {
    // Parse and store the args passed to the program
    let args = parse_args(env::args());

    // SimpleLogger will send errors to stderr and anything less to stdout.
    SimpleLogger::init(args.log_level, LogConfig::default()).context(error::LoggerSnafu)?;

    // Open the lockfile for concurrency control, create it if it doesn't exist
    let lockfile = File::create(UPDATE_LOCKFILE).context(error::UpdateLockFileSnafu {
        path: UPDATE_LOCKFILE,
    })?;
    // Obtain an exclusive lock for upcoming operations to the status file
    lock_exclusive(&lockfile)?;

    // Check if the update status file exists. If it doesn't, create and initialize it.
    if !Path::new(UPDATE_STATUS_FILE).is_file() {
        // Get an exclusive lock for creating the update status file
        initialize_update_status()?;
    }
    let mut update_status = get_update_status(&lockfile)?;

    // The commands inside drive_state_machine update the update_status object (hence &mut) to
    // reflect success or failure, and we want to reflect that in our status file regardless of
    // success, so we store the result rather than returning early here.
    let result = drive_state_machine(&mut update_status, &args.subcommand, &args.socket_path);
    write_update_status(&update_status)?;
    result
}

fn match_error_to_exit_status(err: Error) -> i32 {
    match err {
        Error::UpdateLockHeld { .. } => TbuErrorStatus::UpdateLockHeld,
        Error::DisallowCommand { .. } => TbuErrorStatus::DisallowCommand,
        Error::UpdateDoesNotExist { .. } => TbuErrorStatus::UpdateDoesNotExist,
        Error::StagingPartition { .. } => TbuErrorStatus::NoStagedImage,
        _ => TbuErrorStatus::OtherError,
    }
    .to_i32()
    .unwrap_or(1)
}

// Note: don't use tokio::main here, or a similar process-wide async runtime.  We rely on forking
// for long-running update-related actions, and tokio's threaded runtime doesn't play well.
fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(match_error_to_exit_status(e));
    }
}
