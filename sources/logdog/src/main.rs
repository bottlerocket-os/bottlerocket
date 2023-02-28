/*!
# Introduction

`logdog` is a program that gathers logs from various places on a Bottlerocket host and combines them
into a tarball for easy export.

Usage example:

```shell
$ logdog
logs are at: /var/log/support/bottlerocket-logs.tar.gz
```

# Logs

For the log requests used to gather logs, please see the following:

* [log_request](src/log_request.rs)
* [logdog.common.conf](conf/logdog.common.conf)
* And the variant-specific files in [conf](conf/), one of which is selected by [build.rs](build.rs)
based on the value of the `VARIANT` environment variable at build time.

*/

mod create_tarball;
mod error;
mod log_request;

use create_tarball::create_tarball;
use error::Result;
use log_request::{handle_log_request, log_requests};
use snafu::{ErrorCompat, ResultExt};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, process};
use tempfile::TempDir;

const ERROR_FILENAME: &str = "logdog.errors";
const OUTPUT_FILENAME: &str = "bottlerocket-logs.tar.gz";
const OUTPUT_DIRNAME: &str = "/var/log/support";
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
        None => PathBuf::from(OUTPUT_DIRNAME).join(OUTPUT_FILENAME),
    }
}

/// Runs a list of log requests and writes their output into files in `outdir`. Any failures are
/// noted in the file named by `ERROR_FILENAME`. Note: In the case of `exec` log requests, non-zero
/// exit codes are not considered errors and the command's stdout and stderr will be still be
/// written.
pub(crate) async fn collect_logs<P: AsRef<Path>>(log_requests: &[&str], outdir: P) -> Result<()> {
    // if a command fails, we will pipe its error here and continue.
    let outdir = outdir.as_ref();
    let error_path = outdir.join(crate::ERROR_FILENAME);
    let mut error_file = File::create(&error_path).context(error::ErrorFileSnafu {
        path: error_path.clone(),
    })?;

    for &log_request in log_requests {
        // show the user what command we are running
        println!("Running: {}", log_request);
        if let Err(e) = handle_log_request(log_request, &outdir).await {
            // ignore the error, but make note of it in the error file.
            writeln!(
                &mut error_file,
                "Error running command '{}': '{}'",
                log_request, e
            )
            .context(error::ErrorWriteSnafu {
                path: error_path.clone(),
            })?;
        }
    }
    Ok(())
}

/// Runs the bulk of the program's logic, main wraps this.
async fn run(outfile: &Path, commands: &[&str]) -> Result<()> {
    let temp_dir = TempDir::new().context(error::TempDirCreateSnafu)?;
    collect_logs(commands, &temp_dir.path().to_path_buf()).await?;
    create_tarball(temp_dir.path(), outfile)?;
    println!("logs are at: {}", outfile.display());
    Ok(())
}

#[tokio::main]
async fn main() -> ! {
    let outpath = parse_args(env::args());
    let log_requests = log_requests();
    process::exit(match run(&outpath, &log_requests).await {
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

    #[tokio::test]
    async fn test_program() {
        let output_tempdir = TempDir::new().unwrap();
        let outfile = output_tempdir.path().join("logstest");

        // we assume that `echo` will not do something unexpected on the machine running this test.
        let commands = vec!["exec hello.txt echo hello world"];
        run(&outfile, &commands).await.unwrap();

        // this function will panic if the given path is not found in the tarball.
        let find = |path_to_find: &PathBuf| {
            let tar_gz = File::open(&outfile).unwrap();
            let tar = GzDecoder::new(tar_gz);
            let mut archive = Archive::new(tar);
            let mut entries = archive.entries().unwrap();
            let _found = entries
                .find(|item| {
                    let entry = item.as_ref().unwrap();
                    let path = entry.path().unwrap();
                    path == *path_to_find
                })
                .unwrap()
                .unwrap();
        };

        // assert that the expected paths exist in the tarball
        find(&PathBuf::from(TARBALL_DIRNAME));
        find(&PathBuf::from(TARBALL_DIRNAME).join("hello.txt"));
    }
}
