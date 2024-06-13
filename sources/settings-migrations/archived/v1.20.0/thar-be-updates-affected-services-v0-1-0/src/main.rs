use migration_helpers::common_migrations::{
    MetadataListReplacement, ReplaceMetadataListsMigration,
};
use migration_helpers::{migrate, Result};
use std::process;
fn run() -> Result<()> {
    migrate(ReplaceMetadataListsMigration(vec![
        MetadataListReplacement {
            setting: "settings.updates",
            metadata: "affected-services",
            old_vals: &["updog"],
            new_vals: &["updog", "thar-be-updates"],
        },
    ]))
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}
