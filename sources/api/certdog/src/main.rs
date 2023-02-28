/*!
  certdog is a tool to manage the trusted certificates store. It adds and removes
  certificates from the final certificates bundle based on the configurations
  in the API.
*/

#[macro_use]
extern crate log;

use argh::FromArgs;
use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};
use snafu::ResultExt;
use std::collections::HashMap;
use std::fmt::Write;
use std::fs;
use std::io::BufReader;
use std::io::{BufRead, Seek};
use std::path::Path;
use std::process;

use model::modeled_types::Identifier;

// Read from the source in `/usr/share/factory` not the copy in `/etc`
const DEFAULT_SOURCE_BUNDLE: &str = "/usr/share/factory/etc/pki/tls/certs/ca-bundle.crt";
// This file is first created with tmpfilesd configurations
const DEFAULT_TRUSTED_STORE: &str = "/etc/pki/tls/certs/ca-bundle.crt";

// PEM delimiters
const PEM_HEADER: &str = "-----BEGIN";
const PEM_FOOTER: &str = "-----END";
const PEM_SUFFIX: &str = "-----";

/// Stores user-supplied global arguments
#[derive(FromArgs, Debug)]
struct Args {
    #[argh(option, default = "LevelFilter::Info", short = 'l')]
    /// log-level trace|debug|info|warn|error
    log_level: LevelFilter,
    #[argh(option, default = "constants::API_SOCKET.to_string()", short = 's')]
    /// socket-path path to apiserver socket
    socket_path: String,
    #[argh(option, default = "DEFAULT_TRUSTED_STORE.to_string()", short = 't')]
    /// trusted-store path to the trusted store
    trusted_store: String,
    #[argh(option, default = "DEFAULT_SOURCE_BUNDLE.to_string()", short = 'b')]
    /// source-bundle path to source bundle
    source_bundle: String,
}

struct CertBundle {
    trusted_certs: Vec<x509_parser::pem::Pem>,
    distrusted_certs: Vec<x509_parser::pem::Pem>,
}

/// Query the API for the certificate bundles, returns a tuple with trusted
/// and distrusted PEM certificates
async fn get_certificate_bundles<P>(socket_path: P) -> Result<CertBundle>
where
    P: AsRef<Path>,
{
    debug!("Querying the API for settings");

    let method = "GET";
    let uri = constants::API_SETTINGS_URI;
    let (_code, response_body) = apiclient::raw_request(&socket_path, uri, method, None)
        .await
        .context(error::APIRequestSnafu { method, uri })?;

    // Build a Settings struct from the response string
    debug!("Deserializing response");
    let settings: model::Settings =
        serde_json::from_str(&response_body).context(error::ResponseJsonSnafu { uri })?;

    split_bundles(settings.pki.unwrap_or_default())
}

/// Returns a tuple with two lists, for trusted and distrusted certificates
fn split_bundles(
    certificates_bundle: HashMap<Identifier, model::PemCertificate>,
) -> Result<CertBundle> {
    let mut trusted_certs: Vec<x509_parser::pem::Pem> = Vec::new();
    let mut distrusted_certs: Vec<x509_parser::pem::Pem> = Vec::new();

    for (name, bundle) in certificates_bundle.iter() {
        let data = bundle.data.clone().unwrap_or_default();

        // Empty data means the certificate bundle was disabled in the API
        if data.trim() == "" {
            debug!("Found empty bundle: {}", name);
            continue;
        }

        let name = name.as_ref();
        let decoded = base64::decode(data.as_bytes()).context(error::Base64DecodeSnafu { name })?;
        // Each record in the API could include one or more certificates
        let mut pems = pems_from_iter(x509_parser::pem::Pem::iter_from_buffer(&decoded))?;

        // `trusted` defaults to false if not set in the API record
        if bundle.trusted.unwrap_or(false) {
            trusted_certs.append(&mut pems);
        } else {
            distrusted_certs.append(&mut pems);
        }
    }

    Ok(CertBundle {
        trusted_certs,
        distrusted_certs,
    })
}

/// Updates the trusted certificates store, removing the distrusted certificates
/// from the final bundle
fn update_trusted_store<P>(
    mut cert_bundle: CertBundle,
    trusted_store: P,
    source_bundle: P,
) -> Result<()>
where
    P: AsRef<Path>,
{
    let source_bundle = source_bundle.as_ref();
    let trusted_store = trusted_store.as_ref();

    // The default bundle includes the certificates shipped with the OS
    let default_bundle = fs::File::open(source_bundle).context(error::ReadSourceBundleSnafu {
        path: source_bundle,
    })?;
    let reader = BufReader::new(default_bundle);
    // Initialize trusted bundle with the certificates shipped with the OS
    let mut trusted_bundle = pems_from_iter(x509_parser::pem::Pem::iter_from_reader(reader))?;

    // Add additional trusted certificates
    trusted_bundle.append(&mut cert_bundle.trusted_certs);
    // Remove any distrusted certificate
    trusted_bundle.retain(|pem| !cert_bundle.distrusted_certs.contains(pem));

    // Write a PEM formatted bundle from trusted certificates
    fs::write(trusted_store, pems_to_string(&trusted_bundle)?)
        .context(error::UpdateTrustedStoreSnafu)?;

    Ok(())
}

/// Returns a list with Pem objects from a PemIterator
fn pems_from_iter<R>(iter: x509_parser::pem::PemIterator<R>) -> Result<Vec<x509_parser::pem::Pem>>
where
    R: BufRead + Seek,
{
    let mut vec: Vec<x509_parser::pem::Pem> = Vec::new();
    for pem in iter {
        let pem = pem.context(error::ParsePEMSnafu)?;
        vec.push(pem);
    }
    Ok(vec)
}

/// Concatenates all the PEM objects as a single PEM bundle
fn pems_to_string(pems: &Vec<x509_parser::pem::Pem>) -> Result<String> {
    let mut out = String::new();

    for pem in pems {
        writeln!(out, "{}", pem_to_string(pem)?).context(error::WritePemStringSnafu)?;
    }

    Ok(out)
}

/// Transforms a PEM object into a PEM formatted string
fn pem_to_string(pem: &x509_parser::pem::Pem) -> Result<String> {
    let mut out = String::new();

    // A comment will be added before the PEM formatted string to identify the certificate.
    if let Some(comment) = comment_for_pem(pem)? {
        writeln!(out, "# {}", comment).context(error::WritePemStringSnafu)?;
    }

    writeln!(out, "{} {}{}", PEM_HEADER, pem.label, PEM_SUFFIX)
        .context(error::WritePemStringSnafu)?;
    let encoded = base64::encode(&pem.contents);
    let bytes = encoded.as_bytes();
    for chunk in bytes.chunks(64) {
        let chunk = String::from_utf8_lossy(chunk);
        writeln!(out, "{}", chunk).context(error::WritePemStringSnafu)?;
    }
    writeln!(out, "{} {}{}", PEM_FOOTER, pem.label, PEM_SUFFIX)
        .context(error::WritePemStringSnafu)?;

    Ok(out)
}

/// Returns a string from the common name, organizational unit or organization
/// fields in the certificate
fn comment_for_pem(pem: &x509_parser::pem::Pem) -> Result<Option<String>> {
    let cert = pem.parse_x509().context(error::ParseX509CertificateSnafu)?;
    let subject = cert.tbs_certificate.subject;
    let comment = subject
        .iter_common_name()
        .chain(subject.iter_organizational_unit())
        .chain(subject.iter_organization())
        .next();

    Ok(comment.and_then(|c| c.as_str().ok()).map(|c| c.to_string()))
}

async fn run() -> Result<()> {
    let args: Args = argh::from_env();

    // SimpleLogger will send errors to stderr and anything less to stdout.
    SimpleLogger::init(args.log_level, LogConfig::default()).context(error::LoggerSnafu)?;

    info!("certdog started");
    let certificate_bundles = get_certificate_bundles(&args.socket_path).await?;
    info!("Got certificate bundles from API");
    update_trusted_store(certificate_bundles, args.trusted_store, args.source_bundle)?;
    info!("Updated trusted store");

    Ok(())
}

// Returning a Result from main makes it print a Debug representation of the error, but with Snafu
// we have nice Display representations of the error, so we wrap "main" (run) and print any error.
// https://github.com/shepmaster/snafu/issues/110
#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        error!("{}", e);
        process::exit(1);
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=
mod error {
    use snafu::Snafu;
    use std::io;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(super) enum Error {
        #[snafu(display("Error sending {} to {}: {}", method, uri, source))]
        APIRequest {
            method: String,
            uri: String,
            #[snafu(source(from(apiclient::Error, Box::new)))]
            source: Box<apiclient::Error>,
        },

        #[snafu(display("Unable to decode base64 from certificate '{}': {}", name, source))]
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

        #[snafu(display("Error while reading bundle from file '{}': {}", path.display(), source))]
        ReadSourceBundle { path: PathBuf, source: io::Error },

        #[snafu(display("Error deserializing response from '{}': {}", uri, source))]
        ResponseJson {
            uri: String,
            source: serde_json::Error,
        },

        #[snafu(display("Failed to update trust store: {}", source))]
        UpdateTrustedStore { source: io::Error },

        #[snafu(display("Failed to write to pem string: {}", source))]
        WritePemString { source: std::fmt::Error },
    }
}

type Result<T> = std::result::Result<T, error::Error>;

#[cfg(test)]
mod test_certdog {
    use super::*;
    use model::modeled_types::{Identifier, PemCertificateString};
    use std::collections::HashMap;
    use std::convert::TryFrom;
    use std::fs::File;

    static TEST_PEM: &str = include_str!("../../../models/tests/data/test-pem");

    #[test]
    fn bundles_splitted() {
        let mut bundle = HashMap::new();
        bundle.insert(
            Identifier::try_from("trusted").unwrap(),
            model::PemCertificate {
                data: Some(PemCertificateString::try_from(TEST_PEM).unwrap()),
                trusted: Some(true),
            },
        );
        bundle.insert(
            Identifier::try_from("distrusted").unwrap(),
            model::PemCertificate {
                data: Some(PemCertificateString::try_from(TEST_PEM).unwrap()),
                trusted: Some(false),
            },
        );
        bundle.insert(
            Identifier::try_from("distrusted-without-flag").unwrap(),
            model::PemCertificate {
                data: Some(PemCertificateString::try_from(TEST_PEM).unwrap()),
                trusted: None,
            },
        );

        let splitted = split_bundles(bundle).unwrap();
        // The test-pem file contains two X509 certificates
        assert!(splitted.trusted_certs.len() == 2);
        assert!(splitted.distrusted_certs.len() == 4);
    }

    #[test]
    fn trusted_store_updated() {
        let trusted_store = tempfile::NamedTempFile::new().unwrap();
        let source_bundle = tempfile::NamedTempFile::new().unwrap();
        let (_, pem) =
            x509_parser::pem::parse_x509_pem(&base64::decode(TEST_PEM.as_bytes()).unwrap())
                .unwrap();
        let trusted_certs: Vec<x509_parser::pem::Pem> = vec![pem];
        let certs_bundle = CertBundle {
            trusted_certs,
            distrusted_certs: Vec::new(),
        };
        assert!(update_trusted_store(certs_bundle, &trusted_store, &source_bundle).is_ok());
        assert!(File::open(trusted_store).unwrap().metadata().unwrap().len() != 0);
    }
}
