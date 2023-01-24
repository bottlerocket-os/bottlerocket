use super::{error, Result};
use crate::net_config;
use crate::net_config::Interfaces;
use crate::OVERRIDE_NET_CONFIG_FILE;
use argh::FromArgs;
use snafu::{OptionExt, ResultExt};
use std::path::Path;

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "validate-net-config")]
/// Check the network configuration
pub(crate) struct ValidateNetConfigArgs {
    #[argh(
        option,
        short = 'f',
        default = "String::from(OVERRIDE_NET_CONFIG_FILE)"
    )]
    /// network configuration file
    network_file: String,
}

/// Check that the configuration for network interfaces parses
pub(crate) fn run(args: ValidateNetConfigArgs) -> Result<()> {
    let network_file = Path::new(&args.network_file);
    // `maybe_net_config` could be `None` if no interfaces were defined
    let maybe_net_config: Option<Box<dyn Interfaces>> = if Path::exists(network_file.as_ref()) {
        net_config::from_path(network_file)
            .context(error::NetConfigParseSnafu { path: network_file })?
    } else {
        None
    };

    // `maybe_net_config` could be `None` if no interfaces were defined
    let net_config = match maybe_net_config {
        Some(net_config) => net_config,
        None => {
            eprintln!("No network interfaces were configured");
            return Ok(());
        }
    };

    // Find the primary interface from the config
    let primary_interface = net_config
        .primary_interface()
        .context(error::GetPrimaryInterfaceSnafu)?;
    println!("{} found as primary interface", primary_interface);

    // Print the interface names as feedback that they were found
    for interface in net_config.as_wicked_interfaces() {
        println!("Found {}", interface.name);
    }

    println!("net.toml file provided successfully parsed!");
    Ok(())
}
