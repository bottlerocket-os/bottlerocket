use migration_helpers::common_migrations::AddPrefixesMigration;
use migration_helpers::{migrate, Result};
use std::process;

/// We added a new setting for configuring kubelet's image credential
/// provider plugins. Initially this is only to support ecr-credential-provider,
/// but others may be added as needed.
fn run() -> Result<()> {
    migrate(AddPrefixesMigration(vec![
        "settings.kubernetes.credential-providers",
        "configuration-files.credential-provider-config-yaml",
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
