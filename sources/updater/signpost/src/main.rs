#![warn(clippy::pedantic)]

use serde::Deserialize;
use signpost::State;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Command {
    Status,
    MarkSuccessfulBoot,
    MarkInactiveValid,
    ClearInactive,
    UpgradeToInactive,
    CancelUpgrade,
    RollbackToInactive,
    HasBootEverSucceeded,
    RewriteTable,
}

fn usage() -> ! {
    eprintln!("\
USAGE:
    signpost <SUBCOMMAND>

SUBCOMMANDS:
    status                  Show partition sets and priority status
    mark-successful-boot    Mark the active partitions as successfully booted
    clear-inactive          Clears inactive priority information to prepare writing images to disk
    mark-inactive-valid     Marks the inactive partition as having a valid image
    upgrade-to-inactive     Sets the inactive partitions as new upgrade partitions if marked valid
    cancel-upgrade          Reverse upgrade-to-inactive
    rollback-to-inactive    Deprioritizes the inactive partitions
    has-boot-ever-succeeded Checks whether boot has ever succeeded
    rewrite-table           Rewrite the partition table with no changes to disk (used for testing this code)");
    std::process::exit(1)
}

fn main() {
    let command_str = std::env::args().nth(1).unwrap_or_else(|| usage());
    let command = serde_plain::from_str::<Command>(&command_str).unwrap_or_else(|_| usage());

    if let Err(err) = State::load().and_then(|mut state| {
        match command {
            Command::Status => println!("{state}"),
            Command::ClearInactive => {
                state.clear_inactive();
                state.write()?;
            }
            Command::MarkSuccessfulBoot => {
                state.mark_successful_boot();
                state.write()?;
            }
            Command::MarkInactiveValid => {
                state.mark_inactive_valid();
                state.write()?;
            }
            Command::UpgradeToInactive => {
                state.upgrade_to_inactive()?;
                state.write()?;
            }
            Command::CancelUpgrade => {
                state.cancel_upgrade();
                state.write()?;
            }
            Command::RollbackToInactive => {
                state.rollback_to_inactive()?;
                state.write()?;
            }
            Command::HasBootEverSucceeded => {
                if state.has_boot_succeeded() {
                    println!("true");
                }
            }
            Command::RewriteTable => state.write()?,
        }
        Ok(())
    }) {
        eprintln!("{err}");
        std::process::exit(1)
    }
}
