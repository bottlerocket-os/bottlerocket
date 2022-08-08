use crate::error::{self, Result};
use serde::Deserialize;
use snafu::ResultExt;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    pub(crate) should_wait: bool,
    pub(crate) marker_path: String,
}

impl Config {
    /// Parses configuration file at passed in path and returns struct containing settings value.
    pub(crate) fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let string_parse = fs::read_to_string(path).context(error::ConfigReadSnafu { path })?;
        let config: Config =
            toml::from_str(&string_parse).context(error::ConfigParseSnafu { path })?;
        Ok(config)
    }
}

#[cfg(test)]
mod config_test {
    use crate::config::Config;
    use tempfile::TempDir;

    // Example config file where user sets autoscaling.should-wait to false.
    const FALSE_SHOULD_WAIT: &str = r#"
    should_wait = false
    marker_path = "/var/lib/bottlerocket/hachiko.ran"
    "#;

    // Example config file where user sets autoscaling.should-wait to true.
    const TRUE_SHOULD_WAIT: &str = r#"
    should_wait = true
    marker_path = "/var/lib/bottlerocket/hachiko.ran"
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
