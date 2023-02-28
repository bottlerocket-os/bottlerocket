use migration_helpers::common_migrations::AddPrefixesMigration;
use migration_helpers::{migrate, Result};
use std::process;

/// We added new configuration files and restart commands for docker and host-containerd.
/// On downgrade we need to remove all settings under these services
fn run() -> Result<()> {
    migrate(AddPrefixesMigration(vec![
        "services.docker",
        "services.host-containerd",
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
