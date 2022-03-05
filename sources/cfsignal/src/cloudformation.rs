use crate::error::{self, Result};
use imdsclient::ImdsClient;
use log::info;
use rusoto_cloudformation::{CloudFormation, CloudFormationClient, SignalResourceInput};
use rusoto_core::Region;
use snafu::{OptionExt, ResultExt};
use std::str::FromStr;

/// Signals Cloudformation stack resource
pub async fn signal_resource(
    stack_name: String,
    logical_resource_id: String,
    status: String,
) -> Result<()> {
    info!("Connecting to IMDS");
    let mut client = ImdsClient::new();
    let instance_id = get_instance_id(&mut client).await?;
    let region = get_region(&mut client).await?;

    info!(
        "Region: {:?} - InstanceID: {:?} - Signal: {:?}",
        region, instance_id, status
    );

    let client = CloudFormationClient::new(
        Region::from_str(&region).context(error::RegionParseSnafu { region })?,
    );
    let signal_resource_input = SignalResourceInput {
        stack_name,
        logical_resource_id,
        status,
        unique_id: instance_id,
    };

    client
        .signal_resource(signal_resource_input)
        .await
        .context(error::SignalResourceSnafu)?;

    Ok(())
}

/// Returns the instanceId
async fn get_instance_id(client: &mut ImdsClient) -> Result<String> {
    client
        .fetch_instance_id()
        .await
        .context(error::ImdsRequestSnafu)?
        .context(error::ImdsNoneSnafu {
            what: "instance-id",
        })
}

/// Returns the region
async fn get_region(client: &mut ImdsClient) -> Result<String> {
    client
        .fetch_region()
        .await
        .context(error::ImdsRequestSnafu)?
        .context(error::ImdsNoneSnafu { what: "region" })
}
