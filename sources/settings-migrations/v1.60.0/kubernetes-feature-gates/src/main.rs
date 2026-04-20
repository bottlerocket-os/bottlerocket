use migration_helpers::common_migrations::AddPrefixesMigration;
use migration_helpers::{migrate, Result};
use std::process;

/// We added `settings.kubernetes.feature-gates` to allow users to enable or
/// disable kubelet feature gates. Individual gates are stored as sub-keys
/// (e.g. `settings.kubernetes.feature-gates.MemoryQoS = true`). We don't
/// want to enumerate all possible gate names, so we remove the whole prefix
/// when downgrading to a version that doesn't know about this setting.
fn run() -> Result<()> {
    migrate(AddPrefixesMigration(vec![
        "settings.kubernetes.feature-gates",
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
