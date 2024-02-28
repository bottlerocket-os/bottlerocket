#[macro_use]
extern crate log;

pub mod provider;

/// The environment variable used to set log level for env_logger
pub const LOG_LEVEL_ENV_VAR: &str = "EARLY_BOOT_CONFIG_LOG_LEVEL";
