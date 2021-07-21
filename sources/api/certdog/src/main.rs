/*!
  certdog is a tool to manage the trusted certificates store. It adds/removes
  certificates from the final certificates bundle based on the configurations
  in the API.
*/

#![deny(rust_2018_idioms)]

#[macro_use]
extern crate log;

use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};
use snafu::{OptionExt, ResultExt};
use std::collections::HashMap;
use std::env;
use std::fmt::Write;
use std::fs;
use std::io::BufReader;
use std::io::{BufRead, Seek};
use std::path::Path;
use std::process;
use std::str::FromStr;
use x509_parser;

use model::modeled_types::Identifier;

// FIXME Get from configuration in the future
const DEFAULT_API_SOCKET: &str = "/run/api.sock";
const API_SETTINGS_URI: &str = "/settings";
// Read from the source in `/usr/share/factory` not the copy in `/etc`
const DEFAULT_SOURCE_BUNDLE: &str = "/usr/share/factory/etc/pki/tls/certs/ca-bundle.crt";
// This file is first created with tmpfilesd configurations
const DEFAULT_TRUSTED_STORE: &str = "/etc/pki/tls/certs/ca-bundle.crt";

// PEM delimiters
const PEM_HEADER: &str = "-----BEGIN";
const PEM_FOOTER: &str = "-----END";
const PEM_SUFFIX: &str = "-----";

/// Stores user-supplied global arguments
#[derive(Debug)]
struct Args {
    log_level: LevelFilter,
    socket_path: String,
    trusted_store: String,
    source_bundle: String,
    comment: bool,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            log_level: LevelFilter::Info,
            socket_path: DEFAULT_API_SOCKET.to_string(),
            trusted_store: DEFAULT_TRUSTED_STORE.to_string(),
            source_bundle: DEFAULT_SOURCE_BUNDLE.to_string(),
            comment: false,
        }
    }
}

/// Print a usage message in the event a bad arg is passed
fn usage() {
    let program_name = env::args().next().unwrap_or_else(|| "program".to_string());
    eprintln!(
        r"Usage: {} [ ARGUMENTS... ]

    Global arguments:
        [ --socket-path PATH ]
        [ --source-bundle PATH]
        [ --trusted-store PATH]
        [ --comment]
        [ --log-level trace|debug|info|warn|error ]

    Socket path defaults to {}
    Source bundle defaults to {}
    Trusted store defaults to {}
    ",
        program_name, DEFAULT_API_SOCKET, DEFAULT_SOURCE_BUNDLE, DEFAULT_TRUSTED_STORE
    );
}

/// Parses user arguments into an Args struct
fn parse_args(args: env::Args) -> Result<Args> {
    let mut final_args = Args::default();

    let mut iter = args.skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "--log-level" => {
                let log_level = iter.next().context(error::Usage {
                    message: "Did not give argument to --log-level",
                })?;
                final_args.log_level =
                    LevelFilter::from_str(&log_level).context(error::LogLevel { log_level })?;
            }

            "-s" | "--socket-path" => {
                final_args.socket_path = iter.next().context(error::Usage {
                    message: "Did not give argument to --socket-path",
                })?
            }

            "-t" | "--trusted-store" => {
                final_args.trusted_store = iter.next().context(error::Usage {
                    message: "Did not give argument to --trusted-store",
                })?
            }

            "-b" | "--source-bundle" => {
                final_args.source_bundle = iter.next().context(error::Usage {
                    message: "Did not give argument to --source-bundle",
                })?
            }

            "-c" | "--comment" => final_args.comment = true,

            x => {
                return error::Usage {
                    message: format!("Unknown option '{}'", x),
                }
                .fail()
            }
        }
    }

    Ok(final_args)
}

/// Query the API for the certificate bundles, returns a tuple with trusted
/// and distrusted PEM certificates
async fn get_certificate_bundles<P>(
    socket_path: P,
) -> Result<(Vec<x509_parser::pem::Pem>, Vec<x509_parser::pem::Pem>)>
where
    P: AsRef<Path>,
{
    debug!("Querying the API for settings");

    let method = "GET";
    let uri = API_SETTINGS_URI;
    let (_code, response_body) = apiclient::raw_request(&socket_path, uri, method, None)
        .await
        .context(error::APIRequest { method, uri })?;

    // Build a Settings struct from the response string
    debug!("Deserializing response");
    let settings: model::Settings =
        serde_json::from_str(&response_body).context(error::ResponseJson { method, uri })?;

    split_bundles(settings.pki.unwrap_or_default())
}

/// Returns a tuple with two lists, for trusted and distrusted certificates
fn split_bundles(
    certificates_bundle: HashMap<Identifier, model::PEMCertificate>,
) -> Result<(Vec<x509_parser::pem::Pem>, Vec<x509_parser::pem::Pem>)> {
    let mut trusted: Vec<x509_parser::pem::Pem> = Vec::new();
    let mut distrusted: Vec<x509_parser::pem::Pem> = Vec::new();

    for (name, bundle) in certificates_bundle.iter() {
        let data = bundle.data.clone().unwrap_or_default();

        // Empty data means the certificate bundle was disabled in the API
        if data.trim() == "" {
            debug!("Found empty bundle: {}", name);
            continue;
        }

        let name = name.as_ref();
        let decoded = base64::decode(data.as_bytes()).context(error::Base64Decode { name })?;
        // Each record in the API could include one or more certificates
        let mut pems = pems_from_iter(x509_parser::pem::Pem::iter_from_buffer(&decoded))?;

        // `trusted` defaults to false if not set in the API record
        if bundle.trusted.unwrap_or(false) {
            trusted.append(&mut pems);
        } else {
            distrusted.append(&mut pems);
        }
    }

    Ok((trusted, distrusted))
}

/// Updates the trusted certificates store, removing the distrusted certificates
/// from the final bundle
fn update_trusted_store<P>(
    (mut trusted, distrusted): (Vec<x509_parser::pem::Pem>, Vec<x509_parser::pem::Pem>),
    trusted_store: P,
    source_bundle: P,
    add_comment: bool,
) -> Result<()>
where
    P: AsRef<Path>,
{
    let source_bundle = source_bundle.as_ref();
    let trusted_store = trusted_store.as_ref();

    // The default bundle includes the certificates shipped with the OS
    let default_bundle = fs::File::open(source_bundle).context(error::ReadFile {
        path: source_bundle,
    })?;
    let reader = BufReader::new(default_bundle);
    // Initialize trusted bundle with the certificates shipped with the OS
    let mut trusted_bundle = pems_from_iter(x509_parser::pem::Pem::iter_from_reader(reader))?;

    // Add additional trusted certificates
    trusted_bundle.append(&mut trusted);
    // Remove any distrusted certificate
    trusted_bundle.retain(|pem| !distrusted.contains(pem));

    // Write a PEM formatted bundle from trusted certificates
    fs::write(
        &trusted_store,
        pems_to_string(&trusted_bundle, add_comment)?,
    )
    .context(error::UpdateTrustStore)?;

    Ok(())
}

/// Returns a list with Pem objects from a PemIterator
fn pems_from_iter<R>(iter: x509_parser::pem::PemIterator<R>) -> Result<Vec<x509_parser::pem::Pem>>
where
    R: BufRead + Seek,
{
    let mut vec: Vec<x509_parser::pem::Pem> = Vec::new();
    for pem in iter {
        let pem = pem.context(error::ParsePEM)?;
        vec.push(pem);
    }
    Ok(vec)
}

/// Concatenates all the PEM objects as a single PEM bundle
fn pems_to_string(pems: &Vec<x509_parser::pem::Pem>, add_comment: bool) -> Result<String> {
    let mut out = String::new();

    for pem in pems {
        writeln!(out, "{}", pem_to_string(pem, add_comment)?).context(error::WritePEMString)?;
    }

    Ok(out)
}

/// Transforms a PEM object into a PEM formatted string
fn pem_to_string(pem: &x509_parser::pem::Pem, add_comment: bool) -> Result<String> {
    let mut out = String::new();

    // If provided, a comment will be added before the PEM formatted string to
    // identify the certificate. This is useful for debugging purposes
    if add_comment {
        if let Some(comment) = comment_for_pem(&pem)? {
            writeln!(out, "# {}", comment).context(error::WritePEMString)?;
        }
    }

    writeln!(out, "{} {}{}", PEM_HEADER, pem.label, PEM_SUFFIX).context(error::WritePEMString)?;
    let encoded = base64::encode(&pem.contents);
    let bytes = encoded.as_bytes();
    for chunk in bytes.chunks(64) {
        let chunk = String::from_utf8_lossy(chunk);
        writeln!(out, "{}", chunk).context(error::WritePEMString)?;
    }
    writeln!(out, "{} {}{}", PEM_FOOTER, pem.label, PEM_SUFFIX).context(error::WritePEMString)?;

    Ok(out)
}

/// Returns a string from the common name, organizational unit or organization
/// fields in the certificate
fn comment_for_pem(pem: &x509_parser::pem::Pem) -> Result<Option<String>> {
    let cert = pem.parse_x509().context(error::ParseX509Certificate)?;
    let subject = cert.tbs_certificate.subject;

    if let Some(common_name) = subject.iter_common_name().next() {
        if let Ok(common_name_str) = common_name.as_str() {
            return Ok(Some(common_name_str.to_string()));
        }
    } else if let Some(organizational_unit) = subject.iter_organizational_unit().next() {
        if let Ok(organizational_unit_str) = organizational_unit.as_str() {
            return Ok(Some(organizational_unit_str.to_string()));
        }
    } else if let Some(organization) = subject.iter_organization().next() {
        if let Ok(organization_str) = organization.as_str() {
            return Ok(Some(organization_str.to_string()));
        }
    }

    Ok(None)
}

async fn run() -> Result<()> {
    let args = parse_args(env::args())?;

    // SimpleLogger will send errors to stderr and anything less to stdout.
    SimpleLogger::init(args.log_level, LogConfig::default()).context(error::Logger)?;

    info!("certdog started");
    let certificate_bundles = get_certificate_bundles(&args.socket_path).await?;
    info!("Got certificate bundles from API");
    update_trusted_store(
        certificate_bundles,
        args.trusted_store,
        args.source_bundle,
        args.comment,
    )?;
    info!("Updated trusted store");

    Ok(())
}

// Returning a Result from main makes it print a Debug representation of the error, but with Snafu
// we have nice Display representations of the error, so we wrap "main" (run) and print any error.
// https://github.com/shepmaster/snafu/issues/110
#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        match e {
            error::Error::Usage { .. } => {
                eprintln!("{}", e);
                usage();
                process::exit(1);
            }
            _ => {
                eprintln!("{}", e);
                process::exit(1);
            }
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=
mod error {
    use snafu::Snafu;
    use std::io;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(super) enum Error {
        #[snafu(display("Error sending {} to {}: {}", method, uri, source))]
        APIRequest {
            method: String,
            uri: String,
            source: apiclient::Error,
        },

        #[snafu(display("Unable to decode base64 from certificate '{}': '{}'", name, source))]
        Base64Decode {
            name: String,
            source: base64::DecodeError,
        },

        #[snafu(display("Logger setup error: {}", source))]
        Logger { source: log::SetLoggerError },

        #[snafu(display("Invalid log level '{}'", log_level))]
        LogLevel {
            log_level: String,
            source: log::ParseLevelError,
        },

        #[snafu(display("Failed to parse PEM: {}", source))]
        ParsePEM {
            source: x509_parser::error::PEMError,
        },

        #[snafu(display("Failed to parse cert: {}", source))]
        ParseX509Certificate {
            source: x509_parser::nom::Err<x509_parser::error::X509Error>,
        },

        #[snafu(display("Error while reading file {}: '{}'", path.display(), source))]
        ReadFile { path: PathBuf, source: io::Error },

        #[snafu(display(
            "Error deserializing response as JSON from {} to {}: {}",
            method,
            uri,
            source
        ))]
        ResponseJson {
            method: &'static str,
            uri: String,
            source: serde_json::Error,
        },

        #[snafu(display("{}", message))]
        Usage { message: String },

        #[snafu(display("Failed to update trust store: {}", source))]
        UpdateTrustStore { source: io::Error },

        #[snafu(display("failed to write to pem string: {}", source))]
        WritePEMString { source: std::fmt::Error },
    }
}

type Result<T> = std::result::Result<T, error::Error>;
