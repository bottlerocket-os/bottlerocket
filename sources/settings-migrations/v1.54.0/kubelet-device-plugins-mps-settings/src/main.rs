use migration_helpers::{migrate, Migration, MigrationData, Result};
use std::process;

const DEVICE_SHARING_STRATEGY_SETTING: &str =
    "settings.kubelet-device-plugins.nvidia.device-sharing-strategy";

pub struct ReplaceDeviceSharingStrategy;

impl Migration for ReplaceDeviceSharingStrategy {
    fn forward(&mut self, input: MigrationData) -> Result<MigrationData> {
        println!("ReplaceDeviceSharingStrategy has no work to do on upgrade.");
        Ok(input)
    }

    fn backward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        if let Some(data) = input.data.get_mut(DEVICE_SHARING_STRATEGY_SETTING) {
            if let serde_json::Value::String(s) = data {
                if s == "mps" {
                    *data = serde_json::Value::String("none".to_string());
                    println!("Changed device-sharing-strategy from 'mps' to 'none' on downgrade.");
                }
            }
        }
        Ok(input)
    }
}

/// We added new enum variant for configuring NVIDIA MPS (Multi-Process Service)
/// GPU sharing in the device plugin.
fn run() -> Result<()> {
    migrate(ReplaceDeviceSharingStrategy)
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        process::exit(1);
    }
}
