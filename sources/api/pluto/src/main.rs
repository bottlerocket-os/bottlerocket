/*!
# Introduction

pluto is called by sundog to generate settings required by Kubernetes.
This is done dynamically because we require access to dynamic networking
and cluster setup information.

It uses IMDS to get information such as:

- Instance Type
- Node IP

It uses EKS to get information such as:

- Service IP CIDR

It uses the Bottlerocket API to get information such as:

- Kubernetes Cluster Name
- AWS Region

# Interface

Pluto takes the name of the setting that it is to generate as its first
argument.
It returns the generated setting to stdout as a JSON document.
Any other output is returned to stderr.

Pluto returns a special exit code of 2 to inform `sundog` that a setting should be skipped. For
example, if `max-pods` cannot be generated, we want `sundog` to skip it without failing since a
reasonable default is available.
*/

mod api;
mod aws;
mod ec2;
mod eks;
mod hyper_proxy;
mod proxy;

use api::AwsK8sInfo;
use imdsclient::ImdsClient;
use modeled_types::KubernetesClusterDnsIp;
use snafu::{ensure, OptionExt, ResultExt};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::IpAddr;
use std::process;
use std::str::FromStr;
use std::string::String;

// This is the default DNS unless our CIDR block begins with "10."
const DEFAULT_DNS_CLUSTER_IP: &str = "10.100.0.10";
// If our CIDR block begins with "10." this is our DNS.
const DEFAULT_10_RANGE_DNS_CLUSTER_IP: &str = "172.20.0.10";

const ENI_MAX_PODS_PATH: &str = "/usr/share/eks/eni-max-pods";

const NO_HOSTNAME_VARIANTS: &[&str] = &["aws-k8s-1.23", "aws-k8s-1.24", "aws-k8s-1.25"];

mod error {
    use crate::{api, ec2, eks};
    use snafu::Snafu;
    use std::net::AddrParseError;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(super) enum PlutoError {
        #[snafu(display(
            "Unable to retrieve cluster name and AWS region from Bottlerocket API: {}",
            source
        ))]
        AwsInfo { source: api::Error },

        #[snafu(display("Missing AWS region"))]
        AwsRegion,

        #[snafu(display("Failed to parse setting {} as u32: {}", setting, source))]
        ParseToU32 {
            setting: String,
            source: std::num::ParseIntError,
        },

        #[snafu(display("Unable to parse CIDR '{}': {}", cidr, reason))]
        CidrParse { cidr: String, reason: String },

        #[snafu(display("Unable to parse IP '{}': {}", ip, source))]
        BadIp { ip: String, source: AddrParseError },

        #[snafu(display("No IP address found for this host"))]
        NoIp,

        #[snafu(display("IMDS request failed: {}", source))]
        ImdsRequest { source: imdsclient::Error },

        #[snafu(display("IMDS request failed: No '{}' found", what))]
        ImdsNone { what: String },

        #[snafu(display("{}", source))]
        EksError { source: eks::Error },

        #[snafu(display("{}", source))]
        Ec2Error { source: ec2::Error },

        #[snafu(display("Failed to open eni-max-pods file at {}: {}", path, source))]
        EniMaxPodsFile {
            path: &'static str,
            source: std::io::Error,
        },

        #[snafu(display("Failed to read line: {}", source))]
        IoReadLine { source: std::io::Error },

        #[snafu(display("Failed to serialize generated settings: {}", source))]
        Serialize { source: serde_json::Error },

        #[snafu(display("Failed to set generated settings: {}", source))]
        SetFailure { source: api::Error },

        #[snafu(display(
            "Unable to find maximum number of pods supported for instance-type {}",
            instance_type
        ))]
        NoInstanceTypeMaxPods { instance_type: String },
    }
}

use error::PlutoError;

type Result<T> = std::result::Result<T, PlutoError>;

async fn generate_max_pods(client: &mut ImdsClient, aws_k8s_info: &mut AwsK8sInfo) -> Result<()> {
    if aws_k8s_info.max_pods.is_some() {
        return Ok(());
    }
    aws_k8s_info.max_pods = get_max_pods(client).await.ok();
    Ok(())
}

async fn get_max_pods(client: &mut ImdsClient) -> Result<u32> {
    let instance_type = client
        .fetch_instance_type()
        .await
        .context(error::ImdsRequestSnafu)?
        .context(error::ImdsNoneSnafu {
            what: "instance_type",
        })?;

    // Find the corresponding maximum number of pods supported by this instance type
    let file = BufReader::new(File::open(ENI_MAX_PODS_PATH).context(
        error::EniMaxPodsFileSnafu {
            path: ENI_MAX_PODS_PATH,
        },
    )?);
    for line in file.lines() {
        let line = line.context(error::IoReadLineSnafu)?;
        // Skip the comments in the file
        if line.trim_start().starts_with('#') {
            continue;
        }
        let tokens: Vec<_> = line.split_whitespace().collect();
        if tokens.len() == 2 && tokens[0] == instance_type {
            let setting = tokens[1];
            return setting.parse().context(error::ParseToU32Snafu { setting });
        }
    }
    error::NoInstanceTypeMaxPodsSnafu { instance_type }.fail()
}

/// Returns the cluster's DNS address.
///
/// For IPv4 clusters, first it attempts to call EKS describe-cluster to find the `serviceIpv4Cidr`.
/// If that works, it returns the expected cluster DNS IP address which is obtained by substituting
/// `10` for the last octet. If the EKS call is not successful, it falls back to using IMDS MAC CIDR
/// blocks to return one of two default addresses.
async fn generate_cluster_dns_ip(
    client: &mut ImdsClient,
    aws_k8s_info: &mut AwsK8sInfo,
) -> Result<()> {
    if aws_k8s_info.cluster_dns_ip.is_some() {
        return Ok(());
    }

    // Retrieve the kubernetes network configuration for the EKS cluster
    let ip_addr = if let Some(ip) = get_eks_network_config(aws_k8s_info).await? {
        ip.clone()
    } else {
        // If we were unable to obtain or parse the cidr range from EKS, fallback to one of two default
        // values based on the IPv4 cidr range of our primary network interface
        get_ipv4_cluster_dns_ip_from_imds_mac(client).await?
    };

    aws_k8s_info.cluster_dns_ip = Some(KubernetesClusterDnsIp::Scalar(
        IpAddr::from_str(ip_addr.as_str()).context(error::BadIpSnafu {
            ip: ip_addr.clone(),
        })?,
    ));
    Ok(())
}

/// Retrieves the ip address from the kubernetes network configuration for the
/// EKS Cluster
async fn get_eks_network_config(aws_k8s_info: &AwsK8sInfo) -> Result<Option<String>> {
    if let (Some(region), Some(cluster_name)) = (
        aws_k8s_info.region.as_ref(),
        aws_k8s_info.cluster_name.as_ref(),
    ) {
        if let Ok(config) = eks::get_cluster_network_config(
            region,
            cluster_name,
            aws_k8s_info.https_proxy.clone(),
            aws_k8s_info.no_proxy.clone(),
        )
        .await
        .context(error::EksSnafu)
        {
            // Derive cluster-dns-ip from the service IPv4 CIDR
            if let Some(ipv4_cidr) = config.service_ipv4_cidr {
                if let Ok(dns_ip) = get_dns_from_ipv4_cidr(&ipv4_cidr) {
                    return Ok(Some(dns_ip));
                }
            }
        }
    }
    Ok(None)
}

/// Replicates [this] logic from the EKS AMI:
///
/// ```sh
/// DNS_CLUSTER_IP=${SERVICE_IPV4_CIDR%.*}.10
/// ```
/// [this]: https://github.com/awslabs/amazon-eks-ami/blob/732b6b2/files/bootstrap.sh#L335
fn get_dns_from_ipv4_cidr(cidr: &str) -> Result<String> {
    let mut split: Vec<&str> = cidr.split('.').collect();
    ensure!(
        split.len() == 4,
        error::CidrParseSnafu {
            cidr,
            reason: format!("expected 4 components but found {}", split.len())
        }
    );
    split[3] = "10";
    Ok(split.join("."))
}

/// Gets gets the the first VPC IPV4 CIDR block from IMDS. If it starts with `10`, returns
/// `10.100.0.10`, otherwise returns `172.20.0.10`
async fn get_ipv4_cluster_dns_ip_from_imds_mac(client: &mut ImdsClient) -> Result<String> {
    // Take the first (primary) MAC address. Others may exist from attached ENIs.
    let mac = client
        .fetch_mac_addresses()
        .await
        .context(error::ImdsRequestSnafu)?
        .context(error::ImdsNoneSnafu {
            what: "mac addresses",
        })?
        .first()
        .context(error::ImdsNoneSnafu {
            what: "mac addresses",
        })?
        .clone();

    // Take the first CIDR block for the primary MAC.
    let cidr_block = client
        .fetch_cidr_blocks_for_mac(&mac)
        .await
        .context(error::ImdsRequestSnafu)?
        .context(error::ImdsNoneSnafu {
            what: "CIDR blocks",
        })?
        .first()
        .context(error::ImdsNoneSnafu {
            what: "CIDR blocks",
        })?
        .clone();

    // Infer the cluster DNS based on the CIDR block.
    let dns = if cidr_block.starts_with("10.") {
        DEFAULT_10_RANGE_DNS_CLUSTER_IP
    } else {
        DEFAULT_DNS_CLUSTER_IP
    }
    .to_string();
    Ok(dns)
}

/// Gets the IP address that should be associated with the node.
async fn generate_node_ip(client: &mut ImdsClient, aws_k8s_info: &mut AwsK8sInfo) -> Result<()> {
    if aws_k8s_info.node_ip.is_some() {
        return Ok(());
    }
    // Ensure that this was set in case changes to main occur
    generate_cluster_dns_ip(client, aws_k8s_info).await?;
    let cluster_dns_ip = aws_k8s_info
        .cluster_dns_ip
        .as_ref()
        .and_then(|x| x.iter().next())
        .context(error::NoIpSnafu)?;
    // If the cluster DNS IP is an IPv4 address, retrieve the IPv4 address for the instance.
    // If the cluster DNS IP is an IPv6 address, retrieve the IPv6 address for the instance.
    let node_ip = match cluster_dns_ip {
        IpAddr::V4(_) => client
            .fetch_local_ipv4_address()
            .await
            .context(error::ImdsRequestSnafu)?
            .context(error::ImdsNoneSnafu {
                what: "node ipv4 address",
            }),
        IpAddr::V6(_) => client
            .fetch_primary_ipv6_address()
            .await
            .context(error::ImdsRequestSnafu)?
            .context(error::ImdsNoneSnafu {
                what: "ipv6s associated with primary network interface",
            }),
    }?;
    aws_k8s_info.node_ip = Some(node_ip);
    Ok(())
}

/// Gets the provider ID that should be associated with the node
async fn generate_provider_id(
    client: &mut ImdsClient,
    aws_k8s_info: &mut AwsK8sInfo,
) -> Result<()> {
    if aws_k8s_info.provider_id.is_some()
        || NO_HOSTNAME_VARIANTS.contains(&aws_k8s_info.variant_id.as_str())
    {
        return Ok(());
    }

    let instance_id = client
        .fetch_instance_id()
        .await
        .context(error::ImdsRequestSnafu)?
        .context(error::ImdsNoneSnafu {
            what: "instance ID",
        })?;

    let zone = client
        .fetch_zone()
        .await
        .context(error::ImdsRequestSnafu)?
        .context(error::ImdsNoneSnafu { what: "zone" })?;

    aws_k8s_info.provider_id = Some(format!("aws:///{}/{}", zone, instance_id));
    Ok(())
}

async fn generate_private_dns_name(
    client: &mut ImdsClient,
    aws_k8s_info: &mut AwsK8sInfo,
) -> Result<()> {
    if aws_k8s_info.hostname_override.is_some() {
        return Ok(());
    }

    let region = aws_k8s_info
        .region
        .as_ref()
        .context(error::AwsRegionSnafu)?;
    let instance_id = client
        .fetch_instance_id()
        .await
        .context(error::ImdsRequestSnafu)?
        .context(error::ImdsNoneSnafu {
            what: "instance ID",
        })?;
    aws_k8s_info.hostname_override = Some(
        ec2::get_private_dns_name(
            region,
            &instance_id,
            aws_k8s_info.https_proxy.clone(),
            aws_k8s_info.no_proxy.clone(),
        )
        .await
        .context(error::Ec2Snafu)?,
    );
    Ok(())
}

async fn run() -> Result<()> {
    let mut client = ImdsClient::new();
    let mut aws_k8s_info = api::get_aws_k8s_info().await.context(error::AwsInfoSnafu)?;

    generate_cluster_dns_ip(&mut client, &mut aws_k8s_info).await?;
    generate_node_ip(&mut client, &mut aws_k8s_info).await?;
    generate_max_pods(&mut client, &mut aws_k8s_info).await?;
    generate_provider_id(&mut client, &mut aws_k8s_info).await?;
    generate_private_dns_name(&mut client, &mut aws_k8s_info).await?;

    let settings = serde_json::to_value(&aws_k8s_info).context(error::SerializeSnafu)?;
    let generated_settings: serde_json::Value = serde_json::json!({
        "kubernetes": settings
    });
    let json_str = generated_settings.to_string();
    api::client_command(&["set", "-j", json_str.as_str()])
        .await
        .context(error::SetFailureSnafu)?;

    Ok(())
}

// Returning a Result from main makes it print a Debug representation of the error, but with Snafu
// we have nice Display representations of the error, so we wrap "main" (run) and print any error.
// https://github.com/shepmaster/snafu/issues/110
#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{}", e);
        process::exit(1);
    }
}

#[test]
fn test_get_dns_from_cidr_ok() {
    let input = "123.456.789.0/123";
    let expected = "123.456.789.10";
    let actual = get_dns_from_ipv4_cidr(input).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_get_dns_from_cidr_err() {
    let input = "123_456_789_0/123";
    let result = get_dns_from_ipv4_cidr(input);
    assert!(result.is_err());
}
