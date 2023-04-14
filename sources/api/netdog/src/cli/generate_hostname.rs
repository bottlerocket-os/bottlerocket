use super::{error, print_json, Result};
use crate::CURRENT_IP;
use argh::FromArgs;
use dns_lookup::lookup_addr;
use snafu::ResultExt;
use tokio::time::Duration;
use tokio_retry::{
    strategy::{jitter, FibonacciBackoff},
    Retry,
};

use std::fs;
use std::net::IpAddr;
use std::str::FromStr;

// Maximum number of retries for querying DNS for the hostname.
const DNS_QUERY_MAX_RETRIES: usize = 3;
// Starting retry interval for fibonacci backoff strategy.
const DNS_QUERY_FIBONACCI_RETRY_INTERVAL: u64 = 50;
// getnameinfo() takes about 5 seconds to time out - we add our own brief delay between retries.

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "generate-hostname")]
/// Generate hostname from DNS reverse lookup or use current IP
pub(crate) struct GenerateHostnameArgs {}

/// Attempt to resolve assigned IP address, if unsuccessful use the IP as the hostname.
///
/// The result is returned as JSON. (intended for use as a settings generator)
pub(crate) async fn run() -> Result<()> {
    let ip_string = fs::read_to_string(CURRENT_IP)
        .context(error::CurrentIpReadFailedSnafu { path: CURRENT_IP })?;
    let ip = IpAddr::from_str(&ip_string).context(error::IpFromStringSnafu { ip: &ip_string })?;

    // First, try any platform-specific mechanism that exists.
    let hostname =
        // The interaction between async and `Result.or_else()` chaining makes this the most ergonomic way to write this...
        platform::query_platform_hostname()
            .await
            .map_err(|e| {
                eprintln!("Failed to find hostname from platform: {}", e);
                e
            }).ok().flatten();

    // If the platform-specific mechanism fails, attempt to lookup the hostname via DNS
    let hostname = if hostname.is_none() {
        Retry::spawn(retry_strategy(), || async { lookup_addr(&ip) })
            .await
            .map_err(|e| {
                eprintln!("Reverse DNS lookup failed: {}", e);
                e
            })
            .ok()
    } else {
        hostname
    }
    // If no hostname has been determined we return the IP address of the host.
    .unwrap_or(ip_string);

    // sundog expects JSON-serialized output
    print_json(hostname)
}

/// Returns an iterator of Durations to wait between retries of DNS queries.
fn retry_strategy() -> impl Iterator<Item = Duration> {
    FibonacciBackoff::from_millis(DNS_QUERY_FIBONACCI_RETRY_INTERVAL)
        .map(jitter)
        .take(DNS_QUERY_MAX_RETRIES)
}

mod platform {
    use snafu::Snafu;

    #[cfg(variant_platform = "aws")]
    use snafu::ResultExt;

    #[cfg(variant_platform = "aws")]
    const IMDS_RETRY_TIMEOUT: tokio::time::Duration = tokio::time::Duration::from_secs(3);

    /// Query IMDS on AWS platforms to determine hostname.
    #[cfg(variant_platform = "aws")]
    pub(super) async fn query_platform_hostname() -> Result<Option<String>> {
        let mut imdsclient = imdsclient::ImdsClient::new().with_timeout(IMDS_RETRY_TIMEOUT);
        let imds_hostname = imdsclient.fetch_hostname().await.context(ImdsLookupSnafu)?;
        Ok(imds_hostname)
    }

    #[cfg(not(variant_platform = "aws"))]
    pub(super) async fn query_platform_hostname() -> Result<Option<String>> {
        Ok(None)
    }

    /// Provide a Snafu error type for calling platform-specific hostname-fetching errors.
    ///
    /// This allows us to avoid conditionally compiling error cases into `crate::error::Error`.
    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(super) enum Error {
        #[cfg(variant_platform = "aws")]
        #[snafu(display("Failed to lookup hostname in imds: {}", source))]
        ImdsLookup { source: imdsclient::Error },
    }

    pub(super) type Result<T> = std::result::Result<T, Error>;
}
