use crate::aws::sdk_config;
use crate::{aws, proxy};
use aws_sdk_eks::types::KubernetesNetworkConfigResponse;
use snafu::{OptionExt, ResultExt, Snafu};
use std::time::Duration;

// Limit the timeout for the EKS describe cluster API call to 5 minutes
const EKS_DESCRIBE_CLUSTER_TIMEOUT: Duration = Duration::from_secs(300);

pub(crate) type ClusterNetworkConfig = KubernetesNetworkConfigResponse;

#[derive(Debug, Snafu)]
pub(super) enum Error {
    #[snafu(display("Error describing cluster: {}", source))]
    DescribeCluster {
        source: aws_sdk_eks::error::SdkError<
            aws_sdk_eks::operation::describe_cluster::DescribeClusterError,
        >,
    },

    #[snafu(display("Timed-out waiting for EKS Describe Cluster API response: {}", source))]
    DescribeClusterTimeout { source: tokio::time::error::Elapsed },

    #[snafu(display("Missing field '{}' in EKS response", field))]
    Missing { field: &'static str },

    #[snafu(context(false), display("{}", source))]
    Proxy { source: proxy::Error },

    #[snafu(context(false), display("{}", source))]
    SdkConfig { source: aws::Error },
}

type Result<T> = std::result::Result<T, Error>;

/// Returns the cluster's [kubernetesNetworkConfig] by calling the EKS API.
/// (https://docs.aws.amazon.com/eks/latest/APIReference/API_KubernetesNetworkConfigResponse.html)
pub(super) async fn get_cluster_network_config(
    region: &str,
    cluster: &str,
) -> Result<ClusterNetworkConfig> {
    // Respect proxy environment variables when making AWS EKS API requests
    let (https_proxy, no_proxy) = proxy::fetch_proxy_env();

    let config = sdk_config(region).await?;

    let client = if let Some(https_proxy) = https_proxy {
        let http_client = proxy::setup_http_client(https_proxy, no_proxy)?;
        let eks_config = aws_sdk_eks::config::Builder::from(&config)
            .http_connector(http_client)
            .build();
        aws_sdk_eks::Client::from_conf(eks_config)
    } else {
        aws_sdk_eks::Client::new(&config)
    };

    tokio::time::timeout(
        EKS_DESCRIBE_CLUSTER_TIMEOUT,
        client.describe_cluster().name(cluster.to_owned()).send(),
    )
    .await
    .context(DescribeClusterTimeoutSnafu)?
    .context(DescribeClusterSnafu)?
    .cluster
    .context(MissingSnafu { field: "cluster" })?
    .kubernetes_network_config
    .context(MissingSnafu {
        field: "kubernetes_network_config",
    })
}
