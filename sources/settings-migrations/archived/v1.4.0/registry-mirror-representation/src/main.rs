use migration_helpers::{migrate, Migration, MigrationData, Result};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::process;

const MIRRORS_SETTING_NAME: &'static str = "settings.container-registry.mirrors";
const DATASTORE_KEY_SEPARATOR: char = '.';

/// This migration changes the model type of `settings.container-registry.mirrors` from `HashMap<SingleLineString, Vec<Url>>`
/// to `Vec<RegistryMirrors>` on upgrade and vice-versa on downgrades.
pub struct ChangeRegistryMirrorsType;

// Snapshot of the `datastore::Key::valid_character` method in Bottlerocket version 1.3.0
//
// Determines whether a character is acceptable within a segment of a key name.  This is
// separate from quoting; if a character isn't valid, it isn't valid quoted, either.
fn valid_character(c: char) -> bool {
    match c {
        'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' | '/' => true,
        _ => false,
    }
}

impl Migration for ChangeRegistryMirrorsType {
    /// Newer versions store `settings.container-registry.mirrors` as `Vec<RegistryMirrors>`.
    /// Need to convert from `HashMap<SingleLineString, Vec<Url>>`.
    fn forward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        let mirrors: HashMap<_, _> = input
            .data
            .iter()
            .filter(|&(k, _)| k.starts_with(format!("{}.", MIRRORS_SETTING_NAME).as_str()))
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect();
        let mut new_mirrors = Vec::new();
        for (setting, endpoint) in mirrors {
            // Get the registry name from the settings name. Trim any quotes the settings name might have.
            let registry = setting
                .strip_prefix(&format!("{}.", MIRRORS_SETTING_NAME))
                .unwrap_or_default()
                .trim_matches('"');
            let mut registry_mirrors = Map::new();
            registry_mirrors.insert("registry".to_string(), Value::String(registry.to_string()));
            registry_mirrors.insert("endpoint".to_string(), endpoint.to_owned());
            new_mirrors.push(Value::Object(registry_mirrors));
            if let Some(data) = input.data.remove(&setting) {
                println!("Removed setting '{}', which was set to '{}'", setting, data);
            }
        }
        let data = Value::Array(new_mirrors);
        println!(
            "Creating new setting '{}', which is set to '{}'",
            MIRRORS_SETTING_NAME, &data
        );
        input.data.insert(MIRRORS_SETTING_NAME.to_string(), data);
        Ok(input)
    }

    /// Older versions store `settings.container-registry.mirrors` as `HashMap<SingleLineString, Vec<Url>>`.
    /// Need to convert from `Vec<RegistryMirrors>`.
    fn backward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        if let Some(data) = input.data.get_mut(MIRRORS_SETTING_NAME).cloned() {
            match data {
                Value::Array(arr) => {
                    if let Some(data) = input.data.remove(MIRRORS_SETTING_NAME) {
                        println!(
                            "Removed setting '{}', which was set to '{}'",
                            MIRRORS_SETTING_NAME, data
                        );
                    }
                    for obj in arr {
                        if let Some(obj) = obj.as_object() {
                            if let (Some(registry), Some(endpoint)) = (
                                obj.get("registry").and_then(|s| s.as_str()),
                                obj.get("endpoint"),
                            ) {
                                // Ensure the registry contains valid datastore key characters.
                                // If we encounter any invalid key characters, we skip writing out
                                // the setting key to prevent breakage of the datastore.
                                if registry
                                    .chars()
                                    .all(|c| valid_character(c) || c == DATASTORE_KEY_SEPARATOR)
                                {
                                    let setting_name =
                                        format!(r#"{}."{}""#, MIRRORS_SETTING_NAME, registry);
                                    println!(
                                        "Creating new setting '{}', which is set to '{}'",
                                        setting_name, &endpoint
                                    );
                                    input.data.insert(setting_name, endpoint.to_owned());
                                } else {
                                    eprintln!(
                                        "Container registry '{}' contains invalid datastore key character(s). Skipping to prevent datastore breakage...",
                                        registry
                                    );
                                }
                            }
                        } else {
                            println!(
                                "'{}' contains non-JSON Object value: '{}'.",
                                MIRRORS_SETTING_NAME, obj
                            );
                        }
                    }
                }
                _ => {
                    println!(
                        "'{}' is not a JSON Array value: '{}'.",
                        MIRRORS_SETTING_NAME, data
                    );
                }
            }
        } else {
            println!("Didn't find setting '{}'", MIRRORS_SETTING_NAME);
        }
        Ok(input)
    }
}

fn run() -> Result<()> {
    migrate(ChangeRegistryMirrorsType)
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
