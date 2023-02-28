use migration_helpers::common_migrations::AddSettingsMigration;
use migration_helpers::{migrate, Result};
use std::process;

/// We added a new setting for configuring kubelet's image-gc-high-threshold-percent
/// and image-gc-low-threshold-percent options,
/// `settings.kubernetes.image-gc-high-threshold-percent` and
/// `settings.kubernetes.image-gc-low-threshold-percent`
fn run() -> Result<()> {
    migrate(AddSettingsMigration(&[
        "settings.kubernetes.image-gc-high-threshold-percent",
        "settings.kubernetes.image-gc-low-threshold-percent",
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
