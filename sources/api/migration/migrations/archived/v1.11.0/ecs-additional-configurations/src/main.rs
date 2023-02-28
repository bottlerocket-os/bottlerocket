use migration_helpers::common_migrations::AddSettingsMigration;
use migration_helpers::{migrate, Result};
use std::process;

/// We added additional configurations for the ECS agent
fn run() -> Result<()> {
    migrate(AddSettingsMigration(&[
        "settings.ecs.container-stop-timeout",
        "settings.ecs.task-cleanup-wait",
        "settings.ecs.metadata-service-rps",
        "settings.ecs.metadata-service-burst",
        "settings.ecs.reserved-memory",
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
