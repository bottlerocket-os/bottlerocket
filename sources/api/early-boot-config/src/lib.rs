#[macro_use]
extern crate log;

mod compression;
pub mod provider;
pub mod settings;

/// The environment variable used to set log level for env_logger
pub const LOG_LEVEL_ENV_VAR: &str = "EARLY_BOOT_CONFIG_LOG_LEVEL";
