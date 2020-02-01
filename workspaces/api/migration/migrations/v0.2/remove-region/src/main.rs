#![deny(rust_2018_idioms)]

use migration_helpers::{migrate, Migration, MigrationData, Result};
use std::process;

/// We added a generated setting, "settings.aws.region", and want to make sure it's removed before
/// we go back to old versions that don't understand it.
struct RemoveRegionMigration;

impl Migration for RemoveRegionMigration {
    /// New versions will generate the setting, we don't need to do anything.
    fn forward(&mut self, input: MigrationData) -> Result<MigrationData> {
        println!("RemoveRegionMigration has no work to do on upgrade.");
        Ok(input)
    }

    /// Older versions don't know about region; we remove it so that old versions don't see it and
    /// fail deserialization.  (The setting is generated in new versions, and safe to remove.)
    fn backward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        if let Some(region) = input.data.remove("settings.aws.region") {
            println!("Removed settings.aws.region, which was set to '{}'", region);
        } else {
            println!("Found no settings.aws.region to remove");
        }
        Ok(input)
    }
}

fn run() -> Result<()> {
    migrate(RemoveRegionMigration)
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
