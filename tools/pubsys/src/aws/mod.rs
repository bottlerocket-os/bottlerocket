use aws_sdk_ec2::model::ArchitectureValues;
use aws_sdk_ec2::Region;

#[macro_use]
pub(crate) mod client;

pub(crate) mod ami;
pub(crate) mod promote_ssm;
pub(crate) mod publish_ami;
pub(crate) mod ssm;
pub(crate) mod validate_ami;
pub(crate) mod validate_ssm;

/// Builds a Region from the given region name.
fn region_from_string(name: &str) -> Region {
    Region::new(name.to_owned())
}

/// Parses the given string as an architecture, mapping values to the ones used in EC2.
pub(crate) fn parse_arch(input: &str) -> Result<ArchitectureValues> {
    match input {
        "x86_64" | "amd64" => Ok(ArchitectureValues::X8664),
        "arm64" | "aarch64" => Ok(ArchitectureValues::Arm64),
        _ => error::ParseArchSnafu {
            input,
            msg: "unknown architecture",
        }
        .fail(),
    }
}

mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(crate) enum Error {
        #[snafu(display("Failed to parse arch '{}': {}", input, msg))]
        ParseArch { input: String, msg: String },
    }
}
type Result<T> = std::result::Result<T, error::Error>;
