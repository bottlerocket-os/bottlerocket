use migration_helpers::common_migrations::AddPrefixesMigration;
use migration_helpers::{migrate, Result};
use std::process;

/// We added new settings under `settings.kernel.modules` for configuring
/// /etc/modules-load.d/modules-load.conf. The actual autoload settings are
/// migrated separately in kernel-modules-autoload-settings migration as they
/// require a custom migration implementation.
fn run() -> Result<()> {
    migrate(AddPrefixesMigration(vec![
        "configuration-files.modules-load",
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
