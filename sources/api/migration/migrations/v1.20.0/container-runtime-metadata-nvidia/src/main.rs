use migration_helpers::common_migrations::{AddMetadataMigration, NoOpMigration, SettingMetadata};
use migration_helpers::migrate;
use migration_helpers::Result;
use std::process;

/// We added a new setting for configuring container runtime (containerd) settings only for NVIDIA k8s variants.
fn run() -> Result<()> {
    if cfg!(variant_family = "aws-k8s") && cfg!(variant_flavor = "nvidia") {
        migrate(AddMetadataMigration(&[SettingMetadata {
            metadata: &["affected-services"],
            setting: "settings.container-runtime",
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
