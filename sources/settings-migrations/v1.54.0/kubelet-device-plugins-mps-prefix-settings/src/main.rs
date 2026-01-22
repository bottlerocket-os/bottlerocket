use migration_helpers::common_migrations::AddPrefixesMigration;
use migration_helpers::{migrate, Result};
use std::process;

/// We added new settings for configuring NVIDIA MPS (Multi-Process Service)
/// GPU sharing in the device plugin, remove the prefix for these settings
fn run() -> Result<()> {
    migrate(AddPrefixesMigration(vec![
        "settings.kubelet-device-plugins.nvidia.mps",
    ]))
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        process::exit(1);
    }
}
