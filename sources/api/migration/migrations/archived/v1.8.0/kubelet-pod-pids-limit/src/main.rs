use migration_helpers::common_migrations::AddPrefixesMigration;
use migration_helpers::{migrate, Result};
use std::process;

/// We added a new setting for configuring pod-pids-limit, `settings.kubernetes.pod-pids-limit`
fn run() -> Result<()> {
    migrate(AddPrefixesMigration(vec![
        "settings.kubernetes.pod-pids-limit",
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
