use migration_helpers::common_migrations::AddSettingsMigration;
use migration_helpers::{migrate, Result};
use std::process;

/// We added new settings for configuring the NVIDIA k8s device plugin.
fn run() -> Result<()> {
    migrate(AddSettingsMigration(&[
        "settings.kubelet-device-plugins.nvidia.device-sharing-strategy",
        "settings.kubelet-device-plugins.nvidia.time-slicing.replicas",
        "settings.kubelet-device-plugins.nvidia.time-slicing.rename-by-default",
        "settings.kubelet-device-plugins.nvidia.time-slicing.fail-requests-greater-than-one",
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
