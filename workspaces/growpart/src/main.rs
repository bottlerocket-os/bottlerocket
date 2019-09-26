/*!
# Introduction

growpart is a helper program to expand a partition to fill all available sectors on the
underlying block device.
*/

mod diskpart;
use diskpart::error::Result;
use diskpart::DiskPart;
use std::env;
use std::path::PathBuf;

/// Stores user-supplied arguments.
#[derive(Debug)]
struct Args {
    partition: PathBuf,
}

/// Informs the user about proper usage of the program and exits.
fn usage() -> ! {
    let program_name = env::args().next().unwrap_or_else(|| "program".to_string());
    eprintln!(r"Usage: {} PARTITION", program_name);
    std::process::exit(2);
}

/// Prints a more specific message before exiting through usage().
fn usage_msg<S: AsRef<str>>(msg: S) -> ! {
    eprintln!("{}\n", msg.as_ref());
    usage();
}

/// Parses user arguments into an Args structure.
fn parse_args() -> Result<Args> {
    let partition = env::args()
        .nth(1)
        .unwrap_or_else(|| usage_msg("Did not specify partition"));
    let partition = PathBuf::from(partition);
    Ok(Args { partition })
}

fn run() -> Result<()> {
    let args = parse_args()?;
    let mut diskpart = DiskPart::new(args.partition)?;
    diskpart.grow()?;
    diskpart.write()?;
    Ok(())
}

// Returning a Result from main makes it print a Debug representation of the error, but with Snafu
// we have nice Display representations of the error, so we wrap "main" (run) and print any error.
// https://github.com/shepmaster/snafu/issues/110
fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
