use crate::error::{self, Result};
use serde::Deserialize;
use snafu::ResultExt;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    pub(crate) should_signal: bool,
    pub(crate) stack_name: String,
    pub(crate) logical_resource_id: String,
}

impl Config {
    pub(crate) fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let s = fs::read_to_string(path).context(error::ConfigReadSnafu { path })?;
        toml::from_str(&s).context(error::ConfigParseSnafu { path })
    }
}
