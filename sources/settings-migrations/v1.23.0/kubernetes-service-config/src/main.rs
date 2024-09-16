use migration_helpers::common_migrations::ReplaceStringMigration;
use migration_helpers::{migrate, Result};
use std::process;

const OLD_MODE: &str = "0600";
const NEW_MODE: &str = "0644";

/// We bumped the version of the default control container
fn run() -> Result<()> {
    migrate(ReplaceStringMigration {
        setting: "configuration-files.kubelet-exec-start-conf.mode",
        old_val: OLD_MODE,
        new_val: NEW_MODE,
    })
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
