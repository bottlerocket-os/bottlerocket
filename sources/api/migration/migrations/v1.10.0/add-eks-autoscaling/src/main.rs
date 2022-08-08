#![deny(rust_2018_idioms)]

use migration_helpers::common_migrations::AddPrefixesMigration;
use migration_helpers::{migrate, Result};
use std::process;

/// We added a new setting prefix for configuring autoscaling in k8s variants.
/// Remove the whole `settings.autoscaling` prefix if we downgrade.
fn run() -> Result<()> {
    if cfg!(variant_runtime = "k8s") {
        migrate(AddPrefixesMigration(vec!["settings.autoscaling"]));
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
