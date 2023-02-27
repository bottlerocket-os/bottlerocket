use migration_helpers::{migrate, Migration, MigrationData, Result};
use std::process;

/// This migration removes host-container user data settings when downgrading to versions that
/// don't understand them.
pub struct AddUserDataMigration;

impl Migration for AddUserDataMigration {
    /// There's no user data by default, it's just left empty on upgrade.
    fn forward(&mut self, input: MigrationData) -> Result<MigrationData> {
        println!("AddUserDataMigration has no work to do on upgrade.");
        Ok(input)
    }

    /// Older versions don't know about the user-data settings; we remove them so that old versions
    /// don't see them and fail deserialization.
    fn backward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        for setting in input.data.clone().keys() {
            // We don't currently have structured data available to migrations, and we don't want
            // to re-parse keys.  We know no other keys could match these basic patterns.
            if setting.starts_with("settings.host-containers.") && setting.ends_with(".user-data") {
                if let Some(data) = input.data.remove(setting) {
                    println!("Removed {}, which was set to '{}'", setting, data);
                }
            }
        }
        Ok(input)
    }
}

fn run() -> Result<()> {
    migrate(AddUserDataMigration)
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
