#![deny(rust_2018_idioms)]

use migration_helpers::{migrate, Migration, MigrationData, Result};
use std::process;

/// We changed the path to our containerd configuration template so that we could support image
/// variants with different configs.  We need to update old images to the new path, and on
/// downgrade, new images to the old path.
struct ContainerdConfigPath;

const SETTING: &str = "configuration-files.containerd-config-toml.template-path";
// Old version with no variant
const DEFAULT_CTRD_CONFIG_OLD: &str = "/usr/share/templates/containerd-config-toml";
// Any users coming from old versions would be using the aws-k8s variant because no other existed :)
const DEFAULT_CTRD_CONFIG_NEW: &str = "/usr/share/templates/containerd-config-toml_aws-k8s";

impl Migration for ContainerdConfigPath {
    fn forward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        if let Some(cfg_path) = input.data.get_mut(SETTING) {
            if cfg_path.as_str() == Some(DEFAULT_CTRD_CONFIG_OLD) {
                *cfg_path = serde_json::Value::String(DEFAULT_CTRD_CONFIG_NEW.to_string());
            }
        }
        Ok(input)
    }

    fn backward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        if let Some(cfg_path) = input.data.get_mut(SETTING) {
            if cfg_path.as_str() == Some(DEFAULT_CTRD_CONFIG_NEW) {
                *cfg_path = serde_json::Value::String(DEFAULT_CTRD_CONFIG_OLD.to_string());
            }
        }
        Ok(input)
    }
}

fn run() -> Result<()> {
    migrate(ContainerdConfigPath)
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
