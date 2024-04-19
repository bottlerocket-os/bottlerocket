/*!
dogtag detects the hostname of a bottlerocket server/instance. It's used to generate settings.network.hostname.
To accomplish this, it uses a set of standalone binaries in /usr/libexec/hostname-detectors that detect the hostname via different methods.

Currently, bottlerocket ships with two hostname detector binaries:

20-imds - Fetches hostname from EC2 Instance Metadata Service
10-reverse-dns - Uses reverse DNS lookup to resolve the hostname

dogtag runs the detectors in /usr/libexec/hostname-detectors in reverse alphanumerical order until one of them returns a hostname,
at which point it will exit early and print the returned hostname to stdout. If none of the detectors detect the hostname the
ip address is returned.
 */
use argh::FromArgs;
use log::debug;
use snafu::ResultExt;
use std::net::IpAddr;
use std::{path::PathBuf, process};
use walkdir::WalkDir;

const DOGTAG_BIN_PATH: &str = "/usr/libexec/hostname-detectors";

/// Cli defines the standard cmdline interface for all hostname handlers
#[derive(FromArgs)]
#[argh(description = "hostname resolution tool")]
pub struct Cli {
    #[argh(option)]
    #[argh(description = "ip_address of the host")]
    pub ip_address: String,
}

pub type Result<T> = std::result::Result<T, error::Error>;

/// find_hostname will utilize the helpers located in /usr/libexec/hostname-detectors to try and discover the hostname
pub async fn find_hostname(ip_addr: IpAddr) -> Result<String> {
    debug!(
        "attempting to discover hostname detectors in {}",
        DOGTAG_BIN_PATH
    );
    // We want to do reverse sort as we want to prioritize higher numbers first
    // this is because it makes it easier to add more of these and not have to worry about
    // bumping the binary name for existing ones
    let mut hostname_helpers: Vec<PathBuf> = WalkDir::new(DOGTAG_BIN_PATH)
        .max_depth(1)
        .min_depth(1)
        .sort_by_file_name()
        .into_iter()
        .collect::<std::result::Result<Vec<_>, _>>()
        .context(error::WalkdirSnafu)?
        .into_iter()
        .map(|x| x.into_path())
        .collect();
    hostname_helpers.reverse();

    for helper in hostname_helpers.iter() {
        let output = process::Command::new(helper)
            .arg("--ip-address")
            .arg(ip_addr.to_string())
            .output()
            .map(Some)
            .unwrap_or(None);
        if let Some(output) = output.as_ref() {
            // Read the std output
            if output.status.success() {
                let hostname = String::from_utf8_lossy(output.stdout.as_slice()).to_string();
                return Ok(hostname.trim().to_string());
            }
        }
    }
    // If we fail to find a hostname return the provided ip address sanitized
    Ok(ip_addr.to_string().replace(':', "-"))
}

pub mod error {
    use snafu::Snafu;

    #[derive(Snafu, Debug)]
    #[snafu(visibility(pub))]
    pub enum Error {
        #[snafu(display("Failed to detect hostname due to an io error: {}", source))]
        Walkdir { source: walkdir::Error },
        #[snafu(display(
            "Failed to detect hostname, no helpers are installed in path or io error occurred"
        ))]
        NoHelper,
        #[snafu(display(
            "Failed to detect hostname, no helper installed was able to resolve the hostname"
        ))]
        FailHostname,
    }
}
