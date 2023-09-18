use migration_helpers::common_migrations::{ListReplacement, ReplaceListsMigration};
use migration_helpers::{migrate, Result};
use std::process;

/// We added a config file to the configuration-files list for services.kernel-modules
/// to facilitate module autoload. This needs to be restored to prior values on downgrade.
fn run() -> Result<()> {
    migrate(ReplaceListsMigration(vec![ListReplacement {
        setting: "services.kernel-modules.configuration-files",
        old_vals: &["modprobe-conf"],
        new_vals: &["modprobe-conf", "modules-load"],
    }]))
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
