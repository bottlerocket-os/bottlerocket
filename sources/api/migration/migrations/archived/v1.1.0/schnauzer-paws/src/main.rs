use migration_helpers::{migrate, Migration, MigrationData, Result};
use std::process;

const SETTING: &str = "settings.kubernetes.pod-infra-container-image";
const OLD_SETTING_GENERATOR: &str = "pluto pod-infra-container-image";
const NEW_SETTING_GENERATOR: &str = "schnauzer settings.kubernetes.pod-infra-container-image";
const NEW_TEMPLATE: &str =
    "{{ pause-prefix settings.aws.region }}/eks/pause-{{ goarch os.arch }}:3.1";

/// We moved from using pluto to schnauzer for generating the pause container image URL, since it
/// lets us reuse the existing region and arch settings, improving reliability and allowing for
/// testing new regions through settings overrides.
pub struct SchnauzerPaws;

impl Migration for SchnauzerPaws {
    fn forward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        // Check if we have this setting at all.
        if let Some(metadata) = input.metadata.get_mut(SETTING) {
            if let Some(metadata_value) = metadata.get_mut("setting-generator") {
                // Make sure the value is what we expect.
                match metadata_value {
                    serde_json::Value::String(string) => {
                        if string == OLD_SETTING_GENERATOR {
                            // Happy path.  Update the generator.
                            *metadata_value = NEW_SETTING_GENERATOR.into();
                            println!(
                                "Changed setting-generator for '{}' from {:?} to {:?} on upgrade",
                                SETTING, OLD_SETTING_GENERATOR, NEW_SETTING_GENERATOR
                            );

                            // Set the associated template.  We didn't have a template for this
                            // setting before, and metadata can't be changed by the user, so we can
                            // just set it.
                            metadata.insert("template".to_string(), NEW_TEMPLATE.into());
                            println!(
                                "Set 'template' metadata on '{}' to '{}'",
                                SETTING, NEW_TEMPLATE
                            );
                        } else {
                            println!(
                                "setting-generator for '{}' is not set to {:?}, leaving alone",
                                SETTING, OLD_SETTING_GENERATOR
                            );
                        }
                    }
                    _ => {
                        println!(
                            "setting-generator for '{}' is set to non-string value '{}'; SchnauzerPaws only handles strings",
                            SETTING, metadata_value
                        );
                    }
                }
            } else {
                println!("Found no setting-generator for '{}'", SETTING);
            }
        } else {
            println!("Found no metadata for '{}'", SETTING);
        }

        Ok(input)
    }

    fn backward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        // Check if we have this setting at all.
        if let Some(metadata) = input.metadata.get_mut(SETTING) {
            if let Some(metadata_value) = metadata.get_mut("setting-generator") {
                // Make sure the value is what we expect.
                match metadata_value {
                    serde_json::Value::String(string) => {
                        if string == NEW_SETTING_GENERATOR {
                            // Happy path.  Update the generator.
                            *metadata_value = OLD_SETTING_GENERATOR.into();
                            println!(
                                "Changed setting-generator for '{}' from {:?} to {:?} on downgrade",
                                SETTING, NEW_SETTING_GENERATOR, OLD_SETTING_GENERATOR
                            );

                            // Remove the associated template.  We didn't have a template for this
                            // setting before, and metadata can't be changed by the user, so we can
                            // just remove it.
                            if let Some(metadata_value) = metadata.remove("template") {
                                println!(
                                    "Removed 'template' metadata on '{}', which was set to '{}'",
                                    SETTING, metadata_value
                                );
                            } else {
                                println!(
                                    "Found no 'template' metadata to remove on setting '{}'",
                                    SETTING
                                );
                            }
                        } else {
                            println!(
                                "setting-generator for '{}' is not set to {:?}, leaving alone",
                                SETTING, NEW_SETTING_GENERATOR
                            );
                        }
                    }
                    _ => {
                        println!(
                            "setting-generator for '{}' is set to non-string value '{}'; SchnauzerPaws only handles strings",
                            SETTING, metadata_value
                        );
                    }
                }
            } else {
                println!("Found no setting-generator for '{}'", SETTING);
            }
        } else {
            println!("Found no metadata for '{}'", SETTING);
        }

        Ok(input)
    }
}

fn run() -> Result<()> {
    migrate(SchnauzerPaws)
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
