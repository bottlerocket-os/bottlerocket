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
use tough_schema::key::Key;
use url::Url;

#[derive(Debug)]
pub(crate) enum KeySource {
    Local(PathBuf),
    #[cfg(any(feature = "rusoto-native-tls", feature = "rusoto-rustls"))]
    Ssm {
        profile: Option<String>,
        parameter_name: String,
        key_id: Option<String>,
    },
}

impl KeySource {
    pub(crate) fn as_keypair(&self) -> Result<KeyPair> {
        KeyPair::parse(&self.read()?)
    }

    pub(crate) fn as_public_key(&self) -> Result<Key> {
        let data = self.read()?;
        if let Ok(key_pair) = KeyPair::parse(&data) {
            Ok(key_pair.public_key())
        } else {
            let data = String::from_utf8(data)
                .ok()
                .context(error::UnrecognizedKey)?;
            Key::from_str(&data).ok().context(error::UnrecognizedKey)
        }
    }

    fn read(&self) -> Result<Vec<u8>> {
        match self {
            KeySource::Local(path) => std::fs::read(path).context(error::FileRead { path }),
            #[cfg(any(feature = "rusoto-native-tls", feature = "rusoto-rustls"))]
            KeySource::Ssm {
                profile,
                parameter_name,
                ..
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
                Ok(response
                    .parameter
                    .context(error::SsmMissingField { field: "parameter" })?
                    .value
                    .context(error::SsmMissingField {
                        field: "parameter.value",
                    })?
                    .as_bytes()
                    .to_vec())
            }
        }
    }

    #[cfg_attr(
        not(any(feature = "rusoto-native-tls", feature = "rusoto-rustls")),
        allow(unused)
    )]
    pub(crate) fn write(&self, value: &str, key_id_hex: &str) -> Result<()> {
        match self {
            KeySource::Local(path) => {
                std::fs::write(path, value.as_bytes()).context(error::FileWrite { path })
            }
            #[cfg(any(feature = "rusoto-native-tls", feature = "rusoto-rustls"))]
            KeySource::Ssm {
                profile,
                parameter_name,
                key_id,
            } => {
                use crate::deref::OptionDeref;
                use rusoto_ssm::Ssm;

                let ssm_client = crate::ssm::build_client(profile.deref_shim())?;
                ssm_client
                    .put_parameter(rusoto_ssm::PutParameterRequest {
                        name: parameter_name.to_owned(),
                        description: Some(key_id_hex.to_owned()),
                        key_id: key_id.as_ref().cloned(),
                        overwrite: Some(true),
                        type_: "SecureString".to_owned(),
                        value: value.to_owned(),
                        ..rusoto_ssm::PutParameterRequest::default()
                    })
                    .sync()
                    .context(error::SsmPutParameter {
                        profile: profile.clone(),
                        parameter_name,
                    })?;
                Ok(())
            }
        }
    }
}

impl FromStr for KeySource {
    type Err = Error;

    #[allow(clippy::find_map)]
    fn from_str(s: &str) -> Result<Self> {
        let pwd_url = Url::from_directory_path(std::env::current_dir().context(error::CurrentDir)?)
            .expect("expected current directory to be absolute");
        let url = Url::options()
            .base_url(Some(&pwd_url))
            .parse(s)
            .context(error::UrlParse { url: s })?;

        match url.scheme() {
            "file" => Ok(KeySource::Local(PathBuf::from(url.path()))),
            #[cfg(any(feature = "rusoto-native-tls", feature = "rusoto-rustls"))]
            "aws-ssm" => Ok(KeySource::Ssm {
                profile: url.host_str().and_then(|s| {
                    if s.is_empty() {
                        None
                    } else {
                        Some(s.to_owned())
                    }
                }),
                parameter_name: url.path().to_owned(),
                key_id: url
                    .query_pairs()
                    .find(|(k, _)| k == "kms-key-id")
                    .map(|(_, v)| v.into_owned()),
            }),
            _ => error::UnrecognizedScheme {
                scheme: url.scheme(),
            }
            .fail(),
        }
    }
}
