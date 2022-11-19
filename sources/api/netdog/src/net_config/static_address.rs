use super::error::{InvalidNetConfigSnafu, Result as ValidateResult};
use crate::net_config::Validate;
use ipnet::IpNet;
use serde::Deserialize;
use snafu::{ensure, ResultExt};
use std::collections::BTreeSet;
use std::convert::TryFrom;
use std::net::IpAddr;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StaticConfigV1 {
    pub(crate) addresses: BTreeSet<IpNet>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RouteV1 {
    pub(crate) to: RouteTo,
    pub(crate) from: Option<IpAddr>,
    pub(crate) via: Option<IpAddr>,
    #[serde(rename = "route-metric")]
    pub(crate) route_metric: Option<u32>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(try_from = "String")]
pub(crate) enum RouteTo {
    DefaultRoute,
    Ip(IpNet),
}

// Allows the user to pass the string "default" or a valid ip address prefix.  We can't use an
// untagged enum for this (#[serde(untagged)]) because "default" directly maps to one of the
// variants.  Serde will only allow the "untagged" attribute if neither variant directly matches.
impl TryFrom<String> for RouteTo {
    type Error = error::Error;

    fn try_from(input: String) -> Result<Self> {
        let input = input.to_lowercase();
        Ok(match input.as_str() {
            "default" => RouteTo::DefaultRoute,
            _ => {
                let ip: IpNet = input
                    .parse()
                    .context(error::InvalidRouteDestinationSnafu { input })?;
                RouteTo::Ip(ip)
            }
        })
    }
}

impl Validate for StaticConfigV1 {
    fn validate(&self) -> ValidateResult<()> {
        ensure!(
            self.addresses.iter().all(|a| matches!(a, IpNet::V4(_)))
                || self.addresses.iter().all(|a| matches!(a, IpNet::V6(_))),
            InvalidNetConfigSnafu {
                reason: "static configuration must only contain all IPv4 or all IPv6 addresses"
            }
        );
        Ok(())
    }
}

mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(crate)))]
    pub(crate) enum Error {
        #[snafu(display("Invalid route destination, must be 'default' or a valid IP address prefix.  Received '{}': {}", input, source))]
        InvalidRouteDestination {
            input: String,
            source: ipnet::AddrParseError,
        },
    }
}

type Result<T> = std::result::Result<T, error::Error>;
