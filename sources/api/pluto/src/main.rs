#![deny(rust_2018_idioms)]

/*!
# Introduction

pluto is called by sundog to generate settings required by Kubernetes.
This is done dynamically because we require access to dynamic networking
and cluster setup information.

It uses IMDS to get information such as:

- Instance Type
- Node IP

It uses EKS to get information such as:

- Service IPV4 CIDR

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
mod eks;

use imdsclient::ImdsClient;
use snafu::{ensure, OptionExt, ResultExt};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::string::String;
use std::{env, process};

// This is the default DNS unless our CIDR block begins with "10."
const DEFAULT_DNS_CLUSTER_IP: &str = "10.100.0.10";
// If our CIDR block begins with "10." this is our DNS.
const DEFAULT_10_RANGE_DNS_CLUSTER_IP: &str = "172.20.0.10";

const ENI_MAX_PODS_PATH: &str = "/usr/share/eks/eni-max-pods";

mod error {
    use crate::eks;
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(super) enum PlutoError {
        #[snafu(display("Unable to parse CIDR '{}': {}", cidr, reason))]
        CidrParse { cidr: String, reason: String },

        #[snafu(display("IMDS request failed: {}", source))]
        ImdsRequest { source: imdsclient::Error },

        #[snafu(display("IMDS client failed: {}", source))]
        ImdsClient { source: imdsclient::Error },

        #[snafu(display("IMDS request failed: No '{}' found", what))]
        ImdsNone { what: String },

        #[snafu(display("Error deserializing response into JSON from {}: {}", uri, source))]
        ImdsJson {
            uri: String,
            source: serde_json::error::Error,
        },

        #[snafu(display(
            "Error serializing to JSON from command output '{}': {}",
            output,
            source
        ))]
        OutputJson {
            output: String,
            source: serde_json::error::Error,
        },

        #[snafu(display("{}", source))]
        EksError { source: eks::Error },

        #[snafu(display("Failed to open eni-max-pods file at {}: {}", path, source))]
        EniMaxPodsFile {
            path: &'static str,
            source: std::io::Error,
        },

        #[snafu(display("Failed to parse setting {} as u32: {}", setting, source))]
        ParseToU32 {
            setting: String,
            source: std::num::ParseIntError,
        },

        #[snafu(display("Failed to read line: {}", source))]
        IoReadLine { source: std::io::Error },

        #[snafu(display(
            "Unable to find maximum number of pods supported for instance-type {}",
            instance_type
        ))]
        NoInstanceTypeMaxPods { instance_type: String },
    }
}

use error::PlutoError;

type Result<T> = std::result::Result<T, PlutoError>;

async fn get_max_pods(client: &mut ImdsClient) -> Result<String> {
    let instance_type = client
        .fetch_instance_type()
        .await
        .context(error::ImdsRequest)?
        .context(error::ImdsNone {
            what: "instance_type",
        })?;

    // Find the corresponding maximum number of pods supported by this instance type
    let file = BufReader::new(
        File::open(ENI_MAX_PODS_PATH).context(error::EniMaxPodsFile {
            path: ENI_MAX_PODS_PATH,
        })?,
    );
    for line in file.lines() {
        let line = line.context(error::IoReadLine)?;
        // Skip the comments in the file
        if line.trim_start().starts_with('#') {
            continue;
        }
        let tokens: Vec<_> = line.split_whitespace().collect();
        if tokens.len() == 2 && tokens[0] == instance_type {
            return Ok(tokens[1].to_string());
        }
    }
    error::NoInstanceTypeMaxPods { instance_type }.fail()
}

/// Returns the cluster's DNS IPV4 address. First it attempts to call EKS describe-cluster to find
/// the `serviceIPv4CIDR`. If that works, it returns the expected cluster DNS IP address which is
/// obtained by substituting `10` for the last octet. If the EKS call is not successful, it falls
/// back to using IMDS MAC CIDR blocks to return one of two default addresses.
async fn get_cluster_dns_ip(client: &mut ImdsClient) -> Result<String> {
    // try calling eks describe-cluster to figure out the dns cluster ip
    if let Some(dns_ip) = get_dns_from_eks().await {
        // we were able to calculate the dns ip from the cidr range we received from eks
        return Ok(dns_ip);
    }

    // we were unable to obtain or parse the cidr range from eks, fallback to one of two default
    // values based on the cidr range of our primary network interface
    get_cluster_dns_from_imds_mac(client).await
}

/// Gets the Service IPV4 CIDR setting from EKS and parses it to calculate the cluster DNS IP.
/// Prints the error and returns `None` if anything goes wrong.
async fn get_dns_from_eks() -> Option<String> {
    let aws_k8s_info = match api::get_aws_k8s_info().await {
        Ok(value) => value,
        Err(e) => {
            eprintln!(
                "Unable to get region and cluster name from Bottlerocket API, using default DNS IP: {}",
                e
            );
            return None;
        }
    };

    eks::get_cluster_cidr(&aws_k8s_info.region, &aws_k8s_info.cluster_name)
        .await
        .context(error::EksError)
        .and_then(|cidr| get_dns_from_cidr(&cidr))
        .map_err(|e| eprintln!("Unable to parse CIDR from EKS, using default DNS IP: {}", e))
        .ok()
}

/// Replicates [this] logic from the EKS AMI:
///
/// ```sh
/// DNS_CLUSTER_IP=${SERVICE_IPV4_CIDR%.*}.10
/// ```
/// [this]: https://github.com/awslabs/amazon-eks-ami/blob/732b6b2/files/bootstrap.sh#L335
fn get_dns_from_cidr(cidr: &str) -> Result<String> {
    let mut split: Vec<&str> = cidr.split('.').collect();
    ensure!(
        split.len() == 4,
        error::CidrParse {
            cidr,
            reason: format!("expected 4 components but found {}", split.len())
        }
    );
    split[3] = "10";
    Ok(split.join("."))
}

/// Gets gets the the first VPC IPV4 CIDR block from IMDS. If it starts with `10`, returns
/// `10.100.0.10`, otherwise returns `172.20.0.10`
async fn get_cluster_dns_from_imds_mac(client: &mut ImdsClient) -> Result<String> {
    // Take the first (primary) MAC address. Others may exist from attached ENIs.
    let mac = client
        .fetch_mac_addresses()
        .await
        .context(error::ImdsRequest)?
        .context(error::ImdsNone {
            what: "mac addresses",
        })?
        .first()
        .context(error::ImdsNone {
            what: "mac addresses",
        })?
        .clone();

    // Take the first CIDR block for the primary MAC.
    let cidr_block = client
        .fetch_cidr_blocks_for_mac(&mac)
        .await
        .context(error::ImdsRequest)?
        .context(error::ImdsNone {
            what: "CIDR blocks",
        })?
        .first()
        .context(error::ImdsNone {
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

async fn get_node_ip(client: &mut ImdsClient) -> Result<String> {
    client
        .fetch_local_ipv4_address()
        .await
        .context(error::ImdsRequest)?
        .context(error::ImdsNone { what: "node ip" })
}

/// Print usage message.
fn usage() -> ! {
    let program_name = env::args().next().unwrap_or_else(|| "program".to_string());
    eprintln!(
        r"Usage: {} [max-pods | cluster-dns-ip | node-ip]",
        program_name
    );
    process::exit(1);
}

/// Parses args for the setting key name.
fn parse_args(mut args: env::Args) -> String {
    args.nth(1).unwrap_or_else(|| usage())
}

async fn run() -> Result<()> {
    let setting_name = parse_args(env::args());
    let mut client = ImdsClient::new().await.context(error::ImdsClient)?;

    let setting = match setting_name.as_ref() {
        "cluster-dns-ip" => get_cluster_dns_ip(&mut client).await,
        "node-ip" => get_node_ip(&mut client).await,
        // If we want to specify a reasonable default in a template, we can exit 2 to tell
        // sundog to skip this setting.
        "max-pods" => get_max_pods(&mut client)
            .await
            .map_err(|_| process::exit(2)),

        _ => usage(),
    }?;

    // sundog expects JSON-serialized output so that many types can be represented, allowing the
    // API model to use more accurate types.

    // 'max_pods' setting is an unsigned integer, convert 'settings' to u32 before serializing to JSON
    if setting_name == "max-pods" {
        let max_pods = serde_json::to_string(
            &setting
                .parse::<u32>()
                .context(error::ParseToU32 { setting: &setting })?,
        )
        .context(error::OutputJson { output: &setting })?;
        println!("{}", max_pods);
    } else {
        let output =
            serde_json::to_string(&setting).context(error::OutputJson { output: &setting })?;
        println!("{}", output);
    }
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
    let actual = get_dns_from_cidr(input).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_get_dns_from_cidr_err() {
    let input = "123_456_789_0/123";
    let result = get_dns_from_cidr(input);
    assert!(result.is_err());
}
