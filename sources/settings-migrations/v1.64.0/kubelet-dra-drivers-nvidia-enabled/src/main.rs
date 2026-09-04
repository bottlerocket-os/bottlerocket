use migration_helpers::common_migrations::AddSettingsMigration;
use migration_helpers::{migrate, Result};
use std::process;

/// We added a new setting for enabling/disabling the NVIDIA DRA driver
/// (gpu-kubelet-plugin) host service:
/// - settings.kubelet-dra-drivers.nvidia.enabled
///
/// Forward (upgrade): no-op. The `enabled` field defaults to `None` (the
/// service treats unset as disabled), so existing data is compatible.
/// Backward (downgrade): removes settings.kubelet-dra-drivers.nvidia.enabled from the
/// datastore so older versions don't encounter an unknown field.
fn run() -> Result<()> {
    migrate(AddSettingsMigration(&[
        "settings.kubelet-dra-drivers.nvidia.enabled",
    ]))
}

// Returning a Result from main makes it print a Debug representation of the error, but with Snafu
// we have nice Display representations of the error, so we wrap "main" (run) and print any error.
// https://github.com/shepmaster/snafu/issues/110
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        process::exit(1);
    }
}
