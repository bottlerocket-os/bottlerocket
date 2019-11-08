#![allow(clippy::use_self)]

use crate::schema::decoded::{Decoded, EcdsaPem, Hex, RsaPem};
use crate::schema::error::{self, Result};
use olpc_cjson::CanonicalFormatter;
use ring::signature::VerificationAlgorithm;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use snafu::ResultExt;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "keytype")]
pub enum Key {
    Rsa {
        keyval: RsaKey,
        scheme: RsaScheme,

        #[serde(flatten)]
        _extra: HashMap<String, Value>,
    },
    Ed25519 {
        keyval: Ed25519Key,
        scheme: Ed25519Scheme,

        #[serde(flatten)]
        _extra: HashMap<String, Value>,
    },
    Ecdsa {
        keyval: EcdsaKey,
        scheme: EcdsaScheme,

        #[serde(flatten)]
        _extra: HashMap<String, Value>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum RsaScheme {
    RsassaPssSha256,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct RsaKey {
    pub public: Decoded<RsaPem>,

    #[serde(flatten)]
    pub _extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Ed25519Scheme {
    Ed25519,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Ed25519Key {
    pub public: Decoded<Hex>,

    #[serde(flatten)]
    pub _extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum EcdsaScheme {
    EcdsaSha2Nistp256,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct EcdsaKey {
    pub public: Decoded<EcdsaPem>,

    #[serde(flatten)]
    pub _extra: HashMap<String, Value>,
}

impl Key {
    /// Calculate the key ID for this key.
    pub fn key_id(&self) -> Result<Decoded<Hex>> {
        let mut buf = Vec::new();
        let mut ser = serde_json::Serializer::with_formatter(&mut buf, CanonicalFormatter::new());
        self.serialize(&mut ser).context(error::JsonSerialization {
            what: "key".to_owned(),
        })?;
        Ok(Sha256::digest(&buf).as_slice().to_vec().into())
    }

    /// Verify a signature of an object made with this key.
    pub(super) fn verify(&self, msg: &[u8], signature: &[u8]) -> bool {
        let (alg, public_key): (&dyn VerificationAlgorithm, untrusted::Input<'_>) = match self {
            Key::Ecdsa {
                scheme: EcdsaScheme::EcdsaSha2Nistp256,
                keyval,
                ..
            } => (
                &ring::signature::ECDSA_P256_SHA256_ASN1,
                untrusted::Input::from(&keyval.public),
            ),
            Key::Ed25519 {
                scheme: Ed25519Scheme::Ed25519,
                keyval,
                ..
            } => (
                &ring::signature::ED25519,
                untrusted::Input::from(&keyval.public),
            ),
            Key::Rsa {
                scheme: RsaScheme::RsassaPssSha256,
                keyval,
                ..
            } => (
                &ring::signature::RSA_PSS_2048_8192_SHA256,
                untrusted::Input::from(&keyval.public),
            ),
        };

        alg.verify(
            public_key,
            untrusted::Input::from(msg),
            untrusted::Input::from(signature),
        )
        .is_ok()
    }
}

impl FromStr for Key {
    type Err = KeyParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if let Ok(public) = serde_plain::from_str::<Decoded<RsaPem>>(s) {
            Ok(Key::Rsa {
                keyval: RsaKey {
                    public,
                    _extra: HashMap::new(),
                },
                scheme: RsaScheme::RsassaPssSha256,
                _extra: HashMap::new(),
            })
        } else if let Ok(public) = serde_plain::from_str::<Decoded<Hex>>(s) {
            if public.len() == ring::signature::ED25519_PUBLIC_KEY_LEN {
                Ok(Key::Ed25519 {
                    keyval: Ed25519Key {
                        public,
                        _extra: HashMap::new(),
                    },
                    scheme: Ed25519Scheme::Ed25519,
                    _extra: HashMap::new(),
                })
            } else {
                Err(KeyParseError(()))
            }
        } else if let Ok(public) = serde_plain::from_str::<Decoded<EcdsaPem>>(s) {
            Ok(Key::Ecdsa {
                keyval: EcdsaKey {
                    public,
                    _extra: HashMap::new(),
                },
                scheme: EcdsaScheme::EcdsaSha2Nistp256,
                _extra: HashMap::new(),
            })
        } else {
            Err(KeyParseError(()))
        }
    }
}

#[derive(Debug)]
pub struct KeyParseError(());

impl fmt::Display for KeyParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unrecognized or invalid public key")
    }
}

impl std::error::Error for KeyParseError {}
