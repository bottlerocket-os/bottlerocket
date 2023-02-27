use migration_helpers::common_migrations::AddPrefixesMigration;
use migration_helpers::{migrate, Result};
use std::process;

/// We added new settings under `settings.dns` for configuring /etc/resolv.conf
fn run() -> Result<()> {
    migrate(AddPrefixesMigration(vec![
        "settings.dns",
        "services.dns",
        "configuration-files.netdog-toml",
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
