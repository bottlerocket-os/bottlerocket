use migration_helpers::common_migrations::{
    MetadataListReplacement, ReplaceMetadataListsMigration,
};
use migration_helpers::{migrate, Result};
use std::process;

/// We updated the 'affected-services' list metadata for 'settings.network.hostname' to include the
/// hosts "service" on upgrade, and to remove it on downgrade.
fn run() -> Result<()> {
    migrate(ReplaceMetadataListsMigration(vec![
        MetadataListReplacement {
            setting: "settings.network.hostname",
            metadata: "affected-services",
            old_vals: &["hostname"],
            new_vals: &["hostname", "hosts"],
        },
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
