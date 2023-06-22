use aws_sdk_cloudformation::model::{Output, Parameter};
use aws_sdk_cloudformation::Client as CloudFormationClient;
use clap::Parser;
use log::info;
use snafu::{ensure, OptionExt, ResultExt};
use std::{env, thread, time};

use super::{error, Result};

#[derive(Debug, Parser)]
pub enum KeyRole {
    Root,
    Publication,
}

/// Retrieve a BUILDSYS_* variable that we expect to be set in the environment
pub fn getenv(var: &str) -> Result<String> {
    env::var(var).context(error::EnvironmentSnafu { var })
}

/// Generates a parameter type object used to specify parameters in CloudFormation templates
pub fn create_parameter(key: String, val: String) -> Parameter {
    Parameter::builder()
        .parameter_key(key)
        .parameter_value(val)
        .build()
}

/// Polls cfn_client for stack_name in region until it's ready
/// Once stack is created, we can grab the outputs (before this point, outputs are empty)
pub async fn get_stack_outputs(
    cfn_client: &CloudFormationClient,
    stack_name: &str,
    region: &str,
) -> Result<Vec<Output>> {
    let mut stack_outputs = cfn_client
        .describe_stacks()
        .stack_name(stack_name)
        .send()
        .await
        .context(error::DescribeStackSnafu { stack_name, region })?
        .stacks
        .context(error::ParseResponseSnafu {
            what: "stacks",
            resource_name: stack_name,
        })?
        .first()
        .context(error::MissingStackSnafu { stack_name, region })?
        .clone();

    // Checking that keys have been created so we can return updated outputs
    let mut status = stack_outputs
        .stack_status()
        .context(error::ParseStatusSnafu)?
        .as_str();
    // Max wait is 30 mins (90 attempts * 20s = 1800s = 30mins)
    let mut max_attempts: u32 = 90;
    while status != "CREATE_COMPLETE" {
        ensure!(
            max_attempts > 0,
            error::CreateStackTimeoutSnafu { stack_name, region }
        );
        ensure!(
            status != "CREATE_FAILED",
            error::CreateStackFailureSnafu { stack_name, region }
        );
        info!(
            "Waiting for stack resources to be ready, current status is '{}'...",
            status
        );
        thread::sleep(time::Duration::from_secs(20));
        stack_outputs = cfn_client
            .describe_stacks()
            .stack_name(stack_name)
            .send()
            .await
            .context(error::DescribeStackSnafu { stack_name, region })?
            .stacks
            .context(error::ParseResponseSnafu {
                what: "stacks",
                resource_name: stack_name,
            })?
            .first()
            .context(error::MissingStackSnafu { stack_name, region })?
            .clone();
        status = stack_outputs
            .stack_status()
            .context(error::ParseStatusSnafu)?
            .as_str();
        max_attempts -= 1;
    }

    let output_array = stack_outputs.outputs.context(error::ParseResponseSnafu {
        what: "outputs",
        resource_name: stack_name,
    })?;

    Ok(output_array)
}
