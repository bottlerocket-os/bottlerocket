use super::{error, Result};
use crate::cli::check_net_config;
use crate::interface_id::InterfaceId;
use crate::{PRIMARY_INTERFACE, PRIMARY_MAC_ADDRESS};
use argh::FromArgs;
use snafu::{OptionExt, ResultExt};
use std::fs;
use std::path::Path;

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "generate-net-config")]
/// Generate wicked network configuration
pub(crate) struct GenerateNetConfigArgs {}

/// Generate configuration for network interfaces.
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

    let primary_interface = net_config
        .primary_interface()
        .context(error::GetPrimaryInterfaceSnafu)?;
    // Remove existing primary interface files since the current primary may have changed or may
    // now be using a MAC address (via an override net.toml/reboot)
    remove_old_primary_interface()?;
    write_primary_interface(&primary_interface)?;

    let wicked_interfaces = net_config.as_wicked_interfaces();
    for interface in wicked_interfaces {
        interface
            .write_config_file()
            .context(error::InterfaceConfigWriteSnafu)?;
    }
    Ok(())
}

/// Remove primary interface and mac address files
fn remove_old_primary_interface() -> Result<()> {
    for file in &[PRIMARY_INTERFACE, PRIMARY_MAC_ADDRESS] {
        if Path::exists(Path::new(file)) {
            fs::remove_file(file).context(error::FileRemoveSnafu { path: file })?;
        };
    }
    Ok(())
}

/// Persist the primary interface name or MAC to file
fn write_primary_interface(interface_id: &InterfaceId) -> Result<()> {
    match interface_id {
        InterfaceId::Name(name) => fs::write(PRIMARY_INTERFACE, name.to_string()),
        InterfaceId::MacAddress(mac) => fs::write(PRIMARY_MAC_ADDRESS, mac.to_string()),
    }
    .context(error::PrimaryInterfaceWriteSnafu {
        path: PRIMARY_INTERFACE,
    })
}
