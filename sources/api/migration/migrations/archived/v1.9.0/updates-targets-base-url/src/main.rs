use migration_helpers::common_migrations::{AddMetadataMigration, SettingMetadata};
use migration_helpers::{migrate, Result};
use std::process;

/// We added a `setting-generator` for `settings.updates.targets-base-url` on AWS variants.
/// This migration will do nothing on upgrade, but will remove the metadata if present on downgrade.
fn run() -> Result<()> {
    migrate(AddMetadataMigration(&[SettingMetadata {
        setting: "settings.updates.targets-base-url",
        metadata: &["setting-generator"],
    }]))
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
