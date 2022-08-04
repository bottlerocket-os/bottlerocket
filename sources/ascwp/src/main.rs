mod config;
mod error;
use crate::config::Config;
use crate::error::Result;
use imdsclient::ImdsClient;
use snafu::{OptionExt, ResultExt};
use std::fs;
use log::{error, info, warn};
use std::time::Duration;
use async_std::task;



//Path to config file containing host's eks_autoscaling.should-wait setting
const CONFIG_PATH: &str = "/etc/ascwp.toml";
//Marker file that is created after first run. We only want this program to run once.
const MARKER_FILE: &str = "/var/lib/bottlerocket/ascwp.ran";

//Uses an imdsclient function to fetch the lifecyclestate of the host
async fn get_lifecycle_state(client: &mut ImdsClient) -> Result<String> {
    client
        .fetch_lifecycle_state()
        .await
        .context(error::ImdsRequestSnafu)?
        .context(error::ImdsNoneSnafu {
            what: "instance-id",
        })
}

//Continously fetches lifecycle state of the host. Returns when host is "InService"
async fn wait_until_inservice() -> Result<()> {
    let mut client = ImdsClient::new();
    let mut lifecycle_state = get_lifecycle_state(&mut client).await?;
    while lifecycle_state.ne("InService") {
        lifecycle_state = get_lifecycle_state(&mut client).await?;
        task::sleep(Duration::from_secs(5)).await;
    }
    Ok(())
}

//Parses config file for should wait setting.
async fn get_config() -> Result<Config> {
    Config::from_file(CONFIG_PATH)
}

//If eks_autoscaling.should-wait is true, calls waiting function. Writes marker file to prevent service from unexpectedly rerunning.
#[tokio::main]
async fn main() {
    let should_wait_value = get_config().await.unwrap().should_wait;
    if should_wait_value{
        wait_until_inservice().await.unwrap();
    }
    fs::write(MARKER_FILE, "").unwrap_or_else(|e| {
        warn!(
            "Failed to create marker file '{}', ASCWP service may unexpectedly run again: '{}'",
            MARKER_FILE, e
        )
    });
}
