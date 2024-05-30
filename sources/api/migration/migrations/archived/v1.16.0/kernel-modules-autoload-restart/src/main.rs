use migration_helpers::common_migrations::{ListReplacement, ReplaceListsMigration};
use migration_helpers::{migrate, Result};
use std::process;

/// We added a new `autoload` setting to `settings.kernel.modules`, which needs
/// re restart of `systemd-modules-load.services`.
fn run() -> Result<()> {
    migrate(ReplaceListsMigration(vec![ListReplacement {
        setting: "services.kernel-modules.restart-commands",
        old_vals: &[],
        new_vals: &["/usr/bin/systemctl try-restart systemd-modules-load"],
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
