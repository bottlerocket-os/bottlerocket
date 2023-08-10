use migration_helpers::common_migrations::{AddPrefixesMigration, NoOpMigration};
use migration_helpers::{migrate, Result};
use std::process;

/// We added new resource limit settings for configuring the default OCI runtime spec.
fn run() -> Result<()> {
    if cfg!(variant_runtime = "k8s") {
        migrate(AddPrefixesMigration(vec![
            "settings.oci-defaults.resource-limits.max-address-space",
            "settings.oci-defaults.resource-limits.max-core-file-size",
            "settings.oci-defaults.resource-limits.max-cpu-time",
            "settings.oci-defaults.resource-limits.max-data-size",
            "settings.oci-defaults.resource-limits.max-file-locks",
            "settings.oci-defaults.resource-limits.max-file-size",
            "settings.oci-defaults.resource-limits.max-locked-memory",
            "settings.oci-defaults.resource-limits.max-msgqueue-size",
            "settings.oci-defaults.resource-limits.max-nice-priority",
            "settings.oci-defaults.resource-limits.max-pending-signals",
            "settings.oci-defaults.resource-limits.max-processes",
            "settings.oci-defaults.resource-limits.max-realtime-priority",
            "settings.oci-defaults.resource-limits.max-realtime-timeout",
            "settings.oci-defaults.resource-limits.max-resident-set",
            "settings.oci-defaults.resource-limits.max-stack-size",
        ]))?
    } else {
        migrate(NoOpMigration)?;
    }

    Ok(())
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
