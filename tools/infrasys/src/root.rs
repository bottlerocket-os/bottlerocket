use super::{error, KeyRole, Result};
use log::{trace, warn};
use pubsys_config::SigningKeyConfig;
use rusoto_core::Region;
use snafu::{ensure, OptionExt, ResultExt};
use std::collections::HashMap;
use std::fs;
use std::num::NonZeroUsize;
use std::path::Path;
use std::process::Command;

/// The tuftool macro wraps Command to simplify calls to tuftool, adding region functionality.
macro_rules! tuftool {
    ($region:expr, $format_str:expr, $($format_arg:expr),*) => {
        let arg_str = format!($format_str, $($format_arg),*);
        trace!("tuftool arg string: {}", arg_str);
        let args = shell_words::split(&arg_str).context(error::CommandSplit { command: &arg_str })?;
        trace!("tuftool split args: {:#?}", args);

        let status = Command::new("tuftool")
            .args(args)
            .env("AWS_REGION", $region)
            .status()
            .context(error::TuftoolSpawn)?;

        ensure!(status.success(), error::TuftoolResult {
            command: arg_str,
            code: status.code().map(|i| i.to_string()).unwrap_or_else(|| "<unknown>".to_string())
        });
    }
}

pub fn check_root(root_role_path: &Path) -> Result<()> {
    ensure!(!root_role_path.is_file(), {
        warn!("Cowardly refusing to overwrite the existing root.json at {}. Please manually delete it and run again.", root_role_path.display());
        error::FileExists {
            path: root_role_path,
        }
    });
    Ok(())
}

/// Creates the directory where root.json will live and creates root.json itself according to details specified in root-role-path
pub fn create_root(root_role_path: &Path) -> Result<()> {
    // Make /roles and /keys directories, if they don't exist, so we can write generated files.
    let role_dir = root_role_path.parent().context(error::InvalidPath {
        path: root_role_path,
        thing: "root role",
    })?;
    fs::create_dir_all(role_dir).context(error::Mkdir { path: role_dir })?;
    // Initialize root
    tuftool!(
        Region::default().name(),
        "root init '{}'",
        root_role_path.display()
    );
    tuftool!(
        Region::default().name(),
        // TODO: expose expiration date as a configurable parameter
        "root expire '{}' 'in 52 weeks'",
        root_role_path.display()
    );
    Ok(())
}

/// Adds keys to root.json according to key type  
pub fn add_keys(
    signing_key_config: &mut SigningKeyConfig,
    role: &KeyRole,
    threshold: &NonZeroUsize,
    filepath: &str,
) -> Result<()> {
    match signing_key_config {
        SigningKeyConfig::file { .. } => (),
        SigningKeyConfig::kms { key_id, config, .. } => add_keys_kms(
            &config
                .as_ref()
                .context(error::MissingConfig {
                    missing: "config field for a kms key",
                })?
                .available_keys,
            role,
            threshold,
            filepath,
            key_id,
        )?,
        SigningKeyConfig::ssm { .. } => (),
    }
    Ok(())
}

/// Adds KMSKeys to root.json given root or publication type
/// Input: available-keys (keys to sign with), role (root or publication), threshold for role, filepath for root.JSON,
/// mutable key_id
/// Output: in-place edit of root.json and key_id with a valid publication key
/// (If key-id is populated, it will not change. Otherwise, it will be populated with a key-id of an available key)
fn add_keys_kms(
    available_keys: &HashMap<String, String>,
    role: &KeyRole,
    threshold: &NonZeroUsize,
    filepath: &str,
    key_id: &mut Option<String>,
) -> Result<()> {
    ensure!(
        (*available_keys).len() >= (*threshold).get(),
        error::InvalidThreshold {
            threshold: threshold.to_string(),
            num_keys: (*available_keys).len(),
        }
    );

    match role {
        KeyRole::Root => {
            tuftool!(
                Region::default().name(),
                "root set-threshold '{}' root '{}' ",
                filepath,
                threshold.to_string()
            );
            for (keyid, region) in available_keys.iter() {
                tuftool!(
                    region,
                    "root add-key '{}' aws-kms:///'{}' --role root",
                    filepath,
                    keyid
                );
            }
        }
        KeyRole::Publication => {
            tuftool!(
                Region::default().name(),
                "root set-threshold '{}' snapshot '{}' ",
                filepath,
                threshold.to_string()
            );
            tuftool!(
                Region::default().name(),
                "root set-threshold '{}' targets '{}' ",
                filepath,
                threshold.to_string()
            );
            tuftool!(
                Region::default().name(),
                "root set-threshold '{}' timestamp '{}' ",
                filepath,
                threshold.to_string()
            );
            for (keyid, region) in available_keys.iter() {
                tuftool!(
                region,
                "root add-key '{}' aws-kms:///'{}' --role snapshot --role targets --role timestamp",
                filepath,
                keyid
                );
            }

            // Set key_id using a publication key (if one is not already provided)
            if key_id.is_none() {
                *key_id = Some(
                    available_keys
                        .iter()
                        .next()
                        .context(error::KeyCreation)?
                        .0
                        .to_string(),
                );
            }
        }
    }

    Ok(())
}

/// Signs root with available_keys under root_keys (will have a different tuftool command depending on key type)
pub fn sign_root(signing_key_config: &SigningKeyConfig, filepath: &str) -> Result<()> {
    match signing_key_config {
        SigningKeyConfig::file { .. } => (),
        SigningKeyConfig::kms { config, .. } => {
            for (keyid, region) in config
                .as_ref()
                .context(error::MissingConfig {
                    missing: "KMS key details",
                })?
                .available_keys
                .iter()
            {
                tuftool!(region, "root sign '{}' -k aws-kms:///'{}'", filepath, keyid);
            }
        }
        SigningKeyConfig::ssm { .. } => (),
    }
    Ok(())
}
