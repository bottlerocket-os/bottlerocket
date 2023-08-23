mod error;
mod keys;
mod root;
mod s3;
mod shared;

use aws_sdk_cloudformation::config::Region;
use clap::Parser;
use error::Result;
use log::{error, info};
use pubsys_config::{InfraConfig, RepoConfig, S3Config, SigningKeyConfig};
use sha2::{Digest, Sha512};
use shared::KeyRole;
use simplelog::{CombinedLogger, Config as LogConfig, ConfigBuilder, LevelFilter, SimpleLogger};
use snafu::{ensure, OptionExt, ResultExt};
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::{fs, process};
use tokio::runtime::Runtime;
use url::Url;

//   =^..^=   =^..^=   =^..^=  SUB-COMMAND STRUCTS  =^..^=   =^..^=   =^..^=

#[derive(Debug, Parser)]
struct Args {
    #[arg(global = true, long, default_value = "INFO")]
    log_level: LevelFilter,

    // Path to Infra.toml (NOTE: must be specified before subcommand)
    #[arg(long)]
    infra_config_path: PathBuf,

    #[command(subcommand)]
    subcommand: SubCommand,
}

#[derive(Debug, Parser)]
struct CreateInfraArgs {
    /// Path to the root.json file.
    #[arg(long)]
    root_role_path: PathBuf,
}

#[derive(Debug, Parser)]
enum SubCommand {
    /// Creates infrastructure specified in the Infra.toml file.
    CreateInfra(CreateInfraArgs),
}

//  =^..^=   =^..^=   =^..^=  MAIN METHODS  =^..^=   =^..^=   =^..^=

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    // Parse and store the args passed to the program
    let args = Args::parse();

    match args.log_level {
        // Set log level for AWS SDK to error to reduce verbosity.
        LevelFilter::Info => {
            CombinedLogger::init(vec![
                SimpleLogger::new(
                    LevelFilter::Info,
                    ConfigBuilder::new()
                        .add_filter_ignore_str("aws_config")
                        .add_filter_ignore_str("aws_smithy")
                        .add_filter_ignore_str("tracing::span")
                        .build(),
                ),
                SimpleLogger::new(
                    LevelFilter::Warn,
                    ConfigBuilder::new()
                        .add_filter_allow_str("aws_config")
                        .add_filter_allow_str("aws_smithy")
                        .add_filter_allow_str("tracing::span")
                        .build(),
                ),
            ])
            .context(error::LoggerSnafu)?;
        }

        // Set the supplied log level across the whole crate.
        _ => {
            SimpleLogger::init(args.log_level, LogConfig::default()).context(error::LoggerSnafu)?
        }
    }

    match args.subcommand {
        SubCommand::CreateInfra(ref run_task_args) => {
            let rt = Runtime::new().context(error::RuntimeSnafu)?;
            rt.block_on(async {
                create_infra(&args.infra_config_path, &run_task_args.root_role_path).await
            })
        }
    }
}

fn check_infra_lock(toml_path: &Path) -> Result<()> {
    let lock_path = InfraConfig::compute_lock_path(toml_path).context(error::ConfigSnafu)?;

    ensure!(!lock_path.is_file(), {
        error!(
            "It looks like you've already created some resources for your custom TUF repository because a lock file exists at '{}'.
            \nPlease clean up your TUF resources in AWS, delete Infra.lock, and run again.",
            lock_path.display()
        );
        error::FileExistsSnafu { path: lock_path }
    });
    Ok(())
}

/// Automates setting up infrastructure for a custom TUF repo
async fn create_infra(toml_path: &Path, root_role_path: &Path) -> Result<()> {
    check_infra_lock(toml_path)?;
    info!("Parsing Infra.toml...");
    let mut infra_config = InfraConfig::from_path(toml_path).context(error::ConfigSnafu)?;
    let repos = infra_config
        .repo
        .as_mut()
        .context(error::MissingConfigSnafu { missing: "repo" })?;
    let s3_info_map = infra_config
        .aws
        .as_mut()
        .context(error::MissingConfigSnafu { missing: "aws" })?
        .s3
        .as_mut()
        .context(error::MissingConfigSnafu { missing: "aws.s3" })?;

    for (repo_name, repo_config) in repos.iter_mut() {
        // Validate repo_config and unwrap required optional data
        let mut repo_info = ValidRepoInfo::new(repo_config, repo_name, s3_info_map)?;

        // Validate the key configurations and root file
        keys::check_signing_key_config(repo_info.signing_keys)?;
        keys::check_signing_key_config(repo_info.root_keys)?;
        root::check_root(root_role_path)?;

        // Create the repo
        let (s3_stack_arn, bucket_name, bucket_rdn) =
            create_repo_infrastructure(&mut repo_info).await?;
        *repo_info.stack_arn = Some(s3_stack_arn);
        *repo_info.bucket_name = Some(bucket_name.clone());
        update_root_and_sign_root(&mut repo_info, root_role_path).await?;

        // Upload root.json.
        info!("Uploading root.json to S3 bucket...");
        s3::upload_file(
            &repo_info.s3_region,
            &bucket_name,
            &repo_info.prefix,
            root_role_path,
        )
        .await?;

        // Update infra_config with output parameters if not already set
        if repo_info.metadata_base_url.is_none() {
            *repo_info.metadata_base_url = Some(
                Url::parse(format!("https://{}{}/", &bucket_rdn, &repo_info.prefix).as_str())
                    .context(error::ParseUrlSnafu { input: &bucket_rdn })?,
            );
        }
        if repo_info.targets_url.is_none() {
            *repo_info.targets_url = Some(
                Url::parse(
                    format!("https://{}{}/targets/", &bucket_rdn, &repo_info.prefix).as_str(),
                )
                .context(error::ParseUrlSnafu { input: &bucket_rdn })?,
            );
        }
        if repo_info.root_role_url.is_none() {
            *repo_info.root_role_url = Some(
                Url::parse(
                    format!("https://{}{}/root.json", &bucket_rdn, &repo_info.prefix).as_str(),
                )
                .context(error::ParseUrlSnafu { input: &bucket_rdn })?,
            );
        }
        let root_role_data = fs::read_to_string(root_role_path).context(error::FileReadSnafu {
            path: root_role_path,
        })?;
        let mut d = Sha512::new();
        d.update(&root_role_data);
        let digest = hex::encode(d.finalize());
        repo_config.root_role_sha512 = Some(digest);
    }

    // Generate Infra.lock
    info!("Writing Infra.lock...");
    let yaml_string = serde_yaml::to_string(&infra_config).context(error::InvalidYamlSnafu)?;
    fs::write(
        toml_path
            .parent()
            .context(error::ParentSnafu { path: toml_path })?
            .join("Infra.lock"),
        yaml_string,
    )
    .context(error::FileWriteSnafu { path: toml_path })?;

    info!("Complete!");
    Ok(())
}

struct ValidRepoInfo<'a> {
    bucket_name: &'a mut Option<String>,
    metadata_base_url: &'a mut Option<Url>,
    prefix: String,
    pub_key_threshold: &'a NonZeroUsize,
    root_key_threshold: &'a NonZeroUsize,
    root_keys: &'a mut SigningKeyConfig,
    root_role_url: &'a mut Option<Url>,
    s3_region: Region,
    s3_stack_name: String,
    signing_keys: &'a mut SigningKeyConfig,
    stack_arn: &'a mut Option<String>,
    targets_url: &'a mut Option<Url>,
    vpce_id: &'a String,
}

impl<'a> ValidRepoInfo<'a> {
    fn new(
        repo_config: &'a mut RepoConfig,
        repo_name: &str,
        s3_info_map: &'a mut HashMap<String, S3Config>,
    ) -> Result<Self> {
        let s3_stack_name =
            repo_config
                .file_hosting_config_name
                .to_owned()
                .context(error::MissingConfigSnafu {
                    missing: "file_hosting_config_name",
                })?;
        let s3_info = s3_info_map
            .get_mut(&s3_stack_name)
            .context(error::MissingConfigSnafu {
                missing: format!("aws.s3 config with name {}", s3_stack_name),
            })?;
        Ok(ValidRepoInfo {
            s3_stack_name: s3_stack_name.to_string(),
            s3_region: Region::new(s3_info.region.as_ref().cloned().context(
                error::MissingConfigSnafu {
                    missing: format!("region for '{}' s3 config", s3_stack_name),
                },
            )?),
            bucket_name: &mut s3_info.bucket_name,
            stack_arn: &mut s3_info.stack_arn,
            vpce_id: s3_info
                .vpc_endpoint_id
                .as_ref()
                .context(error::MissingConfigSnafu {
                    missing: format!("vpc_endpoint_id for '{}' s3 config", s3_stack_name),
                })?,
            prefix: s3::format_prefix(&s3_info.s3_prefix),
            signing_keys: repo_config
                .signing_keys
                .as_mut()
                .context(error::MissingConfigSnafu {
                    missing: format!("signing_keys for '{}' repo config", repo_name),
                })?,
            root_keys: repo_config
                .root_keys
                .as_mut()
                .context(error::MissingConfigSnafu {
                    missing: format!("root_keys for '{}' repo config", repo_name),
                })?,
            root_key_threshold: repo_config.root_key_threshold.as_mut().context(
                error::MissingConfigSnafu {
                    missing: format!("root_key_threshold for '{}' repo config", repo_name),
                },
            )?,
            pub_key_threshold: repo_config.pub_key_threshold.as_ref().context(
                error::MissingConfigSnafu {
                    missing: format!("pub_key_threshold for '{}' repo config", repo_name),
                },
            )?,
            root_role_url: &mut repo_config.root_role_url,
            targets_url: &mut repo_config.targets_url,
            metadata_base_url: &mut repo_config.metadata_base_url,
        })
    }
}

async fn create_repo_infrastructure(
    repo_info: &'_ mut ValidRepoInfo<'_>,
) -> Result<(String, String, String)> {
    // Create S3 bucket
    info!("Creating S3 bucket...");
    let (s3_stack_arn, bucket_name, bucket_rdn) =
        s3::create_s3_bucket(&repo_info.s3_region, &repo_info.s3_stack_name).await?;

    // Add Bucket Policy to newly created bucket
    s3::add_bucket_policy(
        &repo_info.s3_region,
        &bucket_name,
        &repo_info.prefix,
        repo_info.vpce_id,
    )
    .await?;

    // Create root + publication keys
    info!("Creating KMS Keys...");
    keys::create_keys(repo_info.signing_keys).await?;
    keys::create_keys(repo_info.root_keys).await?;
    Ok((s3_stack_arn, bucket_name, bucket_rdn))
}

async fn update_root_and_sign_root(
    repo_info: &'_ mut ValidRepoInfo<'_>,
    root_role_path: &Path,
) -> Result<()> {
    // Create and populate (add/sign) root.json
    info!("Creating and signing root.json...");
    root::create_root(root_role_path)?;
    // Add keys (for both roles)
    root::add_keys(
        repo_info.signing_keys,
        &KeyRole::Publication,
        repo_info.pub_key_threshold,
        &root_role_path.display().to_string(),
    )?;
    root::add_keys(
        repo_info.root_keys,
        &KeyRole::Root,
        repo_info.root_key_threshold,
        &root_role_path.display().to_string(),
    )?;
    // Sign root with all root keys
    root::sign_root(repo_info.root_keys, &root_role_path.display().to_string())?;
    Ok(())
}

//  =^..^=   =^..^=   =^..^=  TESTS  =^..^=   =^..^=   =^..^=

#[cfg(test)]
mod tests {
    use super::{fs, shared, InfraConfig};

    #[test]
    fn toml_yaml_conversion() {
        let test_toml_path = format!(
            "{}/test_tomls/toml_yaml_conversion.toml",
            shared::getenv("CARGO_MANIFEST_DIR").unwrap()
        );
        let toml_struct = InfraConfig::from_path(&test_toml_path).unwrap();
        let yaml_string = serde_yaml::to_string(&toml_struct).expect("Could not write to file!");

        let test_yaml_path = format!(
            "{}/test_tomls/toml_yaml_conversion.yml",
            shared::getenv("CARGO_MANIFEST_DIR").unwrap()
        );
        fs::write(&test_yaml_path, &yaml_string).expect("Could not write to file!");
        let decoded_yaml = InfraConfig::from_lock_path(&test_yaml_path).unwrap();

        assert_eq!(toml_struct, decoded_yaml);
    }
}
