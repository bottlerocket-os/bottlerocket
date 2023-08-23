use std::str::FromStr;

use crate::error::{self, Result};
use aws_types::region::Region;
use imdsclient::ImdsClient;
use log::info;
use snafu::{OptionExt, ResultExt};

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
    let config = aws_config::from_env()
        .region(Region::new(region.to_owned()))
        .load()
        .await;
    let client = aws_sdk_cloudformation::Client::new(&config);

    client
        .signal_resource()
        .stack_name(stack_name)
        .logical_resource_id(logical_resource_id)
        .status(
            aws_sdk_cloudformation::types::ResourceSignalStatus::from_str(&status)
                .expect("infallible"),
        )
        .unique_id(instance_id)
        .send()
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
