use crate::proxy;
use aws_smithy_types::error::display::DisplayErrorContext;
use aws_types::region::Region;
use snafu::{OptionExt, ResultExt, Snafu};
use std::time::Duration;

// Limit the timeout for the EC2 describe-instances API call to 5 minutes
const EC2_DESCRIBE_INSTANCES_TIMEOUT: Duration = Duration::from_secs(300);

#[derive(Debug, Snafu)]
pub(super) enum Error {
    #[snafu(display(
        "Error describing instance '{}': {}",
        instance_id,
        DisplayErrorContext(source)
    ))]
    DescribeInstances {
        instance_id: String,
        source: aws_sdk_eks::types::SdkError<aws_sdk_ec2::error::DescribeInstancesError>,
    },

    #[snafu(display("Timed-out waiting for EC2 DescribeInstances API response: {}", source))]
    DescribeInstancesTimeout { source: tokio::time::error::Elapsed },

    #[snafu(display("Missing field '{}' in EC2 response", field))]
    Missing { field: &'static str },

    #[snafu(context(false), display("{}", source))]
    Proxy { source: proxy::Error },
}

type Result<T> = std::result::Result<T, Error>;

pub(super) async fn get_private_dns_name(region: &str, instance_id: &str) -> Result<String> {
    // Respect proxy environment variables when making AWS EC2 API requests
    let (https_proxy, no_proxy) = proxy::fetch_proxy_env();

    let config = aws_config::from_env()
        .region(Region::new(region.to_owned()))
        .load()
        .await;

    let client = if let Some(https_proxy) = https_proxy {
        let http_client = proxy::setup_http_client(https_proxy, no_proxy)?;
        let ec2_config = aws_sdk_ec2::config::Builder::from(&config)
            .http_connector(http_client)
            .build();
        aws_sdk_ec2::Client::from_conf(ec2_config)
    } else {
        aws_sdk_ec2::Client::new(&config)
    };

    tokio::time::timeout(
        EC2_DESCRIBE_INSTANCES_TIMEOUT,
        client
            .describe_instances()
            .instance_ids(instance_id.to_owned())
            .send(),
    )
    .await
    .context(DescribeInstancesTimeoutSnafu)?
    .context(DescribeInstancesSnafu { instance_id })?
    .reservations
    .and_then(|reservations| {
        reservations.first().and_then(|r| {
            r.instances.clone().and_then(|instances| {
                instances
                    .first()
                    .and_then(|i| i.private_dns_name().map(|s| s.to_string()))
            })
        })
    })
    .context(MissingSnafu {
        field: "Reservation.Instance.PrivateDNSName",
    })
}
