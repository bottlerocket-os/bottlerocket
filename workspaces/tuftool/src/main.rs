#![warn(clippy::pedantic)]

mod copylike;
mod create;
mod deref;
mod error;
mod key;
mod root;
mod source;
mod ssm;

use crate::error::Result;
use snafu::{ErrorCompat, OptionExt, ResultExt};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use structopt::StructOpt;
use tempfile::NamedTempFile;

static SPEC_VERSION: &str = "1";

#[derive(Debug, StructOpt)]
enum Command {
    /// Create a TUF repository
    Create(create::CreateArgs),
    /// Manipulate a root.json metadata file
    Root(root::Command),
}

impl Command {
    fn run(&self) -> Result<()> {
        match self {
            Command::Create(args) => args.run(),
            Command::Root(root_subcommand) => root_subcommand.run(),
        }
    }
}

fn load_file<T>(path: &Path) -> Result<T>
where
    for<'de> T: serde::Deserialize<'de>,
{
    serde_json::from_reader(File::open(path).context(error::FileOpen { path })?)
        .context(error::FileParseJson { path })
}

fn write_file<T>(path: &Path, json: &T) -> Result<()>
where
    T: serde::Serialize,
{
    // Use `tempfile::NamedTempFile::persist` to perform an atomic file write.
    let parent = path.parent().context(error::PathParent { path })?;
    let mut writer =
        NamedTempFile::new_in(parent).context(error::FileTempCreate { path: parent })?;
    serde_json::to_writer_pretty(&mut writer, json).context(error::FileWriteJson { path })?;
    writer.write_all(b"\n").context(error::FileWrite { path })?;
    writer.persist(path).context(error::FilePersist { path })?;
    Ok(())
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
