use log::{debug, error};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use serde::{Serialize};

// FIXME Get from configuration in the future
const DEFAULT_API_SOCKET: &str = "/run/api.sock";

#[derive(Serialize, Debug)]
#[serde(rename_all="PascalCase")]
struct ECSConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    cluster: Option<String>,
}

fn main() {
    // Get all settings values for config file templates
    debug!("Requesting settings values");
    let settings = match schnauzer::get_settings(&DEFAULT_API_SOCKET) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to read settings: {}", e);
            return
        }
    };

    debug!("settings = {:#?}", settings.settings);
    let config = ECSConfig{ cluster: settings.settings.and_then(|s| s.ecs).and_then(|s| s.cluster)};
    let serialized = serde_json::to_string(&config).unwrap();
    debug!("serialized = {}", serialized);

    let config_path = PathBuf::from("/etc/ecs/ecs.config.json");
    match write_to_disk(config_path, serialized) {
        Some(e) => {
            error!("Error! {}", e)
            // TODO exit
        }
        _ => {}
    }
}

/// Writes the rendered data at the proper location
fn write_to_disk<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Option<io::Error> {
    if let Some(dirname) = path.as_ref().parent() {
        let result = fs::create_dir_all(dirname);
        match result {
            Err(e) => {
                return Some(e)
            }
            _ => {}
        };
    };

    return match fs::write(path, contents) {
        Err(e) => {
            Some(e)
        }
        _ => {
            None
        }
    }
}