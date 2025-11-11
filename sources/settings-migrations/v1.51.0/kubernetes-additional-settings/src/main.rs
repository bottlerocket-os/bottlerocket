use migration_helpers::common_migrations::AddSettingsMigration;
use migration_helpers::{migrate, Result};
use std::process;

// We added new kubernetes settings to configure:
// - min/max duration before an unused image is garbage-collected
// - max number of image pulls in parallel
// - mapping length of UIDs and GIDs
fn run() -> Result<()> {
    migrate(AddSettingsMigration(&[
        "settings.kubernetes.image-minimum-gc-age",
        "settings.kubernetes.image-maximum-gc-age",
        "settings.kubernetes.max-parallel-image-pulls",
        "settings.kubernetes.ids-per-pod",
    ]))
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        process::exit(1);
    }
}
