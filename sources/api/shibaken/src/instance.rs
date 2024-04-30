use std::process::ExitCode;

use argh::FromArgs;
use imdsclient::ImdsClient;
use snafu::ResultExt;

use crate::error::{self, Result};

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "instance-type")]
/// Fetch instance type from IMDS
pub(crate) struct InstanceType {
    #[argh(option)]
    /// prefix string(s) to match against instance type
    prefix: Vec<String>,
}

impl InstanceType {
    pub(crate) async fn run(self) -> Result<ExitCode> {
        if self.prefix.is_empty() {
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
                    .prefix
                    .iter()
                    .any(|prefix| instance_type.starts_with(prefix));
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
