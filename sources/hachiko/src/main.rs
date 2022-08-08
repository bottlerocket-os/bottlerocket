mod config;
mod error;
use crate::config::Config;
use crate::error::Result;
use imdsclient::ImdsClient;
use log::LevelFilter;
use log::{info, warn};
use simplelog::{Config as LogConfig, SimpleLogger};
use snafu::{OptionExt, ResultExt};
use std::fs;
use std::process;
use tokio::time::{sleep, Duration};

// Path to config file containing host's autoscaling.should-wait setting
const CONFIG_PATH: &str = "/etc/hachiko.toml";

/// Uses an imdsclient function to fetch the lifecyclestate of the host
async fn get_lifecycle_state(client: &mut ImdsClient) -> Result<String> {
    client
        .fetch_lifecycle_state()
        .await
        .context(error::ImdsRequestSnafu)?
        .context(error::ImdsNoneSnafu {
            what: "instance-id",
        })
}

/// Continuously fetches lifecycle state of the host.
/// Returns from function when host is "InService"
async fn wait_until_inservice() -> Result<()> {
    let mut client = ImdsClient::new();
    let mut lifecycle_state = get_lifecycle_state(&mut client).await?;
    info!("Lifecycle state is {}", lifecycle_state);
    while lifecycle_state.ne("InService") {
        lifecycle_state = get_lifecycle_state(&mut client).await?;
        info!("Lifecycle state is {} ... waiting.", lifecycle_state);
        sleep(Duration::from_secs(5)).await;
    }
    info!("Lifecycle state is {} ... exiting.", lifecycle_state);
    Ok(())
}

/// Parses config file for autoscaling.should-wait setting.
async fn get_config() -> Result<Config> {
    Config::from_file(CONFIG_PATH)
}

async fn run() -> Result<()> {
    SimpleLogger::init(LevelFilter::Info, LogConfig::default()).context(error::LoggerSnafu)?;

    let config_parse = get_config().await?;

    let should_wait_value = config_parse.should_wait;
    let marker_file_path = config_parse.marker_path;
    info!("autoscaling.should-wait value is {}", should_wait_value);
    if should_wait_value {
        wait_until_inservice().await?;
    }
    info!("Marker file path is {}", marker_file_path);
    fs::write(&marker_file_path, "").unwrap_or_else(|e| {
        warn!(
            "Failed to create marker file '{}', Hachiko service may unexpectedly run again: '{}'",
            &marker_file_path, e
        )
    });

    Ok(())
}

/// If autoscaling.should-wait is true, calls waiting function.
/// Writes marker file to prevent service from unexpectedly re-running.
#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{}", e);
        process::exit(1);
    }
}
