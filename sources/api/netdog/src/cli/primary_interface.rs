use super::{primary_interface_name, print_json, Result};
use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "primary-interface")]
/// Return the current IP address
pub(crate) struct PrimaryInterfaceArgs {}

/// Return the current IP address as JSON (intended for use as a settings generator)
pub(crate) fn run() -> Result<()> {
    let primary_interface = primary_interface_name()?;
    print_json(primary_interface)
}
