use migration_helpers::common_migrations::AddSettingsMigration;
use migration_helpers::{migrate, Result};
use std::process;

/// We added two new settings for configuring kubelet, `settings.kubernetes.cpu-manager-reconcile-period`
/// and `settings.kubernetes.cpu-manager-policy`
fn run() -> Result<()> {
    migrate(AddSettingsMigration(&[
        "settings.kubernetes.cpu-manager-policy",
        "settings.kubernetes.cpu-manager-reconcile-period",
    ]))
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
