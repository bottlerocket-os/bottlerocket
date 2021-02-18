/*!
# Background

static-pods ensures static pods are running as defined in settings.

It queries for all existing static pod settings, then configures the system as follows:
* If the pod is enabled, it creates the manifest file in the pod manifest path that kubelet is
  configured to read from and populates the file with the base64-decoded manifest setting value.
* If the pod is enabled and the manifest file already exists, it overwrites the existing manifest
  file with the base64-decoded manifest setting value.
* If the pod is disabled, it ensures the manifest file is removed from the pod manifest path.
*/

use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};
use snafu::{ensure, OptionExt, ResultExt};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::str::FromStr;

use model::modeled_types::Identifier;

// FIXME Get from configuration in the future
const DEFAULT_API_SOCKET: &str = "/run/api.sock";

const STATIC_POD_DIR: &str = "/etc/kubernetes/static-pods";

type Result<T> = std::result::Result<T, error::Error>;

/// Query the API for the currently defined static pods
async fn get_static_pods<P>(socket_path: P) -> Result<Option<HashMap<Identifier, model::StaticPod>>>
where
    P: AsRef<Path>,
{
    debug!("Requesting settings values");
    let settings = schnauzer::get_settings(socket_path)
        .await
        .context(error::RetrieveSettings)?
        .settings
        .context(error::MissingSettings)?;

    Ok(settings
        .kubernetes
        .context(error::MissingSettings)?
        .static_pods)
}

/// Write out the manifest file to the pod manifest path with a given filename
fn write_manifest_file<S1, S2>(name: S1, manifest: S2) -> Result<()>
where
    S1: AsRef<str>,
    S2: AsRef<[u8]>,
{
    let name = name.as_ref();

    let dir = Path::new(STATIC_POD_DIR);
    fs::create_dir_all(&dir).context(error::Mkdir { dir: &dir })?;
    let path = dir.join(name);
    // Create the file if it does not exist, completely replace its contents
    // if it does (constitutes an update).
    fs::write(path, manifest).context(error::ManifestWrite { name })?;

    Ok(())
}

/// Deletes the named manifest file if it exists
fn delete_manifest_file<S1>(name: S1) -> Result<()>
where
    S1: AsRef<str>,
{
    let name = name.as_ref();
    let path = Path::new(STATIC_POD_DIR).join(name);
    if path.exists() {
        fs::remove_file(path).context(error::ManifestDelete { name })?;
    }

    Ok(())
}

fn handle_static_pod<S>(name: S, pod_info: &model::StaticPod) -> Result<()>
where
    S: AsRef<str>,
{
    // Get basic settings, as retrieved from API.
    let name = name.as_ref();
    let enabled = pod_info.enabled.context(error::MissingField {
        name,
        field: "enabled",
    })?;

    if enabled {
        let manifest = pod_info.manifest.as_ref().context(error::MissingField {
            name,
            field: "manifest",
        })?;

        let manifest = base64::decode(manifest.as_bytes()).context(error::Base64Decode {
            base64_string: manifest.as_ref(),
        })?;

        info!("Writing static pod '{}' to '{}'", name, STATIC_POD_DIR);

        // Write the manifest file for this static pod
        write_manifest_file(name, manifest)?;
    } else {
        info!("Removing static pod '{}' from '{}'", name, STATIC_POD_DIR);

        // Delete the manifest file so the static pod no longer runs (disabled)
        delete_manifest_file(name)?;
    }

    Ok(())
}

async fn run() -> Result<()> {
    let args = parse_args(env::args())?;

    // SimpleLogger will send errors to stderr and anything less to stdout.
    SimpleLogger::init(args.log_level, LogConfig::default()).context(error::Logger)?;

    info!("static-pods started");

    let mut failed = 0u32;
    if let Some(static_pods) = get_static_pods(args.socket_path).await? {
        for (name, pod) in static_pods.iter() {
            // Continue to handle other static pods if we fail one
            if let Err(e) = handle_static_pod(name, pod) {
                failed += 1;
                error!("Failed to handle static pod '{}': {}", &name, e);
            }
        }

        ensure!(
            failed == 0,
            error::ManageStaticPodsFailed {
                failed,
                tried: static_pods.len()
            }
        );
    }

    Ok(())
}

/// Store the args we receive on the command line
struct Args {
    log_level: LevelFilter,
    socket_path: PathBuf,
}

/// Print a usage message in the event a bad arg is passed
fn usage() {
    let program_name = env::args().next().unwrap_or_else(|| "program".to_string());
    eprintln!(
        r"Usage: {}
            [ --socket-path PATH ]
            [ --log-level trace|debug|info|warn|error ]

    Socket path defaults to {}",
        program_name, DEFAULT_API_SOCKET,
    );
}

/// Parse the args to the program and return an Args struct
fn parse_args(args: env::Args) -> Result<Args> {
    let mut log_level = None;
    let mut socket_path = None;

    let mut iter = args.skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "--log-level" => {
                let log_level_str = iter.next().ok_or_else(|| error::Error::Usage {
                    message: "Did not give argument to --log-level".into(),
                })?;
                log_level = Some(LevelFilter::from_str(&log_level_str).map_err(|_| {
                    error::Error::Usage {
                        message: format!("Invalid log level '{}'", log_level_str),
                    }
                })?);
            }

            "--socket-path" => {
                socket_path = Some(
                    iter.next()
                        .ok_or_else(|| error::Error::Usage {
                            message: "Did not give argument to --socket-path".into(),
                        })?
                        .into(),
                )
            }

            _ => {
                return Err(error::Error::Usage {
                    message: "unexpected argument".into(),
                })
            }
        }
    }

    Ok(Args {
        log_level: log_level.unwrap_or_else(|| LevelFilter::Info),
        socket_path: socket_path.unwrap_or_else(|| DEFAULT_API_SOCKET.into()),
    })
}

// Returning a Result from main makes it print a Debug representation of the error, but with Snafu
// we have nice Display representations of the error, so we wrap "main" (run) and print any error.
// https://github.com/shepmaster/snafu/issues/110
pub(crate) async fn main() -> () {
    if let Err(e) = run().await {
        match e {
            error::Error::Usage { .. } => {
                eprintln!("{}", e);
                usage();
                process::exit(2);
            }
            _ => {
                eprintln!("{}", e);
                process::exit(1);
            }
        }
    }
}

mod error {
    use snafu::Snafu;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(super) enum Error {
        #[snafu(display("{}", message))]
        Usage { message: String },

        #[snafu(display("Failed to retrieve settings: {}", source))]
        RetrieveSettings { source: schnauzer::Error },

        #[snafu(display("settings.kubernetes.static_pods missing in API response"))]
        MissingSettings {},

        #[snafu(display("Static pod '{}' missing field '{}'", name, field))]
        MissingField { name: String, field: String },

        #[snafu(display("Failed to manage {} of {} static pods", failed, tried))]
        ManageStaticPodsFailed { failed: u32, tried: usize },

        #[snafu(display("Logger setup error: {}", source))]
        Logger { source: log::SetLoggerError },

        #[snafu(display("Unable to base64 decode manifest '{}': '{}'", base64_string, source))]
        Base64Decode {
            base64_string: String,
            source: base64::DecodeError,
        },

        #[snafu(display("Failed to create directory '{}': '{}'", dir.display(), source))]
        Mkdir {
            dir: PathBuf,
            source: std::io::Error,
        },

        #[snafu(display("Failed to write manifest for static pod '{}': {}", name, source))]
        ManifestWrite {
            name: String,
            source: std::io::Error,
        },

        #[snafu(display(
            "Failed to delete manifest file for static pod '{}': {}'",
            name,
            source
        ))]
        ManifestDelete {
            name: String,
            source: std::io::Error,
        },
    }
}
