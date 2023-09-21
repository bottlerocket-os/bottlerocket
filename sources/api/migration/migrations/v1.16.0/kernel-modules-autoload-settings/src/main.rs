use migration_helpers::{migrate, Migration, MigrationData, Result};
use std::process;

const KMOD_AUTOLOAD_PREFIX: &str = "settings.kernel.modules";
const KMOD_AUTOLOAD_SETTING: &str = "autoload";

/// We added a new autoload setting to the kernel.mudules set of tables. These tables
/// come with a variable name containing the module name. We can hence not just use
/// an `AddSettingsMigration` as these require the full name. We rather need a hybrid
/// of `AddSettingsMigration` and `AddPrefixesMigration` in order to select the correct
/// parts of these variably named tables to remove on downgrade. Similar to the common
/// forms of `Add*Migrations` we do not need to do anything on upgrade.
pub struct AddKmodAutoload;

impl Migration for AddKmodAutoload {
    /// On upgrade there is nothing to do (see above).
    fn forward(&mut self, input: MigrationData) -> Result<MigrationData> {
        Ok(input)
    }

    /// On downgrade, we need to find the `autoload` setting in all tables with
    /// prefix `settings.kernel.modules` and remove them.
    fn backward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        let settings = input
            .data
            .keys()
            .filter(|k| k.starts_with(KMOD_AUTOLOAD_PREFIX))
            .filter(|k| k.ends_with(KMOD_AUTOLOAD_SETTING))
            .cloned()
            .collect::<Vec<_>>();
        for setting in settings {
            if let Some(data) = input.data.remove(&setting) {
                println!("Removed {}, which was set to '{}'", setting, data);
            }
        }
        Ok(input)
    }
}

/// We added `settigns.kernel.modules.<name>.auotload`.
fn run() -> Result<()> {
    migrate(AddKmodAutoload)
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
