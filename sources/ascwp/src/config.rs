use crate::error::{self, Result};
use serde::Deserialize;
use snafu::ResultExt;
use std::fs;
use std::path::{Path, PathBuf};
const DEFAULT_CONFIG_PATH: &str = "/testConfig.toml";

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    pub(crate) should_wait: bool,
}

impl Config {
    //uses default config path
    pub(crate) fn new() -> Result<Self> {
        Self::from_file(PathBuf::from(DEFAULT_CONFIG_PATH))
    }

    //uses path passed into function
    pub(crate) fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let s = fs::read_to_string(path).context(error::ConfigReadSnafu { path })?;
        let config: Config = toml::from_str(&s).context(error::ConfigParseSnafu { path })?;
        Ok(config)
    }
}

#[cfg(test)]
mod configTest {
    use crate::config::Config;
    use tempfile::TempDir;

    // should wait false
    const false_should_wait: &str = r#"
    should_wait = false
    "#;

    // This is what a config might look like if the user opts out of metrics collection.
    const true_should_wait: &str = r#"
    should_wait = true
    "#;

    #[test]
    fn false_should_wait_test() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, false_should_wait).unwrap();
        let config = Config::from_file(&path).unwrap();
        assert!(!config.should_wait);
    }

    #[test]
    fn true_should_wait_test() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, true_should_wait).unwrap();
        let config = Config::from_file(&path).unwrap();
        assert!(config.should_wait);
    }
}
