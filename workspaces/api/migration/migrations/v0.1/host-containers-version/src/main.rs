#![deny(rust_2018_idioms)]

use migration_helpers::{migrate, Migration, MigrationData, Result};
use std::process;

/// We bumped the versions of the default admin container and the default control container from v0.1 to v0.2
struct HostContainersVersionMigration;
const DEFAULT_ADMIN_CTR_IMG_OLD: &str =
    "328549459982.dkr.ecr.us-west-2.amazonaws.com/thar-admin:v0.1";
const DEFAULT_ADMIN_CTR_IMG_NEW: &str =
    "328549459982.dkr.ecr.us-west-2.amazonaws.com/thar-admin:v0.2";
const DEFAULT_CONTROL_CTR_IMG_OLD: &str =
    "328549459982.dkr.ecr.us-west-2.amazonaws.com/thar-control:v0.1";
const DEFAULT_CONTROL_CTR_IMG_NEW: &str =
    "328549459982.dkr.ecr.us-west-2.amazonaws.com/thar-control:v0.2";

impl Migration for HostContainersVersionMigration {
    fn forward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        if let Some(admin_ctr_source) = input.data.get_mut("settings.host-containers.admin.source")
        {
            // Need to bump versions if the default admin container version source matches its older version
            if admin_ctr_source.as_str() == Some(DEFAULT_ADMIN_CTR_IMG_OLD) {
                *admin_ctr_source =
                    serde_json::Value::String(DEFAULT_ADMIN_CTR_IMG_NEW.to_string());
            }
        }
        if let Some(control_ctr_source) = input
            .data
            .get_mut("settings.host-containers.control.source")
        {
            // Need to bump versions if the default control container version source matches its older version
            if control_ctr_source.as_str() == Some(DEFAULT_CONTROL_CTR_IMG_OLD) {
                *control_ctr_source =
                    serde_json::Value::String(DEFAULT_CONTROL_CTR_IMG_NEW.to_string());
            }
        }
        Ok(input)
    }

    fn backward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        if let Some(admin_ctr_source) = input.data.get_mut("settings.host-containers.admin.source")
        {
            // The default admin container v0.2 image needs OS changes adding persistent host container storage
            if admin_ctr_source.as_str() == Some(DEFAULT_ADMIN_CTR_IMG_NEW) {
                *admin_ctr_source =
                    serde_json::Value::String(DEFAULT_ADMIN_CTR_IMG_OLD.to_string());
            }
        }
        if let Some(control_ctr_source) = input
            .data
            .get_mut("settings.host-containers.control.source")
        {
            if control_ctr_source.as_str() == Some(DEFAULT_CONTROL_CTR_IMG_NEW) {
                *control_ctr_source =
                    serde_json::Value::String(DEFAULT_CONTROL_CTR_IMG_OLD.to_string());
            }
        }
        Ok(input)
    }
}

fn run() -> Result<()> {
    migrate(HostContainersVersionMigration)
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
