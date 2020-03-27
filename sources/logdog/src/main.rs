/*!
# Introduction

`logdog` is a program that gathers logs from various places on a Bottlerocket host and combines them
into a tarball for easy export.

Usage example:
```
$ logdog
logs are at: /tmp/bottlerocket-logs.tar.gz
```

# Logs

For the commands used to gather logs, please see [log_request](src/log_request.rs).

*/

#![deny(rust_2018_idioms)]

mod create_tarball;
mod error;
mod log_request;

use std::collections::VecDeque;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{env, process};

use create_tarball::create_tarball;
use error::Result;
use log_request::{log_requests, LogRequest};
use snafu::{ErrorCompat, OptionExt, ResultExt};
use tempfile::TempDir;

const ERROR_FILENAME: &str = "logdog.errors";
const OUTPUT_FILENAME: &str = "bottlerocket-logs.tar.gz";
const TARBALL_DIRNAME: &str = "bottlerocket-logs";

/// Prints a usage message in the event a bad arg is passed.
fn usage() -> ! {
    let program_name = env::args().next().unwrap_or_else(|| "program".to_string());
    eprintln!(
        r"Usage: {}
            [ --output PATH ]       where to write archived logs
",
        program_name,
    );
    process::exit(2);
}

/// Prints a more specific message before exiting through usage().
fn usage_msg(msg: &str) -> ! {
    eprintln!("{}\n", msg);
    usage();
}

/// Parses the command line arguments.
fn parse_args(args: env::Args) -> PathBuf {
    let mut output_arg = None;
    let mut iter = args.skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "--output" => {
                output_arg = Some(
                    iter.next()
                        .unwrap_or_else(|| usage_msg("Did not give argument to --output")),
                )
            }
            _ => usage(),
        }
    }

    match output_arg {
        Some(path) => PathBuf::from(path),
        None => env::temp_dir().as_path().join(OUTPUT_FILENAME),
    }
}

/// Runs a command and writes its output to a file.
pub(crate) fn run_command<P: AsRef<Path>>(output_filepath: P, command: &str) -> Result<()> {
    let output_filepath = output_filepath.as_ref();
    let mut args: VecDeque<String> = shell_words::split(command)
        .context(error::CommandParse { command })?
        .into();
    let command = args.pop_front().context(error::EmptyCommand)?;
    let ofile = File::create(output_filepath).context(error::CommandOutputFile {
        path: output_filepath,
    })?;
    let stderr_file = ofile.try_clone().context(error::CommandErrFile {
        path: output_filepath,
    })?;
    Command::new(command.as_str())
        .args(&args)
        .stdout(Stdio::from(ofile))
        .stderr(Stdio::from(stderr_file))
        .spawn()
        .context(error::CommandSpawn {
            command: command.clone(),
        })?
        .wait_with_output()
        .context(error::CommandFinish {
            command: command.clone(),
        })?;
    Ok(())
}

/// Runs a list of commands and writes all of their output into files in the same `outdir`.  Any
/// failures are noted in the file named by ERROR_FILENAME.  This function ignores the commands'
/// return status and only fails if we can't save our own errors.
pub(crate) fn collect_logs<P: AsRef<Path>>(
    log_requests: impl Iterator<Item = LogRequest<'static>>,
    outdir: P,
) -> Result<()> {
    // if a command fails, we will pipe its error here and continue.
    let outdir = outdir.as_ref();
    let error_path = outdir.join(crate::ERROR_FILENAME);
    let mut error_file = File::create(&error_path).context(error::ErrorFile {
        path: error_path.clone(),
    })?;

    for log_request in log_requests {
        // show the user what command we are running
        println!("Running: {}", log_request.command);
        if let Err(e) = run_command(outdir.join(&log_request.filename), &log_request.command) {
            // ignore the error, but make note of it in the error file.
            write!(
                &mut error_file,
                "Error running command '{}': '{}'\n",
                log_request.command, e
            )
            .context(error::ErrorWrite {
                path: error_path.clone(),
            })?;
        }
    }
    Ok(())
}

/// Runs the bulk of the program's logic, main wraps this.
fn run(log_requests: impl Iterator<Item = LogRequest<'static>>, output: &PathBuf) -> Result<()> {
    let temp_dir = TempDir::new().context(error::TempDirCreate)?;
    collect_logs(log_requests, &temp_dir.path().to_path_buf())?;
    create_tarball(&temp_dir.path().to_path_buf(), &output)?;
    println!("logs are at: {}", output.display());
    Ok(())
}

fn main() -> ! {
    let output = parse_args(env::args());
    process::exit(match run(log_requests(), &output) {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("{}", err);
            if let Some(var) = env::var_os("RUST_BACKTRACE") {
                if var != "0" {
                    if let Some(backtrace) = err.backtrace() {
                        eprintln!("\n{:?}", backtrace);
                    }
                }
            }
            1
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::read::GzDecoder;
    use std::fs::File;
    use tar::Archive;

    #[test]
    fn test_program() {
        let output_tempdir = TempDir::new().unwrap();
        let output_filepath = output_tempdir.path().join("logstest");

        // we assume that `echo` will not do something unexpected on the machine running this test.
        run(
            [("hello.txt", "echo hello")]
                .iter()
                .map(|&item| LogRequest {
                    filename: item.0,
                    command: item.1,
                }),
            &output_filepath,
        )
        .unwrap();

        // open the file and check that its contents are as expected.

        // this function will panic if the path is not found in the tarball.
        let find = |path_to_find: &PathBuf| {
            let tar_gz = File::open(&output_filepath).unwrap();
            let tar = GzDecoder::new(tar_gz);
            let mut archive = Archive::new(tar);
            let mut entries = archive.entries().unwrap();
            let _found = entries
                .find(|item| {
                    let entry = item.as_ref().clone().unwrap();
                    let path = entry.path().unwrap();
                    PathBuf::from(path) == PathBuf::from(path_to_find)
                })
                .unwrap()
                .unwrap();
        };

        // these assert that the provided paths exist in the tarball
        find(&PathBuf::from(TARBALL_DIRNAME));
        find(&PathBuf::from(TARBALL_DIRNAME).join("hello.txt"));
    }
}
