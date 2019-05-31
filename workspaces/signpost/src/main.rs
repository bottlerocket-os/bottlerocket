#![warn(clippy::pedantic)]

mod error;
mod gptprio;
mod guid;
mod set;
mod state;

use crate::state::State;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Command {
    Status,
    MarkSuccessfulBoot,
    UpgradeToInactive,
    RollbackToInactive,
    RewriteTable,
}

fn usage() -> ! {
    eprintln!("\
USAGE:
    signpost <SUBCOMMAND>

SUBCOMMANDS:
    status                  Show partition sets and priority status
    mark-successful-boot    Mark the active partition as successfully booted
    upgrade-to-inactive     Sets the inactive partition as a new upgrade partition
    rollback-to-inactive    Deprioritizes the inactive partition
    rewrite-table           Rewrite the partition table with no changes to disk (used for testing this code)");
    std::process::exit(1)
}

fn main() {
    let command_str = std::env::args().nth(1).unwrap_or_else(|| usage());
    let command = serde_plain::from_str::<Command>(&command_str).unwrap_or_else(|_| usage());

    if let Err(err) = State::load().and_then(|mut state| {
        match command {
            Command::Status => println!("{}", state),
            Command::MarkSuccessfulBoot => {
                state.mark_successful_boot();
                state.write()?;
            }
            Command::UpgradeToInactive => {
                state.upgrade_to_inactive();
                state.write()?;
            }
            Command::RollbackToInactive => {
                state.rollback_to_inactive();
                state.write()?;
            }
            Command::RewriteTable => state.write()?,
        }
        Ok(())
    }) {
        eprintln!("{}", err);
        std::process::exit(1)
    }
}
