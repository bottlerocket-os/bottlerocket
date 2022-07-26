use aws_sdk_eks::model::KubernetesNetworkConfigResponse;
use aws_types::region::Region;
use hyper::http::uri::InvalidUri;
use hyper::Uri;
use hyper_proxy::{Proxy, ProxyConnector};
use hyper_rustls::HttpsConnectorBuilder;
use snafu::{OptionExt, ResultExt, Snafu};
use std::env;

pub(crate) type ClusterNetworkConfig = KubernetesNetworkConfigResponse;

#[derive(Debug, Snafu)]
pub(super) enum Error {
    #[snafu(display("Error describing cluster: {}", source))]
    DescribeCluster {
        source: aws_sdk_eks::types::SdkError<aws_sdk_eks::error::DescribeClusterError>,
    },

    #[snafu(display("Missing field '{}' EKS response", field))]
    Missing { field: &'static str },

    #[snafu(display("Unable to parse '{}' as URI: {}", input, source))]
    UriParse { input: String, source: InvalidUri },

    #[snafu(display("Failed to create proxy creator: {}", source))]
    ProxyConnector { source: std::io::Error },
}

type Result<T> = std::result::Result<T, Error>;

/// Returns the cluster's [kubernetesNetworkConfig] by calling the EKS API.
/// (https://docs.aws.amazon.com/eks/latest/APIReference/API_KubernetesNetworkConfigResponse.html)
pub(super) async fn get_cluster_network_config(
    region: &str,
    cluster: &str,
) -> Result<ClusterNetworkConfig> {
    // Respect proxy environment variables when making AWS EKS API requests
    let https_proxy = ["https_proxy", "HTTPS_PROXY"]
        .iter()
        .map(env::var)
        .find(|env_var| *env_var != Err(env::VarError::NotPresent))
        .and_then(|s| s.ok());
    let no_proxy = ["no_proxy", "NO_PROXY"]
        .iter()
        .map(env::var)
        .find(|env_var| *env_var != Err(env::VarError::NotPresent))
        .and_then(|s| s.ok());

    let config = aws_config::from_env()
        .region(Region::new(region.to_owned()))
        .load()
        .await;

    let client = if let Some(https_proxy) = https_proxy {
        // Determines whether a request of a given scheme, host and port should be proxied
        // according to `https_proxy` and `no_proxy`.
        let intercept = move |scheme: Option<&str>, host: Option<&str>, _port| {
            if let Some(host) = host {
                if let Some(no_proxy) = &no_proxy {
                    if scheme != Some("https") {
                        return false;
                    }
                    let no_proxy_hosts: Vec<&str> = no_proxy.split(',').map(|s| s.trim()).collect();
                    if no_proxy_hosts.iter().any(|s| *s == "*") {
                        // Don't proxy anything
                        return false;
                    }
                    // If the host matches one of the no proxy list entries, return false (don't proxy)
                    // Note that we're not doing anything fancy here for checking `no_proxy` since
                    // we only expect requests here to be going out to some AWS API endpoint.
                    return !no_proxy_hosts.iter().any(|no_proxy_host| {
                        !no_proxy_host.is_empty() && host.ends_with(no_proxy_host)
                    });
                }
                true
            } else {
                false
            }
        };
        let mut proxy_uri = https_proxy.parse::<Uri>().context(UriParseSnafu {
            input: &https_proxy,
        })?;
        // If the proxy's URI doesn't have a scheme, assume HTTP for the scheme and let the proxy
        // server forward HTTPS connections and start a tunnel.
        if proxy_uri.scheme().is_none() {
            proxy_uri =
                format!("http://{}", https_proxy)
                    .parse::<Uri>()
                    .context(UriParseSnafu {
                        input: &https_proxy,
                    })?;
        }
        let proxy = Proxy::new(intercept, proxy_uri);
        let https_connector = HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_or_http()
            .enable_http2()
            .build();
        let proxy_connector =
            ProxyConnector::from_proxy(https_connector, proxy).context(ProxyConnectorSnafu)?;
        let http_client = aws_smithy_client::hyper_ext::Adapter::builder().build(proxy_connector);
        let eks_config = aws_sdk_eks::config::Builder::from(&config).build();
        aws_sdk_eks::Client::from_conf_conn(eks_config, http_client)
    } else {
        aws_sdk_eks::Client::new(&config)
    };

    client
        .describe_cluster()
        .name(cluster.to_owned())
        .send()
        .await
        .context(DescribeClusterSnafu)?
        .cluster
        .context(MissingSnafu { field: "cluster" })?
        .kubernetes_network_config
        .context(MissingSnafu {
            field: "kubernetes_network_config",
        })
}
