use log::debug;
use std::{process};
use std::fs;
use std::path::{Path};
use serde::{Serialize};
use snafu::{ResultExt};

// FIXME Get from configuration in the future
const DEFAULT_API_SOCKET: &str = "/run/api.sock";

const DEFAULT_ECS_CONFIG_PATH: &str = "/etc/ecs/ecs.config.json";


#[derive(Serialize, Debug)]
#[serde(rename_all="PascalCase")]
struct ECSConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    cluster: Option<String>,
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

fn run() -> Result<()> {
    // Get all settings values for config file templates
    debug!("Requesting settings values");
    let settings = schnauzer::get_settings(&DEFAULT_API_SOCKET).context(error::Settings)?;

    debug!("settings = {:#?}", settings.settings);
    let config = ECSConfig{ cluster: settings.settings.and_then(|s| s.ecs).and_then(|s| s.cluster)};
    let serialized = serde_json::to_string(&config).context(error::Serialization)?;
    debug!("serialized = {}", serialized);

    write_to_disk(DEFAULT_ECS_CONFIG_PATH, serialized).context(error::FS{path:DEFAULT_ECS_CONFIG_PATH})?;
    Ok(())
}

/// Writes the rendered data at the proper location
fn write_to_disk<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> std::io::Result<()> {
    if let Some(dirname) = path.as_ref().parent() {
        fs::create_dir_all(dirname)?;
    };

    fs::write(path, contents)
}

type Result<T> = std::result::Result<T, error::Error>;

mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(super) enum Error {
        #[snafu(display("Failed to read settings: {}", source))]
        Settings{
            source: schnauzer::Error
        },

        #[snafu(display("Failed to serialize ECS config: {}", source))]
        Serialization{
            source: serde_json::error::Error
        },

        #[snafu(display("Filesystem operation for path {} failed: {}", path, source))]
        FS{
            path: &'static str,
            source: std::io::Error
        }
    }
}
