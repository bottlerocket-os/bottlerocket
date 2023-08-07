use migration_helpers::common_migrations::NoOpMigration;
use migration_helpers::{migrate, Migration, MigrationData, Result};
use serde_json::Value;
use std::process;

const HARD_RESOURCE_LIMIT_SETTING_NAME: &str =
    "settings.oci-defaults.resource-limits.max-open-files.hard-limit";
const SOFT_RESOURCE_LIMIT_SETTING_NAME: &str =
    "settings.oci-defaults.resource-limits.max-open-files.soft-limit";

/// This migration changes the hard and soft limit for rlimit_nofile to u32 from i64 on downgrade.
/// There is no need of migration on upgrade as u32 will automatically change to i64
pub struct ChangeMaxOpenFileResourceLimitType;

fn convert_to_u32(value: &mut Value) {
    if !value.is_i64() {
        return;
    }
    let v: i64 = serde_json::from_value(value.clone()).unwrap();
    let s = match v {
        -1 => u32::MAX,
        v if v > u32::MAX as i64 => u32::MAX,
        _ => v as u32,
    };

    *value = Value::Number(s.into());
}

impl Migration for ChangeMaxOpenFileResourceLimitType {
    /// On upgrade there is nothing to do (see above).
    fn forward(&mut self, input: MigrationData) -> Result<MigrationData> {
        Ok(input)
    }

    /// On downgrade, if the value is an i64 integer, we need to convert it to a u32.
    ///
    /// Note that this potentially causes data loss, if current value of the setting
    /// is -1 or higher than u_32::MAX we will set it to max possible value i.e. u32::MAX.
    fn backward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        if let Some(v) = input.data.get_mut(HARD_RESOURCE_LIMIT_SETTING_NAME) {
            convert_to_u32(v);
        }
        if let Some(v) = input.data.get_mut(SOFT_RESOURCE_LIMIT_SETTING_NAME) {
            convert_to_u32(v);
        }
        Ok(input)
    }
}

fn run() -> Result<()> {
    if cfg!(variant_runtime = "k8s") {
        migrate(ChangeMaxOpenFileResourceLimitType)?
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
