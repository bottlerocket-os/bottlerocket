use dogtag::Cli;
use imdsclient::ImdsClient;
use snafu::{OptionExt, ResultExt};

type Result<T> = std::result::Result<T, error::Error>;

/// Implements a hostname lookup tool by fetching the public hostname
/// from the instance metadata via IMDS. It will interface with IMDS
/// via:
///
/// * Check for IPv6, default to IPv4 if not available
/// * Check for IMDSv2, fallback to IMDSv1 if not enabled
#[tokio::main]
async fn main() -> Result<()> {
    // Even though for this helper we do not need any arguments
    // still validate to ensure the helper follows standards.
    let _: Cli = argh::from_env();
    let mut imds = ImdsClient::new();
    let hostname = imds
        .fetch_hostname()
        .await
        .context(error::ImdsSnafu)?
        .context(error::NoHostnameSnafu)?;
    println!("{}", hostname);
    Ok(())
}

mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(super) enum Error {
        #[snafu(display("failed to fetch hostname from IMDS: {}", source))]
        Imds { source: imdsclient::Error },
        #[snafu(display("no hostname returned by imds"))]
        NoHostname,
    }
}
