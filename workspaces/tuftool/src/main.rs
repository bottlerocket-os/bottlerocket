#![warn(clippy::pedantic)]

mod copylike;
mod create;
mod deref;
mod error;
mod key;
mod source;
mod ssm;

use crate::error::Result;
use snafu::ErrorCompat;
use structopt::StructOpt;

static SPEC_VERSION: &str = "1";

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
enum Command {
    /// Create a TUF repository
    Create(create::CreateArgs),
}

impl Command {
    fn run(&self) -> Result<()> {
        match self {
            Command::Create(args) => args.run(),
        }
    }
}

fn main() -> ! {
    std::process::exit(match Command::from_args().run() {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("{}", err);
            if let Some(var) = std::env::var_os("RUST_BACKTRACE") {
                if var != "0" {
                    if let Some(backtrace) = err.backtrace() {
                        eprintln!("\n{:?}", backtrace.as_ref());
                    }
                }
            }
            1
        }
    })
}
