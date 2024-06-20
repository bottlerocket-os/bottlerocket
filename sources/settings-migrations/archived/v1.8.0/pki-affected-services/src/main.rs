use migration_helpers::common_migrations::{
    MetadataListReplacement, ReplaceMetadataListsMigration,
};
use migration_helpers::{migrate, Result};
use std::process;

/// We updated the 'affected-services' list metadata for 'settings.pki' to include
/// containerd or docker on upgrade, and to remove them on downgrade depending on the
/// running variant.
fn run() -> Result<()> {
    migrate(ReplaceMetadataListsMigration(vec![
        MetadataListReplacement {
            setting: "settings.pki",
            metadata: "affected-services",
            old_vals: &["pki"],
            new_vals: if cfg!(variant_runtime = "k8s") {
                &["pki", "containerd"]
            } else {
                &["pki", "docker"]
            },
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
