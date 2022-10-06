use async_trait::async_trait;
use aws_sdk_cloudformation::Client as CloudFormationClient;
use aws_types::region::Region;
use pubsys_config::{KMSKeyConfig, SigningKeyConfig};
use snafu::{OptionExt, ResultExt};
use std::fs;

use super::{error, shared, Result};

/// Creates keys using data stored in SigningKeyConfig enum
/// Output: Edits KMSConfig fields in place after creating new keys
pub async fn create_keys(signing_key_config: &mut SigningKeyConfig) -> Result<()> {
    // An extra check even through these parameters are checked earlier in main.rs
    check_signing_key_config(signing_key_config)?;
    match signing_key_config {
        SigningKeyConfig::file { .. } => (),
        SigningKeyConfig::kms { config, .. } => {
            config
                .as_mut()
                .context(error::MissingConfigSnafu {
                    missing: "config field for a kms key",
                })?
                .create_kms_keys()
                .await?;
        }
        SigningKeyConfig::ssm { .. } => (),
    }
    Ok(())
}

pub fn check_signing_key_config(signing_key_config: &SigningKeyConfig) -> Result<()> {
    match signing_key_config {
        SigningKeyConfig::file { .. } => (),
        SigningKeyConfig::kms { config, .. } => {
            let config = config.as_ref().context(error::MissingConfigSnafu {
                missing: "config field for kms keys",
            })?;

            match (
                config.available_keys.is_empty(),
                config.regions.is_empty(),
                config.key_alias.as_ref(),
            ) {
                // everything is unspecified (no way to allocate a key_id)
                (true, true, None) => error::KeyConfigSnafu {
                    missing: "an available_key or region/key_alias",
                }
                .fail()?,
                // regions is populated, but no key alias
                // (it doesn't matter if available keys are listed or not)
                (_, false, None) => error::KeyConfigSnafu {
                    missing: "key_alias",
                }
                .fail()?,
                // key alias is populated, but no key regions to create keys in
                // (it doesn't matter if available keys are listed or not)
                (_, true, Some(..)) => error::KeyConfigSnafu { missing: "region" }.fail()?,
                _ => (),
            };
        }
        SigningKeyConfig::ssm { .. } => (),
    }
    Ok(())
}

/// Must create a trait because can't directly implement a method for an struct in an
/// external crate like KMSKeyConfig (which lives in pubsys-config/lib.rs)
#[async_trait]
trait KMSKeyConfigExt {
    async fn create_kms_keys(&mut self) -> Result<()>;
}

/// Creates new KMS keys using cloudformation in regions specified
/// Input Conditions: Alias+Region or AvailableKeys must be specified
/// Output: Populates KMSKeyConfig with information about resources created
/// 'available-keys' starts as a map of pre-existing keyids:regions and will end as a
/// map of pre-existing and generated keyids:regions,
/// 'key-stack-arns' starts empty and will end as a
/// map of keyids:stackarn if new keys are created
#[async_trait]
impl KMSKeyConfigExt for KMSKeyConfig {
    async fn create_kms_keys(&mut self) -> Result<()> {
        // Generating new keys (if regions is non-empty)
        for region in self.regions.iter() {
            let stack_name = format!(
                "TUF-KMS-{}",
                self.key_alias.as_ref().context(error::KeyConfigSnafu {
                    missing: "key_alias",
                })?
            );

            let config = aws_config::from_env()
                .region(Region::new(region.to_owned()))
                .load()
                .await;
            let cfn_client = CloudFormationClient::new(&config);

            let cfn_filepath = format!(
                "{}/infrasys/cloudformation-templates/kms_key_setup.yml",
                shared::getenv("BUILDSYS_TOOLS_DIR")?
            );
            let cfn_template = fs::read_to_string(&cfn_filepath)
                .context(error::FileReadSnafu { path: cfn_filepath })?;

            let stack_result = cfn_client
                .create_stack()
                .parameters(shared::create_parameter(
                    "Alias".to_string(),
                    self.key_alias
                        .as_ref()
                        .context(error::KeyConfigSnafu {
                            missing: "key_alias",
                        })?
                        .to_string(),
                ))
                .stack_name(stack_name.clone())
                .template_body(cfn_template.clone())
                .send()
                .await
                .context(error::CreateStackSnafu {
                    stack_name: &stack_name,
                    region,
                })?;

            let stack_arn = stack_result
                .clone()
                .stack_id
                .context(error::ParseResponseSnafu {
                    what: "stack_id",
                    resource_name: &stack_name,
                })?;

            let output_array = shared::get_stack_outputs(&cfn_client, &stack_name, region).await?;
            let key_id =
                output_array[0]
                    .output_value
                    .as_ref()
                    .context(error::ParseResponseSnafu {
                        what: "outputs[0].output_value (key id)",
                        resource_name: stack_name,
                    })?;
            self.available_keys
                .insert(key_id.to_string(), region.to_string());
            self.key_stack_arns
                .insert(key_id.to_string(), stack_arn.to_string());
        }

        Ok(())
    }
}
