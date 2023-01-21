use super::{check_net_config, error, Result};
use argh::FromArgs;
use snafu::OptionExt;

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "check-net-config")]
/// Check the network configuration
pub(crate) struct CheckNetConfigArgs {}

/// Check that the configuration for network interfaces parses
pub(crate) fn run() -> Result<()> {
    // `maybe_net_config` could be `None` if no interfaces were defined
    let net_config = match check_net_config() {
        Ok(Some(net_config)) => net_config,
        Ok(None) => {
            eprintln!("No network interfaces were configured");
            return Ok(());
        }
        Err(e) => {
            eprintln!("{}", e);
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
