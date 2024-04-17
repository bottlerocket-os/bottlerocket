use dns_lookup::lookup_addr;
use dogtag::Cli;
use snafu::ResultExt;

type Result<T> = std::result::Result<T, error::Error>;

/// Looks up the public hostname by using dns-lookup to
/// resolve it from the ip address provided
fn main() -> Result<()> {
    let cli: Cli = argh::from_env();
    let ip: std::net::IpAddr = cli.ip_address.parse().context(error::InvalidIpSnafu)?;
    let hostname = lookup_addr(&ip).context(error::LookupSnafu)?;
    println!("{}", hostname);
    Ok(())
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
