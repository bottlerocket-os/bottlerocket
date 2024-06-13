use migration_helpers::common_migrations::{
    MetadataListReplacement, ReplaceMetadataListsMigration,
};
use migration_helpers::{migrate, Result};
use std::process;

/// We updated the 'affected-services' list metadata for 'settings.network' to include
/// host-containers. The metadata list need to be restored to the prior value on downgrade and
/// updated to include host-containers on upgrades.
/// We're trying to match old values for different variants.
fn run() -> Result<()> {
    migrate(ReplaceMetadataListsMigration(vec![
        MetadataListReplacement {
            setting: "settings.network",
            metadata: "affected-services",
            old_vals: &["containerd", "host-containerd"],
            new_vals: &["containerd", "host-containerd", "host-containers"],
        },
        // For K8S variants
        MetadataListReplacement {
            setting: "settings.network",
            metadata: "affected-services",
            old_vals: &["containerd", "kubernetes", "host-containerd"],
            new_vals: &[
                "containerd",
                "kubernetes",
                "host-containerd",
                "host-containers",
            ],
        },
        // For the aws-ecs-1 variant
        MetadataListReplacement {
            setting: "settings.network",
            metadata: "affected-services",
            old_vals: &["containerd", "docker", "ecs", "host-containerd"],
            new_vals: &[
                "containerd",
                "docker",
                "ecs",
                "host-containerd",
                "host-containers",
            ],
        },
        // For aws-dev and vmware-dev variants
        MetadataListReplacement {
            setting: "settings.network",
            metadata: "affected-services",
            old_vals: &["containerd", "docker", "host-containerd"],
            new_vals: &["containerd", "docker", "host-containerd", "host-containers"],
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
