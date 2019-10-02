#![deny(rust_2018_idioms)]

/*!
# Introduction

pluto is called by sundog to generate settings required by Kubernetes.
This is done dynamically because we require access to dynamic networking
setup information.

It makes calls to IMDS to get meta data:

- Cluster DNS
- Node IP
- POD Infra Container Image
*/
use std::{env, process};

use snafu::{OptionExt, ResultExt};

// This is the default DNS unless our CIDR block begins with "10."
const DEFAULT_DNS_CLUSTER_IP: &str = "10.100.0.10";
// If our CIDR block begins with "10." this is our DNS.
const DEFAULT_10_RANGE_DNS_CLUSTER_IP: &str = "172.20.0.10";

// Instance Meta Data Service
const IMDS_URI: &str = "http://169.254.169.254/2018-09-24";

const PAUSE_CONTAINER_ACCOUNT: &str = "602401143452";
const PAUSE_CONTAINER_VERSION: &str = "3.1";

mod error {
    use snafu::Snafu;

    // Taken from sundog.
    fn code(source: &reqwest::Error) -> String {
        source
            .status()
            .as_ref()
            .map(|i| i.as_str())
            .unwrap_or("Unknown")
            .to_string()
    }

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(super) enum PlutoError {
        #[snafu(display("Error '{}' to '{}': {}", code(&source), path, source))]
        ImdsRequest {
            path: String,
            source: reqwest::Error,
        },

        #[snafu(display("Error '{}' from '{}': {}", code(&source), path, source))]
        ImdsResponse {
            path: String,
            source: reqwest::Error,
        },

        #[snafu(display("Error getting text response from {}: {}", path, source))]
        ImdsText {
            path: String,
            source: reqwest::Error,
        },

        #[snafu(display("Error deserializing response into JSON from {}: {}", path, source))]
        ImdsJson {
            path: String,
            source: serde_json::error::Error,
        },

        #[snafu(display(
            "Missing 'region' key in Instance Identity Document from IMDS: {}",
            path
        ))]
        MissingRegion { path: String },

        #[snafu(display("Missing MAC address from IMDS: {}", path))]
        MissingMac { path: String },

        #[snafu(display("Invalid machine architecture, not one of 'x86_64' or 'aarch64'"))]
        UnknownArchitecture,
    }
}

use error::PlutoError;

type Result<T> = std::result::Result<T, PlutoError>;

fn get_text_from_imds(client: &reqwest::Client, path: &str) -> Result<String> {
    client
        .get(&format!("{}{}", IMDS_URI, path))
        .send()
        .context(error::ImdsRequest {
            path: path.to_string(),
        })?
        .error_for_status()
        .context(error::ImdsResponse {
            path: path.to_string(),
        })?
        .text()
        .context(error::ImdsText {
            path: path.to_string(),
        })
}

fn get_cluster_dns_ip() -> Result<String> {
    let client = reqwest::Client::new();

    let macs_path = "/meta-data/network/interfaces/macs";
    let macs = get_text_from_imds(&client, macs_path)?;
    // Take the first (primary) MAC address. Others will exist from attached ENIs.
    let mac = macs.split('\n').next().context(error::MissingMac {
        path: macs_path.to_string(),
    })?;

    // Infer the cluster DNS based on our CIDR blocks.
    let mac_cidr_blocks_path = format!(
        "/meta-data/network/interfaces/macs/{}/vpc-ipv4-cidr-blocks",
        mac
    );
    let mac_cidr_blocks = get_text_from_imds(&client, &mac_cidr_blocks_path)?;

    let dns = if mac_cidr_blocks.starts_with("10.") {
        DEFAULT_10_RANGE_DNS_CLUSTER_IP
    } else {
        DEFAULT_DNS_CLUSTER_IP
    }
    .to_string();

    Ok(dns)
}

fn get_node_ip() -> Result<String> {
    let client = reqwest::Client::new();
    let path = "/meta-data/local-ipv4";
    get_text_from_imds(&client, &path)
}

fn get_pod_infra_container_image() -> Result<String> {
    let client = reqwest::Client::new();

    // Get the region from the correct location.
    let instance_identity_document_path = "/dynamic/instance-identity/document";
    let iid_text = get_text_from_imds(&client, &instance_identity_document_path)?;
    let iid_json: serde_json::Value = serde_json::from_str(&iid_text).context(error::ImdsJson {
        path: instance_identity_document_path.to_string(),
    })?;
    let region = iid_json["region"].as_str().context(error::MissingRegion {
        path: instance_identity_document_path.to_string(),
    })?;

    // Get machine architecture.
    let arch = if cfg!(target_arch = "x86_64") {
        "amd64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        return error::UnknownArchitecture.fail();
    };

    Ok(format!(
        "{}.dkr.ecr.{}.amazonaws.com/eks/pause-{}:{}",
        PAUSE_CONTAINER_ACCOUNT, region, arch, PAUSE_CONTAINER_VERSION
    ))
}

/// Print usage message.
fn usage() -> ! {
    let program_name = env::args().next().unwrap_or_else(|| "program".to_string());
    eprintln!(
        r"Usage: {} [cluster-dns-ip | node-ip | pod-infra-container-image]",
        program_name
    );
    process::exit(2);
}

/// Parses args for the setting key name.
fn parse_args(mut args: env::Args) -> String {
    args.nth(1).unwrap_or_else(|| usage())
}

fn main() -> Result<()> {
    let setting_name = parse_args(env::args());

    let setting = match setting_name.as_ref() {
        "cluster-dns-ip" => get_cluster_dns_ip(),
        "node-ip" => get_node_ip(),
        "pod-infra-container-image" => get_pod_infra_container_image(),
        _ => usage(),
    }?;

    println!("{}", setting);
    Ok(())
}
