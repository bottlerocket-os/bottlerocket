/// This module contains utilities for waiting until the instance on warm pool to be
/// marked as InService.
use crate::warmpool::error::{self, Result};
use argh::FromArgs;
use imdsclient::ImdsClient;
use serde::Deserialize;
use snafu::ResultExt;
use std::fs;
use std::path::Path;
use tokio::time::{sleep, Duration};

// Path to config file containing host's autoscaling.should-wait setting
const CONFIG_PATH: &str = "/etc/warm-pool-wait.toml";
const FETCH_LIFECYCLE_INTERVAL_SECS: Duration = Duration::from_secs(5);

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "warm-pool-wait")]
/// Wait until the instance reaches the `InService` state
pub(crate) struct WarmPoolWait {}

impl WarmPoolWait {
    pub(crate) async fn run(self) -> Result<()> {
        let config_parse = get_config().await?;

        let should_wait_value = config_parse.should_wait;
        let marker_file_path = config_parse.marker_path;

        println!("autoscaling.should-wait value is {}", should_wait_value);
        if should_wait_value {
            wait_until_inservice().await?;
        }

        fs::write(&marker_file_path, "").unwrap_or_else(|e| {
            log::warn!("Failed to create marker file '{}', warm-pool-wait service may unexpectedly run again: '{}'",
            &marker_file_path, e);
        });
        println!("Marker file path is {}", marker_file_path);

        Ok(())
    }
}

/// Continuously fetches lifecycle state of the host.
/// Returns from function when host is "InService"
async fn wait_until_inservice() -> Result<()> {
    let mut client = ImdsClient::new();

    loop {
        let lifecycle_state = client
            .fetch_autoscaling_lifecycle_state()
            .await
            .context(error::ImdsRequestSnafu)?;

        if let Some(lifecycle_state) = lifecycle_state {
            if lifecycle_state == "InService" {
                println!("Lifecycle state is {} ... exiting.", lifecycle_state);
                return Ok(());
            }
        }

        sleep(FETCH_LIFECYCLE_INTERVAL_SECS).await;
        continue;
    }
}

/// Parses config file for autoscaling.should-wait setting.
async fn get_config() -> Result<Config> {
    Config::from_file(CONFIG_PATH)
}

#[derive(Debug, Deserialize)]
struct Config {
    should_wait: bool,
    marker_path: String,
}

impl Config {
    /// Parses configuration file at passed in path and returns struct containing settings value.
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let string_parse = fs::read_to_string(path).context(error::ConfigReadSnafu { path })?;
        let config: Config =
            toml::from_str(&string_parse).context(error::ConfigParseSnafu { path })?;
        Ok(config)
    }
}

#[cfg(test)]
mod config_test {
    use super::Config;
    use tempfile::TempDir;

    // Example config file where user sets autoscaling.should-wait to false.
    const FALSE_SHOULD_WAIT: &str = r#"
    should_wait = false
    marker_path = "/var/lib/bottlerocket/warm-pool-wait.ran"
    "#;

    // Example config file where user sets autoscaling.should-wait to true.
    const TRUE_SHOULD_WAIT: &str = r#"
    should_wait = true
    marker_path = "/var/lib/bottlerocket/warm-pool-wait.ran"
    "#;

    #[test]
    fn false_should_wait_test() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, FALSE_SHOULD_WAIT).unwrap();
        let config = Config::from_file(&path).unwrap();
        assert!(!config.should_wait);
    }

    #[test]
    fn true_should_wait_test() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, TRUE_SHOULD_WAIT).unwrap();
        let config = Config::from_file(&path).unwrap();
        assert!(config.should_wait);
    }
}
