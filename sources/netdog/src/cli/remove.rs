use super::{InterfaceFamily, InterfaceType, Result};
use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "remove")]
// `wicked` calls `remove` with the below args and failing to parse them can cause an error in
// `wicked`.
/// Does nothing
pub(crate) struct RemoveArgs {
    #[argh(option, short = 'i')]
    /// name of the network interface
    interface_name: String,

    #[argh(option, short = 't')]
    /// network interface type
    interface_type: InterfaceType,

    #[argh(option, short = 'f')]
    /// network interface family (ipv4/6)
    interface_family: InterfaceFamily,
}

pub(crate) fn run(_: RemoveArgs) -> Result<()> {
    eprintln!("The 'remove' command is not implemented.");
    Ok(())
}
