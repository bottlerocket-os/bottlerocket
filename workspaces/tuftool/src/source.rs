//! Private keys are generally provided as paths, but may sometimes be provided as a URL. For
//! example, when one of the Rusoto features is enabled, you can use an aws-ssm:// URL to refer to
//! a key accessible in SSM.
//!
//! This module parses a key source command line parameter as a URL, relative to `file://$PWD`,
//! then matches the URL scheme against ones we understand.

use crate::error::{self, Error, Result};
use crate::key::KeyPair;
use snafu::{OptionExt, ResultExt};
use std::path::PathBuf;
use std::str::FromStr;
use url::Url;

// Silence "variant is never constructed: `Ssm`"; there appears to be a bug in this linter?
#[allow(dead_code)]
#[derive(Debug)]
pub(crate) enum KeySource {
    Local(PathBuf),
    #[cfg(any(feature = "rusoto-native-tls", feature = "rusoto-rustls"))]
    Ssm {
        profile: Option<String>,
        parameter_name: String,
    },
}

impl KeySource {
    pub(crate) fn as_keypair(&self) -> Result<KeyPair> {
        match self {
            KeySource::Local(path) => {
                let buf = std::fs::read(path).context(error::FileRead { path })?;
                KeyPair::parse(&buf)
            }
            #[cfg(any(feature = "rusoto-native-tls", feature = "rusoto-rustls"))]
            KeySource::Ssm {
                profile,
                parameter_name,
            } => {
                use crate::deref::OptionDeref;
                use rusoto_ssm::Ssm;

                let ssm_client = crate::ssm::build_client(profile.deref_shim())?;
                let response = ssm_client
                    .get_parameter(rusoto_ssm::GetParameterRequest {
                        name: parameter_name.to_owned(),
                        with_decryption: Some(true),
                    })
                    .sync()
                    .context(error::SsmGetParameter {
                        profile: profile.clone(),
                        parameter_name,
                    })?;
                KeyPair::parse(
                    response
                        .parameter
                        .context(error::SsmMissingField { field: "parameter" })?
                        .value
                        .context(error::SsmMissingField {
                            field: "parameter.value",
                        })?
                        .as_bytes(),
                )
            }
        }
    }
}

impl FromStr for KeySource {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let pwd_url = Url::from_directory_path(std::env::current_dir().context(error::CurrentDir)?)
            .expect("expected current directory to be absolute");
        let url = Url::options()
            .base_url(Some(&pwd_url))
            .parse(s)
            .context(error::UrlParse { url: s })?;

        match url.scheme() {
            "file" => Ok(Self::Local(PathBuf::from(url.path()))),
            #[cfg(any(feature = "rusoto-native-tls", feature = "rusoto-rustls"))]
            "aws-ssm" => Ok(Self::Ssm {
                profile: url.host_str().and_then(|s| {
                    if s.is_empty() {
                        None
                    } else {
                        Some(s.to_owned())
                    }
                }),
                parameter_name: url.path().to_owned(),
            }),
            _ => error::UnrecognizedScheme {
                scheme: url.scheme(),
            }
            .fail(),
        }
    }
}
