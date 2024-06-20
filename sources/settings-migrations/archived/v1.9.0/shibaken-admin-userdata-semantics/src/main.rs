use migration_helpers::common_migrations::{MetadataReplacement, ReplaceMetadataMigration};
use migration_helpers::{migrate, Result};
use std::process;

/// We modified the setting generator for `settings.host-containers.admin.user-data` to use the
/// new interface to shibaken.
fn run() -> Result<()> {
    migrate(ReplaceMetadataMigration(vec![MetadataReplacement {
        setting: "settings.host-containers.admin.user-data",
        metadata: "setting-generator",
        old_val: "shibaken",
        new_val: "shibaken generate-admin-userdata",
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
