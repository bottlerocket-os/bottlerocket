use std::process::ExitCode;

use argh::FromArgs;
use imdsclient::ImdsClient;
use snafu::ResultExt;

use crate::error::{self, Result};

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "does-value-start-with")]
/// Compare IMDS variable with set of values
pub(crate) struct ValueStartsWith {
    #[argh(positional)]
    /// imds variable name (must be instance-type for now)
    variable_name: String,
    #[argh(option)]
    /// value to compare against that
    value: Vec<String>,
}

impl ValueStartsWith {
    pub(crate) async fn run(self) -> Result<ExitCode> {
        if self.variable_name != "instance-type" {
            log::info!(
                "Unknown variable name {}, returning false.",
                self.variable_name
            );
            return Ok(ExitCode::FAILURE);
        }
        if self.value.is_empty() {
            // Is it success or failure to match against the empty set?
            // Pretend this is the element-of operator. Nothing is an
            // element of the empty set.
            return Ok(ExitCode::FAILURE);
        }
        let mut client = ImdsClient::new();
        let instance_type = client
            .fetch_instance_type()
            .await
            .context(error::ImdsClientSnafu)?;
        match instance_type {
            // No instance type? No match.
            None => Ok(ExitCode::FAILURE),
            Some(instance_type) => {
                let does_match = self
                    .value
                    .iter()
                    .any(|value| instance_type.starts_with(value));
                if does_match {
                    log::info!("Instance type {} matched.", instance_type);
                    Ok(ExitCode::SUCCESS)
                } else {
                    log::info!("Instance type {} did not match.", instance_type);
                    Ok(ExitCode::FAILURE)
                }
            }
        }
    }
}
