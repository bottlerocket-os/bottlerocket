use crate::config::AwsConfig;
use rusoto_core::Region;
use snafu::ResultExt;

#[macro_use]
pub(crate) mod client;

pub(crate) mod ami;

/// Builds a Region from the given region name, and uses the custom endpoint from the AWS config,
/// if specified in aws.region.REGION.endpoint.
fn region_from_string(name: &str, aws: &AwsConfig) -> Result<Region> {
    let maybe_endpoint = aws.region.get(name).and_then(|r| r.endpoint.clone());
    Ok(match maybe_endpoint {
        Some(endpoint) => Region::Custom {
            name: name.to_string(),
            endpoint,
        },
        None => name.parse().context(error::ParseRegion { name })?,
    })
}

mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(crate) enum Error {
        #[snafu(display("Failed to parse region '{}': {}", name, source))]
        ParseRegion {
            name: String,
            source: rusoto_signature::region::ParseRegionError,
        },
    }
}
pub(crate) use error::Error;
type Result<T> = std::result::Result<T, error::Error>;
