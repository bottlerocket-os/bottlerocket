use migration_helpers::common_migrations::{AddMetadataMigration, SettingMetadata};
use migration_helpers::migrate;
use migration_helpers::Result;
use std::process;

/// We added a new setting for configuring the NVIDIA k8s device plugin.
fn run() -> Result<()> {
    migrate(AddMetadataMigration(&[SettingMetadata {
        metadata: &["affected-services"],
        setting: "settings.kubernetes.device-plugins.nvidia",
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
