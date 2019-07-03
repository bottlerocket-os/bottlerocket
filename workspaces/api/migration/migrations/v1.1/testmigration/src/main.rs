use migration_helpers::{migrate, Migration, MigrationData, Result};

/// Example migration that prepends "New" to the system time zone.
struct TestMigration;

impl Migration for TestMigration {
    fn forward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        if let Some(timezone) = input.data.get_mut("settings.timezone") {
            if let Some(tz_str) = timezone.as_str() {
                // Some modification to the value
                *timezone = ("New".to_string() + tz_str).into();
            } else {
                // We can handle some easy error conditions using defaults
                *timezone = "Default".into();
            }
        }
        Ok(input)
    }

    fn backward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        if let Some(timezone) = input.data.get_mut("settings.timezone") {
            if let Some(tz_str) = timezone.as_str() {
                // Revert modification if we find it
                if tz_str.starts_with("New") {
                    *timezone = tz_str[3..].into();
                }
            } else {
                // We can handle some easy error conditions using defaults
                *timezone = "Default".into();
            }
        }
        Ok(input)
    }
}

fn main() -> Result<()> {
    migrate(TestMigration)
}
