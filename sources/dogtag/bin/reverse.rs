use dns_lookup::lookup_addr;
use dogtag::Cli;
use snafu::ResultExt;
use tokio::time::Duration;
use tokio_retry::{
    strategy::{jitter, FibonacciBackoff},
    Retry,
};

// Maximum number of retries for querying DNS for the hostname.
const DNS_QUERY_MAX_RETRIES: usize = 3;
// Starting retry interval for fibonacci backoff strategy.
const DNS_QUERY_FIBONACCI_RETRY_INTERVAL: u64 = 50;
// getnameinfo() takes about 5 seconds to time out - we add our own brief delay between retries.

type Result<T> = std::result::Result<T, error::Error>;

/// Looks up the public hostname by using dns-lookup to
/// resolve it from the ip address provided
#[tokio::main]
async fn main() -> Result<()> {
    let cli: Cli = argh::from_env();
    let ip: std::net::IpAddr = cli.ip_address.parse().context(error::InvalidIpSnafu)?;
    let hostname = Retry::spawn(retry_strategy(), || async { lookup_addr(&ip) })
        .await
        .map_err(|e| error::Error::Lookup {
            source: Box::new(e),
        })?;
    println!("{}", hostname);
    Ok(())
}

/// Returns an iterator of Durations to wait between retries of DNS queries.
fn retry_strategy() -> impl Iterator<Item = Duration> {
    FibonacciBackoff::from_millis(DNS_QUERY_FIBONACCI_RETRY_INTERVAL)
        .map(jitter)
        .take(DNS_QUERY_MAX_RETRIES)
}

mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(super) enum Error {
        #[snafu(display("Invalid ip address passed to tool {}", source))]
        InvalidIp {
            #[snafu(source(from(std::net::AddrParseError, Box::new)))]
            source: Box<std::net::AddrParseError>,
        },
        #[snafu(display("Failed to lookup hostname via dns {}", source))]
        Lookup {
            #[snafu(source(from(std::io::Error, Box::new)))]
            source: Box<std::io::Error>,
        },
    }
}
