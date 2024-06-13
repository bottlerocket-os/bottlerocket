use migration_helpers::{error, migrate, Migration, MigrationData, Result};
use snafu::ResultExt;
use std::fs;
use std::io;
use std::process;

const CPU_MANAGER_POLICY_CHECKPOINT: &str = "/var/lib/kubelet/cpu_manager_state";

/// forward - We always remove the state file on boot, therefore we don't need to explicitly
/// remove the file during forward migration.
/// backward - We remove cpu manager policy checkpoint value on downgrade, since older versions did not
/// clean up this state file on boot.
pub struct CpuManagerPolicyCleaner;

impl Migration for CpuManagerPolicyCleaner {
    fn forward(&mut self, input: MigrationData) -> Result<MigrationData> {
        println!("CpuManagerPolicyCleaner has no work to do on upgrade.");
        Ok(input)
    }

    fn backward(&mut self, input: MigrationData) -> Result<MigrationData> {
        // removing existing cpu_manager_policy_state file
        println!(
            "Deleting existing cpu manager policy checkpoint: '{}'",
            CPU_MANAGER_POLICY_CHECKPOINT
        );
        if let Err(e) = fs::remove_file(CPU_MANAGER_POLICY_CHECKPOINT) {
            if e.kind() != io::ErrorKind::NotFound {
                return Err(e).context(error::RemoveFile {
                    path: CPU_MANAGER_POLICY_CHECKPOINT,
                });
            } else {
                println!("NotFound: '{}'", CPU_MANAGER_POLICY_CHECKPOINT)
            }
        }
        Ok(input)
    }
}
/// We changed the default for CPU manager policy and need to handle kubelet's state file.
fn run() -> Result<()> {
    migrate(CpuManagerPolicyCleaner)
}

// Returning a Result from main makes it print a Debug representation of the error, but with Snafu
// we have nice Display representations of the error, so we wrap "main" (run) and print any error.
// https://github.com/shepmaster/snafu/issues/110
fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}
