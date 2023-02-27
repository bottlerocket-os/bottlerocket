#![deny(unused_imports)]

/*!
## Introduction

Cfsignal is similar to [cfn-signal](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/cfn-signal.html).

When creating an Auto Scaling Group, CloudFormation can be configured to wait for the expected number of signals from instances before considering the ASG successfully created. See [CreationPolicy](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-attribute-creationpolicy.html) and [UpdatePolicy](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-attribute-updatepolicy.html) for more details.

Cfsignal uses `systemctl is-system-running` to determine whether the boot has succeeded or failed, and sends the corresponding signal to the CloudFormation stack.

## Configuration

Configuration is read from a TOML file, which is generated from Bottlerocket settings:
* `should_signal`: Whether to check system status and send signal.
* `stack_name`: Name of the CFN stack to signal.
* `logical_resource_id`: The logical ID of the AutoScalingGroup resource that you want to signal.
*/

mod cloudformation;
mod config;
mod error;
mod system_check;

use crate::config::Config;
use crate::error::Result;
use crate::system_check::SystemCheck;
use cloudformation::signal_resource;
use log::LevelFilter;
use log::{error, info, warn};
use simplelog::{Config as LogConfig, SimpleLogger};
use std::fs;
use std::process;

// We only want to run cfsignal once, at first boot.  Our systemd unit file has a
// ConditionPathExists that will prevent it from running again if this file exists.
// We create it after running successfully.
const MARKER_FILE: &str = "/var/lib/bottlerocket/cfsignal.ran";
const CFSIGNAL_TOML: &str = "/etc/cfsignal.toml";

/// pub(crate) for testing.
async fn run() -> Result<()> {
    SimpleLogger::init(LevelFilter::Info, LogConfig::default())
        .expect("unable to configure logger");

    // load the cfsignal config file
    let config = Config::from_file(CFSIGNAL_TOML)?;

    let mut signal_status = "FAILURE";
    let system_check = Box::new(SystemCheck {});
    let system_status = system_check.system_running()?;
    info!(
        "System status is: {} [{}]",
        system_status.status, system_status.exit_code
    );

    // run only if the opt-in flag is set
    if config.should_signal {
        if system_status.is_healthy {
            signal_status = "SUCCESS";
        }

        if let Err(err) = signal_resource(
            config.stack_name,
            config.logical_resource_id,
            signal_status.to_owned(),
        )
        .await
        {
            error!("Error while sending signal: {}", err);
        }
    }

    fs::write(MARKER_FILE, "").unwrap_or_else(|e| {
        warn!(
            "Failed to create marker file '{}', may unexpectedly run again: '{}'",
            MARKER_FILE, e
        )
    });

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{}", e);
        process::exit(1);
    }
}
