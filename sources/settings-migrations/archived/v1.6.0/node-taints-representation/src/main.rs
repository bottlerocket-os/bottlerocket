use migration_helpers::{error, migrate, Migration, MigrationData, Result};
use serde_json::Value;
use snafu::OptionExt;
use std::process;

const NODE_TAINTS_SETTING_NAME: &str = "settings.kubernetes.node-taints";

/// This migration changes the model type of `settings.kubernetes.node-taints` from `HashMap<KubernetesLabelKey, KubernetesTaintValue>`
/// to `HashMap<KubernetesLabelKey, Vec<KubernetesTaintValue>>` on upgrade and vice-versa on downgrades.
pub struct ChangeNodeTaintsType;

impl Migration for ChangeNodeTaintsType {
    /// Newer versions store `settings.kubernetes.node-taints` as `HashMap<KubernetesLabelKey, Vec<KubernetesTaintValue>>`.
    /// Need to convert from `HashMap<KubernetesLabelKey, KubernetesTaintValue>`.
    fn forward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        for (taint_key, taint_val) in input
            .data
            .iter_mut()
            .filter(|&(k, _)| k.starts_with(format!("{}.", NODE_TAINTS_SETTING_NAME).as_str()))
        {
            match taint_val {
                Value::String(taint_val_string) => {
                    let taint_val_array =
                        Value::Array(vec![Value::String(taint_val_string.to_owned())]);
                    println!(
                        "Changing '{}', from '{}' to '{}' on upgrade",
                        taint_key, &taint_val, taint_val_array
                    );
                    *taint_val = taint_val_array;
                }
                _ => {
                    println!(
                        "'{}' is not a JSON string value: '{}'",
                        taint_key, taint_val
                    );
                }
            }
        }
        Ok(input)
    }

    /// Older versions store `settings.kubernetes.node-taints` as `HashMap<KubernetesLabelKey, KubernetesTaintValue>`.
    /// Need to convert from `HashMap<KubernetesLabelKey, Vec<KubernetesTaintValue>>`.
    ///
    /// Note that this potentially causes data loss if there are more than one taint value/effect assigned to a taint key.
    /// Older versions can only map one taint value/effect to a taint key, so we default to choosing the first in the list if there are multiple.
    fn backward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        for (taint_key, taint_val) in input
            .data
            .iter_mut()
            .filter(|&(k, _)| k.starts_with(format!("{}.", NODE_TAINTS_SETTING_NAME).as_str()))
        {
            match taint_val {
                Value::Array(taint_val_array) => {
                    // There should always at least be one value in the sequence
                    let first_taint_val = Value::String(
                        taint_val_array
                            .first()
                            .cloned()
                            .unwrap_or_default()
                            .as_str()
                            .context(error::NonStringSettingDataTypeSnafu {
                                setting: taint_key.to_string(),
                            })?
                            .to_string(),
                    );
                    println!(
                        "Changing '{}', from '{}' to '{}' on downgrade",
                        taint_key, &taint_val, first_taint_val
                    );
                    *taint_val = first_taint_val;
                }
                _ => {
                    println!("'{}' is not a JSON Array value: '{}'", taint_key, taint_val);
                }
            }
        }
        Ok(input)
    }
}

fn run() -> Result<()> {
    migrate(ChangeNodeTaintsType)
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
