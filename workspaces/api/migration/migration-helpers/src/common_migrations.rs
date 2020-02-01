use crate::{Migration, MigrationData, Result};

/// We use this migration when we add a setting and want to make sure it's removed before we go
/// back to old versions that don't understand it.
pub struct AddSettingMigration(pub &'static str);

impl Migration for AddSettingMigration {
    /// New versions must either have a default for the setting or generate it; we don't need to
    /// do anything.
    fn forward(&mut self, input: MigrationData) -> Result<MigrationData> {
        println!("AddSettingMigration({}) has no work to do on upgrade.", self.0);
        Ok(input)
    }

    /// Older versions don't know about the setting; we remove it so that old versions don't see
    /// it and fail deserialization.  (The setting must be defaulted or generated in new versions,
    /// and safe to remove.)
    fn backward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        if let Some(data) = input.data.remove(self.0) {
            println!("Removed {}, which was set to '{}'", self.0, data);
        } else {
            println!("Found no {} to remove", self.0);
        }
        Ok(input)
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// We use this migration when we remove a setting from the model, so the new version doesn't see
/// it and error.
pub struct RemoveSettingMigration(String);

impl Migration for RemoveSettingMigration {
    /// Newer versions don't know about the setting; we remove it so that new versions don't see
    /// it and fail deserialization.  (The setting must be defaulted or generated in old versions,
    /// and safe to remove.)
    fn forward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        if let Some(data) = input.data.remove(&self.0) {
            println!("Removed {}, which was set to '{}'", self.0, data);
        } else {
            println!("Found no {} to remove", self.0);
        }
        Ok(input)
    }

    /// Old versions must either have a default for the setting or generate it; we don't need to
    /// do anything.
    fn backward(&mut self, input: MigrationData) -> Result<MigrationData> {
        println!("RemoveSettingMigration({}) has no work to do on downgrade.", self.0);
        Ok(input)
    }
}
