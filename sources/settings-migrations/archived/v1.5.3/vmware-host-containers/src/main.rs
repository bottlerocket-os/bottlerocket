use migration_helpers::{migrate, Migration, MigrationData, Result};
use std::process;

const ADMIN_CONTAINER_SOURCE_SETTING_NAME: &str = "settings.host-containers.admin.source";
const ADMIN_CONTAINER_IMAGE_REPOSITORY: &str = "public.ecr.aws/bottlerocket/bottlerocket-admin";
const PREVIOUS_ADMIN_CONTAINER_VERSIONS: &[&str] = &["v0.7.0", "v0.7.1", "v0.7.2"];
const TARGET_ADMIN_CONTAINER_VERSION: &str = "v0.7.3";

const CONTROL_CONTAINER_SOURCE_SETTING_NAME: &str = "settings.host-containers.control.source";
const CONTROL_CONTAINER_IMAGE_REPOSITORY: &str = "public.ecr.aws/bottlerocket/bottlerocket-control";
const PREVIOUS_CONTROL_CONTAINER_VERSIONS: &[&str] = &["v0.5.0", "v0.5.1", "v0.5.2", "v0.5.3"];
const TARGET_CONTROL_CONTAINER_VERSION: &str = "v0.5.4";

pub struct VmwareHostContainerVersions;

impl Migration for VmwareHostContainerVersions {
    fn forward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        // For admin container
        if let Some(data) = input.data.get_mut(ADMIN_CONTAINER_SOURCE_SETTING_NAME) {
            match data {
                serde_json::Value::String(source) => {
                    for ver in PREVIOUS_ADMIN_CONTAINER_VERSIONS {
                        let prev_source = format!("{}:{}", ADMIN_CONTAINER_IMAGE_REPOSITORY, ver);
                        if *source == prev_source {
                            *source = format!(
                                "{}:{}",
                                ADMIN_CONTAINER_IMAGE_REPOSITORY, TARGET_ADMIN_CONTAINER_VERSION
                            );
                            println!(
                                "Changed value of '{}' from '{}' to '{}' on upgrade",
                                ADMIN_CONTAINER_SOURCE_SETTING_NAME, prev_source, source
                            );
                            break;
                        }
                    }
                }
                _ => {
                    println!(
                        "'{}' is set to non-string value '{}'",
                        ADMIN_CONTAINER_SOURCE_SETTING_NAME, data
                    );
                }
            }
        } else {
            println!(
                "Found no '{}' to change on upgrade",
                ADMIN_CONTAINER_SOURCE_SETTING_NAME
            );
        }

        // For control container
        if let Some(data) = input.data.get_mut(CONTROL_CONTAINER_SOURCE_SETTING_NAME) {
            match data {
                serde_json::Value::String(source) => {
                    for ver in PREVIOUS_CONTROL_CONTAINER_VERSIONS {
                        let prev_source = format!("{}:{}", CONTROL_CONTAINER_IMAGE_REPOSITORY, ver);
                        if *source == prev_source {
                            *source = format!(
                                "{}:{}",
                                CONTROL_CONTAINER_IMAGE_REPOSITORY,
                                TARGET_CONTROL_CONTAINER_VERSION
                            );
                            println!(
                                "Changed value of '{}' from '{}' to '{}' on upgrade",
                                CONTROL_CONTAINER_SOURCE_SETTING_NAME, prev_source, source
                            );
                            break;
                        }
                    }
                }
                _ => {
                    println!(
                        "'{}' is set to non-string value '{}'",
                        CONTROL_CONTAINER_SOURCE_SETTING_NAME, data
                    );
                }
            }
        } else {
            println!(
                "Found no '{}' to change on upgrade",
                CONTROL_CONTAINER_SOURCE_SETTING_NAME
            );
        }

        Ok(input)
    }

    fn backward(&mut self, input: MigrationData) -> Result<MigrationData> {
        // It's unclear what version of the host-containers we should downgrade to since it could
        // be any of the older host-container versions.
        // We can just stay on the latest host-container version since there are no breaking changes.
        println!("Vmware host-container versions migration has no work to do on downgrade");
        Ok(input)
    }
}

fn run() -> Result<()> {
    migrate(VmwareHostContainerVersions)
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
