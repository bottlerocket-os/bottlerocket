use migration_helpers::common_migrations::{
    MetadataListReplacement, ReplaceMetadataListsMigration,
};
use migration_helpers::{migrate, Result};
use std::process;

fn run() -> Result<()> {
    migrate(ReplaceMetadataListsMigration(vec![
        MetadataListReplacement {
            setting: "settings.kubernetes.pod-infra-container-image",
            metadata: "affected-services",
            old_vals: &["kubernetes", "containerd"],
            new_vals: &["pod-infra-container-image"],
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
