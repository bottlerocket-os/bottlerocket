/*!
# Introduction

early-boot-config-provider defines the interface of the user data provider binaries used by early-boot-config.

User data provider binaries can also be easily created using the UserDataProvider trait and logging functions defined by this crate.
*/

#[macro_use]
extern crate log;

pub mod compression;
pub mod provider;
pub mod settings;

/// The environment variable used to set log level for env_logger
pub const LOG_LEVEL_ENV_VAR: &str = "EARLY_BOOT_CONFIG_LOG_LEVEL";
