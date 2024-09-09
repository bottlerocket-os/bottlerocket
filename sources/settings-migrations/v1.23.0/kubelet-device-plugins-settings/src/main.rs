use migration_helpers::common_migrations::AddPrefixesMigration;
use migration_helpers::{migrate, Result};
use std::process;

/// We added a new setting for configuring the NVIDIA k8s device plugin.
fn run() -> Result<()> {
    migrate(AddPrefixesMigration(vec![
        "settings.kubelet-device-plugins",
        "services.nvidia-k8s-device-plugin",
        "configuration-files.nvidia-k8s-device-plugin-conf",
        "configuration-files.nvidia-k8s-device-plugin-exec-start-conf",
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
