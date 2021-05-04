use rusoto_core::region::ParseRegionError;
use rusoto_core::{Region, RusotoError};
use rusoto_eks::{DescribeClusterError, Eks, EksClient};
use snafu::{OptionExt, ResultExt, Snafu};
use std::str::FromStr;

#[derive(Debug, Snafu)]
pub(super) enum Error {
    #[snafu(display("Error describing cluster: {}", source))]
    DescribeCluster {
        source: RusotoError<DescribeClusterError>,
    },

    #[snafu(display("Missing field '{}' EKS response", field))]
    Missing { field: &'static str },

    #[snafu(display("Unable to parse '{}' as a region: {}", region, source))]
    RegionParse {
        region: String,
        source: ParseRegionError,
    },
}

type Result<T> = std::result::Result<T, Error>;

/// Returns the cluster's [serviceIPv4CIDR] DNS IP by calling the EKS API.
/// (https://docs.aws.amazon.com/eks/latest/APIReference/API_KubernetesNetworkConfigRequest.html)
pub(super) async fn get_cluster_cidr(region: &str, cluster: &str) -> Result<String> {
    let parsed_region = Region::from_str(region).context(RegionParse { region })?;
    let client = EksClient::new(parsed_region);
    let describe_cluster = rusoto_eks::DescribeClusterRequest {
        name: cluster.to_owned(),
    };
    client
        .describe_cluster(describe_cluster)
        .await
        .context(DescribeCluster {})?
        .cluster
        .context(Missing { field: "cluster" })?
        .kubernetes_network_config
        .context(Missing {
            field: "kubernetes_network_config",
        })?
        .service_ipv_4_cidr
        .context(Missing {
            field: "service_ipv_4_cidr",
        })
}
