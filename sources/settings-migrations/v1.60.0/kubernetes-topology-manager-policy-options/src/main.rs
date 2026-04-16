use migration_helpers::common_migrations::AddSettingsMigration;
use migration_helpers::{migrate, Result};
use std::process;

// Added new kubernetes topology manager policy options settings.
fn run() -> Result<()> {
    migrate(AddSettingsMigration(&[
        "settings.kubernetes.topology-manager-policy-options",
        "settings.kubernetes.topology-manager-policy-options.prefer-closest-numa-nodes",
        "settings.kubernetes.topology-manager-policy-options.max-allowable-numa-nodes",
    ]))
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        process::exit(1);
    }
}
