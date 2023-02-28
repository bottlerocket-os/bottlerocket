use migration_helpers::common_migrations::AddPrefixesMigration;
use migration_helpers::{migrate, Result};
use std::process;

/// We added a new setting for configuring image registries, `settings.container-registry`
/// We also added a new configuration template file for the Docker daemon
fn run() -> Result<()> {
    migrate(AddPrefixesMigration(vec![
        "settings.container-registry",
        "configuration-files.docker-daemon-config",
        "configuration-files.host-ctr-toml",
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
