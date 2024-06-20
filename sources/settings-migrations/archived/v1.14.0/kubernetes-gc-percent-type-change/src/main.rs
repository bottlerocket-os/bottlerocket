use migration_helpers::{migrate, Migration, MigrationData, Result};
use serde_json::Value;
use std::process;

const GC_HIGH_SETTING: &str = "settings.kubernetes.image-gc-high-threshold-percent";
const GC_LOW_SETTING: &str = "settings.kubernetes.image-gc-low-threshold-percent";

/// We changed these settings so that they can be specified as numbers. Previously they could only
/// be specified as strings, which was confusing since they are numeric. On upgrade we don't need
/// to do anything because a valid string representation will still be accepted. On downgrade, we
/// need to check if the values are represented as numbers, and if so, convert them to strings.
pub struct ChangeK8sGcPercentType;

fn convert_to_string(value: &mut Value) {
    let s = if let Value::Number(n) = value {
        n.to_string()
    } else {
        return;
    };
    *value = Value::String(s);
}

impl Migration for ChangeK8sGcPercentType {
    /// On upgrade there is nothing to do (see above).
    fn forward(&mut self, input: MigrationData) -> Result<MigrationData> {
        Ok(input)
    }

    /// On downgrade, if the value is a number, we need to convert it to a string (see above).
    fn backward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        if let Some(v) = input.data.get_mut(GC_HIGH_SETTING) {
            convert_to_string(v);
        }
        if let Some(v) = input.data.get_mut(GC_LOW_SETTING) {
            convert_to_string(v);
        }
        Ok(input)
    }
}

/// We made changes to `image-gc-low-threshold-percent` and `image-gc-high-threshold-percent`.
fn run() -> Result<()> {
    migrate(ChangeK8sGcPercentType)
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
