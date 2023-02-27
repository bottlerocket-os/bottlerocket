use migration_helpers::common_migrations::{AddMetadataMigration, SettingMetadata};
use migration_helpers::{migrate, Result};
use std::process;

/// We added a new setting and `affected-services` metadata for `container-registry.credentials`
/// We subdivided metadata for `container-registry` into `container-registry.mirrors` and `container-registry.credentials`
/// This is for the docker variants where don't want to restart the docker daemon when credentials settings change.
fn run() -> Result<()> {
    migrate(AddMetadataMigration(&[
        SettingMetadata {
            metadata: &["affected-services"],
            setting: "settings.container-registry.credentials",
        },
        SettingMetadata {
            metadata: &["affected-services"],
            setting: "settings.container-registry.mirrors",
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
