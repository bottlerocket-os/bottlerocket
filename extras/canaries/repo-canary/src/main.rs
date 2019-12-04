#![deny(rust_2018_idioms)]
#![warn(clippy::pedantic)]

#[macro_use]
extern crate log;

use rand::seq::SliceRandom;
use signal_hook::{iterator::Signals, SIGINT, SIGQUIT, SIGTERM};
use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};
use snafu::ResultExt;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::string::ToString;
use std::sync::atomic::{AtomicI32, Ordering};
use std::{env, io};
use std::{process, thread};
use tempfile::tempdir;
use tough::{error as tough_error, HttpTransport, Limits, Repository, Settings};

static SIGNAL: AtomicI32 = AtomicI32::new(0);

type HttpRepo<'a> = Repository<'a, HttpTransport>;

// Custom exit codes
const TRUSTED_ROOT_VALIDATION_FAILURE: i32 = 64;
const METADATA_VALIDATION_FAILURE: i32 = 65;
const VERSION_MISMATCH: i32 = 66;
const FETCH_FAILURE: i32 = 67;
const EXPIRED_METADATA: i32 = 68;
const MISSING_METADATA: i32 = 69;
const ROLLBACK_DETECTED: i32 = 70;
const MISSING_TARGET: i32 = 71;
const TARGET_DOWNLOAD_FAILURE: i32 = 72;
const OTHER_ERROR: i32 = 1;

mod error {
    use snafu::{Backtrace, Snafu};
    use std::path::PathBuf;

    /// Potential errors during pre-init process.
    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(crate) enum RepoCanaryError {
        #[snafu(display("Failed to open trusted root metadata file {}: {}", path.display(), source))]
        OpenRoot {
            path: PathBuf,
            source: std::io::Error,
            backtrace: Backtrace,
        },

        #[snafu(display("Failed to open file {} for writing: {}", path.display(), source))]
        OpenFile {
            path: PathBuf,
            source: std::io::Error,
            backtrace: Backtrace,
        },

        #[snafu(display("Failed to create tempdir for data store: {}", source))]
        CreateTempdir {
            source: std::io::Error,
            backtrace: Backtrace,
        },

        #[snafu(display("Failed to remove tempdir used for data store: {}", source))]
        CloseTempdir {
            source: std::io::Error,
            backtrace: Backtrace,
        },

        #[snafu(display("Failed to set up signal handler: {}", source))]
        Signal {
            source: std::io::Error,
            backtrace: Backtrace,
        },

        #[snafu(display("Logger setup error: {}", source))]
        Logger { source: log::SetLoggerError },
    }
}

type Result<T> = std::result::Result<T, error::RepoCanaryError>;

#[derive(Debug)]
struct Args {
    log_level: LevelFilter,
    metadata_base_url: String,
    target_base_url: String,
    trusted_root_path: PathBuf,
    percent_target_files: u8,
}

/// Print a usage message in the event a bad arg is passed
fn usage() -> ! {
    let program_name = env::args().next().unwrap_or_else(|| "program".to_string());
    eprintln!(
        r"Usage: {}
            --metadata-base-url URL
            --target-base-url URL
            --trusted-root-path PATH_TO_root.json
            [ --percentage-of-targets-to-retrieve 0-100 ] 'Randomly samples specified percentage of targets'
            [ --log-level trace|debug|info|warn|error ]

            If --percentage-of-targets-to-retrieve is not specified, {} will attempt to retrieve all targets listed in the TUF repository.
        ",
        program_name,
        program_name
    );
    process::exit(2);
}

/// Prints a more specific message before exiting through usage().
fn usage_msg<S: AsRef<str>>(msg: S) -> ! {
    eprintln!("{}\n", msg.as_ref());
    usage();
}

/// Parse the args to the program and return an Args struct
fn parse_args(args: env::Args) -> Args {
    let mut log_level = None;
    let mut metadata_base_url = None;
    let mut percent_target_files: Option<u8> = None;
    let mut target_base_url = None;
    let mut trusted_root_path = None;

    let mut iter = args.skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "--metadata-base-url" => {
                metadata_base_url =
                    Some(iter.next().unwrap_or_else(|| {
                        usage_msg("Did not give argument to --metadata_base_url")
                    }))
            }

            "--target-base-url" => {
                target_base_url = Some(
                    iter.next()
                        .unwrap_or_else(|| usage_msg("Did not give argument to --target_base_url")),
                )
            }

            "--trusted-root-path" => {
                trusted_root_path = Some(
                    iter.next()
                        .unwrap_or_else(|| {
                            usage_msg("Did not give argument to --trusted-root-path")
                        })
                        .into(),
                )
            }

            "--percentage-of-targets-to-retrieve" => {
                let percentage = iter
                    .next()
                    .unwrap_or_else(|| {
                        usage_msg("Did not give argument to --percentage-of-targets-to-retrieve")
                    })
                    .parse::<u8>()
                    .unwrap_or_else(|_| {
                        usage_msg("Invalid argument: expecting percentage from 0 to 100")
                    });
                if percentage > 100 {
                    usage_msg("Invalid argument: expecting percentage from 0 to 100")
                }
                percent_target_files = Some(percentage);
            }

            "--log-level" => {
                let log_level_str = iter
                    .next()
                    .unwrap_or_else(|| usage_msg("Did not give argument to --log-level"));
                log_level = Some(LevelFilter::from_str(&log_level_str).unwrap_or_else(|_| {
                    usage_msg(format!("Invalid log level '{}'", log_level_str))
                }));
            }

            _ => usage(),
        }
    }

    Args {
        log_level: log_level.unwrap_or_else(|| LevelFilter::Info),
        metadata_base_url: metadata_base_url.unwrap_or_else(|| usage()),
        percent_target_files: percent_target_files.unwrap_or_else(|| 100),
        target_base_url: target_base_url.unwrap_or_else(|| usage()),
        trusted_root_path: trusted_root_path.unwrap_or_else(|| usage()),
    }
}

/// Report errors through custom exit codes to be picked up by Cloudwatch event rules
// Potentially add other processing/reporting through rusoto?
fn match_report_tough_error(err: &tough::error::Error) -> i32 {
    eprintln!("Error: {}", err);
    match err {
        tough_error::Error::ExpiredMetadata { .. } => EXPIRED_METADATA,
        tough_error::Error::MetaMissing { .. } => MISSING_METADATA,
        tough_error::Error::OlderMetadata { .. } => ROLLBACK_DETECTED,
        tough_error::Error::VerifyTrustedMetadata { .. } => TRUSTED_ROOT_VALIDATION_FAILURE,
        tough_error::Error::VerifyMetadata { .. } => METADATA_VALIDATION_FAILURE,
        tough_error::Error::VersionMismatch { .. } => VERSION_MISMATCH,
        tough_error::Error::Transport { .. } => FETCH_FAILURE,
        _ => OTHER_ERROR,
    }
}

/// Randomly samples specified percentage of listed targets in the TUF repo and tries to retrieve them
fn retrieve_percentage_of_targets<P>(
    repo: &HttpRepo<'_>,
    datastore_path: P,
    percentage: u8,
) -> Result<i32>
where
    P: AsRef<Path>,
{
    let targets = repo.targets();
    let percentage = percentage as f32 / 100.0;
    let num_to_retrieve = (targets.len() as f32 * percentage).ceil();
    let mut rng = &mut rand::thread_rng();
    let mut sampled_targets: Vec<String> = targets.keys().map(|key| key.to_string()).collect();
    sampled_targets = sampled_targets
        .choose_multiple(&mut rng, num_to_retrieve as usize)
        .cloned()
        .collect();
    for target in sampled_targets {
        let recv_signal = SIGNAL.load(Ordering::SeqCst);
        if recv_signal != 0 {
            return Ok(recv_signal + 128);
        }
        let target_reader = repo.read_target(&target);
        match target_reader {
            Err(ref err) => return Ok(match_report_tough_error(err)),
            Ok(target_reader) => match target_reader {
                None => {
                    eprintln!("Missing target: {}", target);
                    return Ok(MISSING_TARGET);
                }
                Some(mut reader) => {
                    info!("Downloading target: {}", target);
                    let path = datastore_path.as_ref().join(target);
                    let mut f = OpenOptions::new()
                        .write(true)
                        .create(true)
                        .open(&path)
                        .context(error::OpenFile { path: &path })?;
                    if let Err(ref err) = io::copy(&mut reader, &mut f) {
                        eprintln!("Error: {}", err);
                        return Ok(TARGET_DOWNLOAD_FAILURE);
                    }
                }
            },
        };
    }
    Ok(0)
}

fn main() -> Result<()> {
    // Parse and store the args passed to the program
    let args = parse_args(env::args());

    SimpleLogger::init(args.log_level, LogConfig::default()).context(error::Logger)?;

    // Create the datastore path for storing the metadata files
    let datastore = tempdir().context(error::CreateTempdir)?;
    let signals = Signals::new(&[SIGINT, SIGTERM, SIGQUIT]).context(error::Signal)?;
    thread::spawn(move || {
        for sig in signals.forever() {
            // No, we're not supposed to print here, but by the time we check STOP in our main loop
            // it could be many seconds later and the user will have no indication that their input was received.
            SIGNAL.store(sig, Ordering::SeqCst);
            info!("Received termination signal, will exit after next operation");
        }
    });

    info!("Loading TUF repo");
    let transport = HttpTransport::new();
    let repo = Repository::load(
        &transport,
        Settings {
            root: File::open(&args.trusted_root_path).context(error::OpenRoot {
                path: &args.trusted_root_path,
            })?,
            datastore: datastore.path(),
            metadata_base_url: &args.metadata_base_url,
            target_base_url: &args.target_base_url,
            limits: Limits {
                max_root_size: 1024 * 1024,         // 1 MiB
                max_targets_size: 1024 * 1024 * 10, // 10 MiB
                max_timestamp_size: 1024 * 1024,    // 1 MiB
                max_root_updates: 1024,
            },
        },
    );
    // Check for errors from loading the TUF repository
    let rc = match &repo {
        Err(err) => match_report_tough_error(err),
        Ok(repo) => {
            info!("Loaded TUF repo");
            // Try retrieving listed targets
            info!(
                "Downloading {}% of listed targets",
                args.percent_target_files
            );
            retrieve_percentage_of_targets(repo, datastore.path(), args.percent_target_files)?
        }
    };

    // Close/delete tempdir
    datastore.close().context(error::CloseTempdir)?;

    process::exit(rc);
}
