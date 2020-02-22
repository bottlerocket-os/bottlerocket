#![deny(rust_2018_idioms)]

use migration_helpers::{migrate, Result};
use migration_helpers::common_migrations::AddSettingMigration;
use std::process;

/// We added a generated setting, "settings.aws.region", and want to make sure it's removed before
/// we go back to old versions that don't understand it.
fn run() -> Result<()> {
    migrate(AddSettingMigration("settings.aws.region"))
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
