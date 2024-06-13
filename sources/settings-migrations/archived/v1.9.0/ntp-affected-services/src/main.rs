use migration_helpers::common_migrations::{
    MetadataListReplacement, ReplaceMetadataListsMigration,
};
use migration_helpers::{migrate, Result};
use std::process;

/// We updated the 'affected-services' list metadata for 'settings.ntp' to refer
/// to the correct service name ("ntp") instead of the incorrect one ("chronyd").
fn run() -> Result<()> {
    migrate(ReplaceMetadataListsMigration(vec![
        MetadataListReplacement {
            setting: "settings.ntp",
            metadata: "affected-services",
            old_vals: &["chronyd"],
            new_vals: &["ntp"],
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
