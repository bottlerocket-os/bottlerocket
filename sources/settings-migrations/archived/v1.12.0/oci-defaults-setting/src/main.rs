use migration_helpers::common_migrations::{AddPrefixesMigration, NoOpMigration};
use migration_helpers::{migrate, Result};
use std::process;

/// We added new settings for configuring the default OCI runtime spec,
/// `settings.oci-defaults`, which will initially contain
/// `settings.oci-defaults.capabilities` and
/// `settings.oci-defaults.resource-limits`
fn run() -> Result<()> {
    if cfg!(variant_runtime = "k8s") {
        migrate(AddPrefixesMigration(vec![
            "settings.oci-defaults",
            "services.oci-defaults",
            "configuration-files.oci-defaults",
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
