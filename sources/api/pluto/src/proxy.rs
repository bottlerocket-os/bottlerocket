use hyper::Uri;
use hyper_proxy::{Proxy, ProxyConnector};
use hyper_rustls::HttpsConnectorBuilder;
use snafu::{ResultExt, Snafu};
use std::env;

#[derive(Debug, Snafu)]
pub(super) enum Error {
    #[snafu(display("Unable to parse '{}' as URI: {}", input, source))]
    UriParse {
        input: String,
        source: hyper::http::uri::InvalidUri,
    },

    #[snafu(display("Failed to create proxy creator: {}", source))]
    ProxyConnector { source: std::io::Error },
}

type Result<T> = std::result::Result<T, Error>;

/// Fetches `HTTPS_PROXY` and `NO_PROXY` variables from the process environment.
pub(crate) fn fetch_proxy_env() -> (Option<String>, Option<String>) {
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
    (https_proxy, no_proxy)
}

/// Setups a hyper-based HTTP client configured with a proxy connector.
pub(crate) fn setup_http_client(
    https_proxy: String,
    no_proxy: Option<String>,
) -> Result<impl Into<aws_smithy_client::http_connector::HttpConnector>> {
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
        proxy_uri = format!("http://{}", https_proxy)
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
    Ok(aws_smithy_client::hyper_ext::Adapter::builder().build(proxy_connector))
}
