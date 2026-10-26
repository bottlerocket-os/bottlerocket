use migration_helpers::common_migrations::AddSettingsMigration;
use migration_helpers::{migrate, Result};
use std::process;

/// We added a new setting for overriding the IP family the ECS agent registers with:
/// - settings.ecs.instance-ip-compatibility
fn run() -> Result<()> {
    migrate(AddSettingsMigration(&[
        "settings.ecs.instance-ip-compatibility",
    ]))
}

// Returning a Result from main makes it print a Debug representation of the error, but with Snafu
// we have nice Display representations of the error, so we wrap "main" (run) and print any error.
// https://github.com/shepmaster/snafu/issues/110
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        process::exit(1);
    }
}
