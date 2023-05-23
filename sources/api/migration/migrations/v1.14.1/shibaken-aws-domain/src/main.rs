use migration_helpers::common_migrations::{AddMetadataMigration, NoOpMigration, SettingMetadata};
use migration_helpers::{migrate, Result};
use std::process;

/// We added a `setting-generator` for `settings.aws.domain` on AWS variants.
/// This migration will do nothing on upgrade, but will remove the metadata if present on downgrade.
fn run() -> Result<()> {
    if cfg!(variant_platform = "aws") {
        migrate(AddMetadataMigration(&[SettingMetadata {
            setting: "settings.aws.domain",
            metadata: &["setting-generator"],
        }]))
    } else {
        migrate(NoOpMigration)
    }
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
