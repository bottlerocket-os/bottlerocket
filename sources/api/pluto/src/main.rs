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

use lazy_static::lazy_static;
use reqwest::blocking::Client;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::string::String;
use std::{env, process};

use snafu::{OptionExt, ResultExt};

// This is the default DNS unless our CIDR block begins with "10."
const DEFAULT_DNS_CLUSTER_IP: &str = "10.100.0.10";
// If our CIDR block begins with "10." this is our DNS.
const DEFAULT_10_RANGE_DNS_CLUSTER_IP: &str = "172.20.0.10";

// Instance Meta Data Service
const IMDS_BASE_URL: &str = "http://169.254.169.254/2018-09-24";
// Currently only able to get fetch session tokens from `latest`
// FIXME Pin to a date version that supports IMDSv2 once such a date version is available.
const IMDS_SESSION_TOKEN_ENDPOINT: &str = "http://169.254.169.254/latest/api/token";
const IMDS_NODE_IPV4_ENDPOINT: &str = "http://169.254.169.254/2018-09-24/meta-data/local-ipv4";
const IMDS_MAC_ENDPOINT: &str =
    "http://169.254.169.254/2018-09-24/meta-data/network/interfaces/macs";
const IMDS_INSTANCE_TYPE_ENDPOINT: &str =
    "http://169.254.169.254/2018-09-24/meta-data/instance-type";
const IMDS_INSTANCE_IDENTITY_DOCUMENT_ENDPOINT: &str =
    "http://169.254.169.254/2018-09-24/dynamic/instance-identity/document";

// const ENI_MAX_PODS_PATH: &str = "/usr/share/eks/eni-max-pods";
const ENI_MAX_PODS_PATH: &str = "/usr/share/eks/eni-max-pods-custom-networking";

const PAUSE_CONTAINER_VERSION: &str = "3.1";
lazy_static! {
    /// A map to tell us which account to pull pause container images from for a given region.
    static ref PAUSE_CONTAINER_ACCOUNT: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("af-south-1", "877085696533");
        m.insert("ap-east-1", "800184023465");
        m.insert("ap-northeast-1", "602401143452");
        m.insert("ap-northeast-2", "602401143452");
        m.insert("ap-south-1", "602401143452");
        m.insert("ap-southeast-1", "602401143452");
        m.insert("ap-southeast-2", "602401143452");
        m.insert("ca-central-1", "602401143452");
        m.insert("cn-north-1", "918309763551");
        m.insert("cn-northwest-1", "961992271922");
        m.insert("eu-central-1", "602401143452");
        m.insert("eu-north-1", "602401143452");
        m.insert("eu-south-1", "590381155156");
        m.insert("eu-south-1", "590381155156");
        m.insert("eu-west-1", "602401143452");
        m.insert("eu-west-2", "602401143452");
        m.insert("eu-west-3", "602401143452");
        m.insert("me-south-1", "558608220178");
        m.insert("sa-east-1", "602401143452");
        m.insert("us-east-1", "602401143452");
        m.insert("us-east-2", "602401143452");
        m.insert("us-gov-east-1", "151742754352");
        m.insert("us-gov-west-1", "013241004608");
        m.insert("us-west-1", "602401143452");
        m.insert("us-west-2", "602401143452");
        m
    };
}

/// But if there is a region that does not exist in our map (for example a new
/// region is created or being tested), then we will fall back to this.
const PAUSE_FALLBACK_ACCOUNT: &str = "602401143452";
const PAUSE_FALLBACK_REGION: &str = "us-east-1";

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
        #[snafu(display("Error {}ing '{}': {}", method, uri, source))]
        ImdsRequest {
            method: String,
            uri: String,
            source: reqwest::Error,
        },

        #[snafu(display("Error '{}' from '{}': {}", code(&source), uri, source))]
        ImdsResponse { uri: String, source: reqwest::Error },

        #[snafu(display("Error getting text response from {}: {}", uri, source))]
        ImdsText { uri: String, source: reqwest::Error },

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

        #[snafu(display(
            "Missing 'region' key in Instance Identity Document from IMDS: {}",
            uri
        ))]
        MissingRegion { uri: String },

        #[snafu(display("Missing MAC address from IMDS: {}", uri))]
        MissingMac { uri: String },

        #[snafu(display("Invalid machine architecture, not one of 'x86_64' or 'aarch64'"))]
        UnknownArchitecture,

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

fn get_text_from_imds(client: &Client, uri: &str, session_token: &str) -> Result<String> {
    client
        .get(uri)
        .header("X-aws-ec2-metadata-token", session_token)
        .send()
        .context(error::ImdsRequest { method: "GET", uri })?
        .error_for_status()
        .context(error::ImdsResponse { uri })?
        .text()
        .context(error::ImdsText { uri })
}

fn get_max_pods(client: &Client, session_token: &str) -> Result<String> {
    let instance_type = get_text_from_imds(&client, IMDS_INSTANCE_TYPE_ENDPOINT, session_token)?;
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

fn get_cluster_dns_ip(client: &Client, session_token: &str) -> Result<String> {
    let uri = IMDS_MAC_ENDPOINT;
    let macs = get_text_from_imds(&client, uri, session_token)?;
    // Take the first (primary) MAC address. Others will exist from attached ENIs.
    let mac = macs.split('\n').next().context(error::MissingMac { uri })?;

    // Infer the cluster DNS based on our CIDR blocks.
    let mac_cidr_blocks_uri = format!(
        "{}/meta-data/network/interfaces/macs/{}/vpc-ipv4-cidr-blocks",
        IMDS_BASE_URL, mac
    );
    let mac_cidr_blocks = get_text_from_imds(&client, &mac_cidr_blocks_uri, session_token)?;

    let dns = if mac_cidr_blocks.starts_with("10.") {
        DEFAULT_10_RANGE_DNS_CLUSTER_IP
    } else {
        DEFAULT_DNS_CLUSTER_IP
    }
    .to_string();

    Ok(dns)
}

fn get_node_ip(client: &Client, session_token: &str) -> Result<String> {
    get_text_from_imds(&client, IMDS_NODE_IPV4_ENDPOINT, session_token)
}

fn get_pod_infra_container_image(client: &Client, session_token: &str) -> Result<String> {
    // Get the region from the correct location.
    let uri = IMDS_INSTANCE_IDENTITY_DOCUMENT_ENDPOINT;
    let iid_text = get_text_from_imds(&client, uri, session_token)?;
    let iid_json: serde_json::Value =
        serde_json::from_str(&iid_text).context(error::ImdsJson { uri })?;
    let region = iid_json["region"]
        .as_str()
        .context(error::MissingRegion { uri })?;

    pause_container_uri(region)
}

/// Returns the machine architecture.
fn arch() -> Result<&'static str> {
    if cfg!(target_arch = "x86_64") {
        Ok("amd64")
    } else if cfg!(target_arch = "aarch64") {
        Ok("arm64")
    } else {
        error::UnknownArchitecture.fail()
    }
}

/// Constructs the URI of the pause container image for the given region.  Returns a URI for the
/// default region/account if the region is not mapped.
fn pause_container_uri(region: &str) -> Result<String> {
    // Look up the pause container account, or fall back to the default ID and region
    let (region, account) = match PAUSE_CONTAINER_ACCOUNT.get(&region) {
        Some(account) => (region, *account),
        None => (PAUSE_FALLBACK_REGION, PAUSE_FALLBACK_ACCOUNT),
    };

    Ok(format!(
        "{}.dkr.ecr.{}.amazonaws.com/eks/pause-{}:{}",
        account,
        region,
        arch()?,
        PAUSE_CONTAINER_VERSION
    ))
}

/// Print usage message.
fn usage() -> ! {
    let program_name = env::args().next().unwrap_or_else(|| "program".to_string());
    eprintln!(
        r"Usage: {} [max-pods | cluster-dns-ip | node-ip | pod-infra-container-image]",
        program_name
    );
    process::exit(1);
}

/// Parses args for the setting key name.
fn parse_args(mut args: env::Args) -> String {
    args.nth(1).unwrap_or_else(|| usage())
}

fn run() -> Result<()> {
    let setting_name = parse_args(env::args());

    let client = Client::new();
    // Use IMDSv2 for accessing instance metadata
    let uri = IMDS_SESSION_TOKEN_ENDPOINT;
    let imds_session_token = client
        .put(uri)
        .header("X-aws-ec2-metadata-token-ttl-seconds", "60")
        .send()
        .context(error::ImdsRequest { method: "PUT", uri })?
        .error_for_status()
        .context(error::ImdsResponse { uri })?
        .text()
        .context(error::ImdsText { uri })?;

    let setting = match setting_name.as_ref() {
        "cluster-dns-ip" => get_cluster_dns_ip(&client, &imds_session_token),
        "node-ip" => get_node_ip(&client, &imds_session_token),
        "pod-infra-container-image" => get_pod_infra_container_image(&client, &imds_session_token),

        // If we want to specify a reasonable default in a template, we can exit 2 to tell
        // sundog to skip this setting.
        "max-pods" => get_max_pods(&client, &imds_session_token).map_err(|_| process::exit(2)),

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
fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}

#[cfg(test)]
mod test_pause_container_account {
    use super::{arch, pause_container_uri, PAUSE_CONTAINER_VERSION};

    #[test]
    fn url_eu_west_1() {
        assert_eq!(
            pause_container_uri("eu-west-1").unwrap(),
            format!(
                "602401143452.dkr.ecr.eu-west-1.amazonaws.com/eks/pause-{}:{}",
                arch().unwrap(),
                PAUSE_CONTAINER_VERSION
            )
        );
    }

    #[test]
    fn url_af_south_1() {
        assert_eq!(
            pause_container_uri("af-south-1").unwrap(),
            format!(
                "877085696533.dkr.ecr.af-south-1.amazonaws.com/eks/pause-{}:{}",
                arch().unwrap(),
                PAUSE_CONTAINER_VERSION
            )
        );
    }

    #[test]
    fn url_fallback() {
        assert_eq!(
            pause_container_uri("xy-ztown-1").unwrap(),
            format!(
                "602401143452.dkr.ecr.us-east-1.amazonaws.com/eks/pause-{}:{}",
                arch().unwrap(),
                PAUSE_CONTAINER_VERSION
            )
        );
    }
}
