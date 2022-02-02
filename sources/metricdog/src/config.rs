use crate::error::{self, Result};
use serde::Deserialize;
use snafu::ResultExt;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_CONFIG_PATH: &str = "/etc/metricdog.toml";

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    pub(crate) metrics_url: String,
    pub(crate) send_metrics: bool,
    pub(crate) service_checks: Vec<String>,
    pub(crate) region: String,
    pub(crate) seed: u32,
    pub(crate) version_lock: String,
    pub(crate) ignore_waves: bool,
}

impl Config {
    pub(crate) fn new() -> Result<Self> {
        Self::from_file(PathBuf::from(DEFAULT_CONFIG_PATH))
    }

    pub(crate) fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let s = fs::read_to_string(path).context(error::ConfigReadSnafu { path })?;
        let config: Config = toml::from_str(&s).context(error::ConfigParseSnafu { path })?;
        Ok(config)
    }
}

#[cfg(test)]
mod test {
    use crate::config::Config;
    use tempfile::TempDir;

    // This is what most configs will look like.
    const STANDARD_CONFIG: &str = r#"
    metrics_url = "https://example.com"
    send_metrics = true
    service_checks = ["a", "b", "c",]
    region = "us-west-2"
    seed = 1234
    version_lock = "v0.1.2"
    ignore_waves = false
    "#;

    // This is what a config might look like if the user opts out of metrics collection.
    const OPT_OUT_CONFIG: &str = r#"
    metrics_url = ""
    send_metrics = false
    service_checks = ["a", "b", "c",]
    region = "us-west-2"
    seed = 1234
    version_lock = "v0.1.2"
    ignore_waves = false
    "#;

    #[test]
    fn standard_config() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, STANDARD_CONFIG).unwrap();
        let config = Config::from_file(&path).unwrap();
        assert_eq!("https://example.com", config.metrics_url.as_str());
        assert!(config.send_metrics);
        assert_eq!(3, config.service_checks.len());
        assert_eq!("a", config.service_checks.get(0).unwrap());
        assert_eq!("b", config.service_checks.get(1).unwrap());
        assert_eq!("c", config.service_checks.get(2).unwrap());
        assert_eq!("us-west-2", config.region);
        assert_eq!(1234, config.seed);
        assert_eq!("v0.1.2", config.version_lock);
        assert!(!config.ignore_waves);
    }

    #[test]
    fn opt_out_config() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, OPT_OUT_CONFIG).unwrap();
        let config = Config::from_file(&path).unwrap();
        assert_eq!("", config.metrics_url.as_str());
        assert!(!config.send_metrics);
        assert_eq!(3, config.service_checks.len());
        assert_eq!("a", config.service_checks.get(0).unwrap());
        assert_eq!("b", config.service_checks.get(1).unwrap());
        assert_eq!("c", config.service_checks.get(2).unwrap());
        assert_eq!("us-west-2", config.region);
        assert_eq!(1234, config.seed);
        assert_eq!("v0.1.2", config.version_lock);
        assert!(!config.ignore_waves);
    }
}
