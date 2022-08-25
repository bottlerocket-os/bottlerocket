use aws_sdk_cloudformation::{Client as CloudFormationClient, Region};
use aws_sdk_s3::Client as S3Client;
use snafu::{OptionExt, ResultExt};
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use super::{error, shared, Result};

pub fn format_prefix(prefix: &str) -> String {
    if prefix.is_empty() {
        return prefix.to_string();
    }
    let formatted = {
        if prefix.starts_with('/') {
            prefix.to_string()
        } else {
            format!("/{}", prefix)
        }
    };
    if formatted.ends_with('/') {
        formatted[..formatted.len() - 1].to_string()
    } else if formatted.ends_with("/*") {
        formatted[..formatted.len() - 2].to_string()
    } else {
        formatted
    }
}

/// Creates a *private* S3 Bucket using a CloudFormation template
/// Input: The region in which the bucket will be created and the name of the bucket
/// Output: The stack_arn of the stack w/ the S3 bucket, the CFN allocated bucket name,
/// and the bucket url (for the url fields in Infra.lock)
pub async fn create_s3_bucket(
    region: &Region,
    stack_name: &str,
) -> Result<(String, String, String)> {
    // TODO: Add support for accommodating pre-existing buckets (skip this creation process)
    let config = aws_config::from_env()
        .region(region.to_owned())
        .load()
        .await;
    let cfn_client = CloudFormationClient::new(&config);

    let cfn_filepath: PathBuf = format!(
        "{}/infrasys/cloudformation-templates/s3_setup.yml",
        shared::getenv("BUILDSYS_TOOLS_DIR")?
    )
    .into();
    let cfn_template =
        fs::read_to_string(&cfn_filepath).context(error::FileReadSnafu { path: cfn_filepath })?;

    let stack_result = cfn_client
        .create_stack()
        .stack_name(stack_name.to_string())
        .template_body(cfn_template.clone())
        .send()
        .await
        .context(error::CreateStackSnafu {
            stack_name,
            region: region.as_ref(),
        })?;
    // We don't have to wait for successful stack creation to grab the stack ARN
    let stack_arn = stack_result
        .clone()
        .stack_id
        .context(error::ParseResponseSnafu {
            what: "stack_id",
            resource_name: stack_name,
        })?;

    // Grab the StackOutputs to get the Bucketname and BucketURL
    let output_array = shared::get_stack_outputs(&cfn_client, stack_name, region.as_ref()).await?;
    let bucket_name = output_array[0]
        .output_value
        .as_ref()
        .context(error::ParseResponseSnafu {
            what: "outputs[0].output_value (bucket name)",
            resource_name: stack_name,
        })?
        .to_string();
    let bucket_rdn = output_array[1]
        .output_value
        .as_ref()
        .context(error::ParseResponseSnafu {
            what: "outputs[1].output_value (bucket url)",
            resource_name: stack_name,
        })?
        .to_string();

    Ok((stack_arn, bucket_name, bucket_rdn))
}

/// Adds a BucketPolicy allowing GetObject access to a specified VPC
/// Input: Region, Name of bucket, which prefix root.json should be put under, and vpcid
/// Note that the prefix parameter must have the format "/<folder>/*" and the bucket name "<name>"
/// Output: Doesn't need to save any metadata from this action
pub async fn add_bucket_policy(
    region: &Region,
    bucket_name: &str,
    prefix: &str,
    vpcid: &str,
) -> Result<()> {
    // Get old policy
    let config = aws_config::from_env()
        .region(region.to_owned())
        .load()
        .await;
    let s3_client = S3Client::new(&config);
    let mut policy: serde_json::Value = match s3_client
        .get_bucket_policy()
        .bucket(bucket_name.to_string())
        .send()
        .await
    {
        Ok(output) => serde_json::from_str(&output.policy.context(error::ParseResponseSnafu {
            what: "policy",
            resource_name: bucket_name,
        })?)
        .context(error::InvalidJsonSnafu {
            what: format!("retrieved bucket policy for {}", &bucket_name),
        })?,

        Err(..) => serde_json::from_str(
            r#"{"Version": "2008-10-17",
                     "Statement": []}"#,
        )
        .context(error::InvalidJsonSnafu {
            what: format!("new bucket policy for {}", &bucket_name),
        })?,
    };

    // Create a new policy
    let new_bucket_policy = serde_json::from_str(&format!(
        r#"{{
                       "Effect": "Allow",
                        "Principal": "*",
                        "Action": "s3:GetObject",
                        "Resource": "arn:aws:s3:::{}{}/*",
                        "Condition": {{
                            "StringEquals": {{
                                "aws:sourceVpce": "{}"
                            }}
                        }}
                    }}"#,
        bucket_name, prefix, vpcid
    ))
    .context(error::InvalidJsonSnafu {
        what: format!("new bucket policy for {}", &bucket_name),
    })?;

    // Append new policy onto old one
    policy
        .get_mut("Statement")
        .context(error::GetPolicyStatementSnafu { bucket_name })?
        .as_array_mut()
        .context(error::GetPolicyStatementSnafu { bucket_name })?
        .push(new_bucket_policy);

    // Push the new policy as a string
    s3_client
        .put_bucket_policy()
        .bucket(bucket_name.to_string())
        .policy(
            serde_json::to_string(&policy).context(error::InvalidJsonSnafu {
                what: format!("new bucket policy for {}", &bucket_name),
            })?,
        )
        .send()
        .await
        .context(error::PutPolicySnafu { bucket_name })?;

    Ok(())
}

/// Uploads root.json to S3 Bucket (automatically creates the folder that the bucket policy was scoped to or will simply add to it)
/// Input: Region, Name of bucket, which prefix root.json should be put under, and path to the S3 bucket CFN template
/// Note that the prefix parameter must have the format "/<folder>" and the bucket name "<name>"
/// Output: Doesn't need to save any metadata from this action
pub async fn upload_file(
    region: &Region,
    bucket_name: &str,
    prefix: &str,
    file_path: &Path,
) -> Result<()> {
    let config = aws_config::from_env()
        .region(region.to_owned())
        .load()
        .await;
    let s3_client = S3Client::new(&config);

    // File --> Bytes
    let mut file = File::open(file_path).context(error::FileOpenSnafu { path: file_path })?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .context(error::FileReadSnafu { path: file_path })?;

    s3_client
        .put_object()
        .bucket(format!("{}{}", bucket_name, prefix))
        .key("root.json".to_string())
        .body(aws_sdk_s3::types::ByteStream::from(buffer))
        .send()
        .await
        .context(error::PutObjectSnafu { bucket_name })?;

    Ok(())
}

//  =^..^=   =^..^=   =^..^=  TESTS  =^..^=   =^..^=   =^..^=

#[cfg(test)]
mod tests {
    use super::format_prefix;
    use assert_json_diff::assert_json_include;

    #[test]
    fn format_prefix_test() {
        let valid = "/prefix";
        let missing_slash = "prefix";
        let excess_ending_1 = "/prefix/";
        let excess_ending_2 = "/prefix/*";
        let slash_and_excess_ending = "prefix/*";
        let empty = "";
        let single_slash = "/";

        assert_eq!("/prefix", format_prefix(valid));
        assert_eq!("/prefix", format_prefix(missing_slash));
        assert_eq!("/prefix", format_prefix(excess_ending_1));
        assert_eq!("/prefix", format_prefix(excess_ending_2));
        assert_eq!("/prefix", format_prefix(slash_and_excess_ending));
        assert_eq!("", format_prefix(empty));
        assert_eq!("", format_prefix(single_slash));
    }

    #[test]
    fn empty_bucket_policy() {
        let mut policy: serde_json::Value = serde_json::from_str(
            r#"{"Version": "2008-10-17",
                     "Statement": []}"#,
        )
        .unwrap();

        let new_bucket_policy = serde_json::from_str(&format!(
            r#"{{
                "Effect": "Allow",
                 "Principal": "*",
                 "Action": "s3:GetObject",
                 "Resource": "arn:aws:s3:::{}{}/*",
                 "Condition": {{
                     "StringEquals": {{
                         "aws:sourceVpce": "{}"
                     }}
                 }}
             }}"#,
            "test-bucket-name", "/test-prefix", "testvpc123"
        ))
        .unwrap();

        policy
            .get_mut("Statement")
            .unwrap()
            .as_array_mut()
            .unwrap()
            .push(new_bucket_policy);

        let expected_policy: serde_json::Value = serde_json::from_str(
            r#"{
            "Version": "2008-10-17",
            "Statement": [
                {
                    "Effect": "Allow",
                    "Principal": "*",
                    "Action": "s3:GetObject",
                    "Resource": "arn:aws:s3:::test-bucket-name/test-prefix/*",
                    "Condition": {
                        "StringEquals": {
                            "aws:sourceVpce": "testvpc123"
                        }
                    }
                }
            ]
        }"#,
        )
        .unwrap();

        assert_json_include!(expected: expected_policy, actual: &policy);
    }

    #[test]
    fn populated_bucket_policy() {
        let mut policy: serde_json::Value = serde_json::from_str(
            r#"{
                "Version": "2008-10-17",
                "Statement": [
                    {
                        "Effect": "Allow",
                        "Principal": "*",
                        "Action": "s3:GetObject",
                        "Resource": "arn:aws:s3:::test-bucket-name/test-prefix/*",
                        "Condition": {
                            "StringEquals": {
                                "aws:sourceVpce": "testvpc123"
                            }
                        }
                    }
                ]
            }"#,
        )
        .unwrap();

        let new_bucket_policy = serde_json::from_str(&format!(
            r#"{{
                "Effect": "Deny",
                 "Principal": "*",
                 "Action": "s3:GetObject",
                 "Resource": "arn:aws:s3:::{}{}/*",
                 "Condition": {{
                     "StringEquals": {{
                         "aws:sourceVpce": "{}"
                     }}
                 }}
             }}"#,
            "test-bucket-name", "/test-prefix", "testvpc123"
        ))
        .unwrap();

        policy
            .get_mut("Statement")
            .unwrap()
            .as_array_mut()
            .unwrap()
            .push(new_bucket_policy);

        let expected_policy: serde_json::Value = serde_json::from_str(
            r#"{
            "Version": "2008-10-17",
            "Statement": [
                {
                    "Effect": "Allow",
                    "Principal": "*",
                    "Action": "s3:GetObject",
                    "Resource": "arn:aws:s3:::test-bucket-name/test-prefix/*",
                    "Condition": {
                        "StringEquals": {
                            "aws:sourceVpce": "testvpc123"
                        }
                    }
                },
                {
                    "Effect": "Deny",
                    "Principal": "*",
                    "Action": "s3:GetObject",
                    "Resource": "arn:aws:s3:::test-bucket-name/test-prefix/*",
                    "Condition": {
                        "StringEquals": {
                            "aws:sourceVpce": "testvpc123"
                        }
                    }
                }
            ]
        }"#,
        )
        .unwrap();

        assert_json_include!(expected: expected_policy, actual: &policy);
    }
}
