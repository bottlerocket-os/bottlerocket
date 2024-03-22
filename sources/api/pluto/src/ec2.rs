use crate::aws::sdk_config;
use crate::{aws, proxy};
use aws_smithy_types::error::display::DisplayErrorContext;
use snafu::{OptionExt, ResultExt, Snafu};
use std::time::Duration;
use tokio_retry::{
    strategy::{jitter, FibonacciBackoff},
    Retry,
};

// Limit the timeout for fetching the private DNS name of the EC2 instance to 5 minutes.
const FETCH_PRIVATE_DNS_NAME_TIMEOUT: Duration = Duration::from_secs(300);
// Fibonacci backoff base duration when retrying requests
const FIBONACCI_BACKOFF_BASE_DURATION_MILLIS: u64 = 200;

#[derive(Debug, Snafu)]
pub(super) enum Error {
    #[snafu(display(
        "Error describing instance '{}': {}",
        instance_id,
        DisplayErrorContext(source)
    ))]
    DescribeInstances {
        instance_id: String,
        source: aws_sdk_eks::error::SdkError<
            aws_sdk_ec2::operation::describe_instances::DescribeInstancesError,
        >,
    },

    #[snafu(display("Timed out retrieving private DNS name from EC2: {}", source))]
    FetchPrivateDnsNameTimeout { source: tokio::time::error::Elapsed },

    #[snafu(display("Missing field '{}' in EC2 response", field))]
    Missing { field: &'static str },

    #[snafu(context(false), display("{}", source))]
    Proxy { source: proxy::Error },

    #[snafu(context(false), display("{}", source))]
    SdkConfig { source: aws::Error },
}

type Result<T> = std::result::Result<T, Error>;

pub(super) async fn get_private_dns_name(region: &str, instance_id: &str) -> Result<String> {
    // Respect proxy environment variables when making AWS EC2 API requests
    let (https_proxy, no_proxy) = proxy::fetch_proxy_env();

    let config = sdk_config(region).await?;

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
        FETCH_PRIVATE_DNS_NAME_TIMEOUT,
        Retry::spawn(
            FibonacciBackoff::from_millis(FIBONACCI_BACKOFF_BASE_DURATION_MILLIS).map(jitter),
            || async {
                client
                    .describe_instances()
                    .instance_ids(instance_id.to_owned())
                    .send()
                    .await
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
                    .filter(|private_dns_name| !private_dns_name.is_empty())
                    .context(MissingSnafu {
                        field: "Reservation.Instance.PrivateDNSName",
                    })
            },
        ),
    )
    .await
    .context(FetchPrivateDnsNameTimeoutSnafu)?
}
