use crate::decoded::{Decoded, EcdsaPem, Hex, RsaPem};
use ring::signature::VerificationAlgorithm;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "keytype")]
pub enum Key {
    Rsa {
        keyval: RsaKey,
        scheme: RsaScheme,
    },
    Ed25519 {
        keyval: Ed25519Key,
        scheme: Ed25519Scheme,
    },
    Ecdsa {
        keyval: EcdsaKey,
        scheme: EcdsaScheme,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RsaScheme {
    RsassaPssSha256,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct RsaKey {
    pub public: Decoded<RsaPem>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Ed25519Scheme {
    Ed25519,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Ed25519Key {
    pub public: Decoded<Hex>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum EcdsaScheme {
    EcdsaSha2Nistp256,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct EcdsaKey {
    pub public: Decoded<EcdsaPem>,
}

impl Key {
    /// Verify a signature of an object made with this key.
    pub(crate) fn verify(&self, msg: &[u8], signature: &[u8]) -> bool {
        let (alg, public_key): (&dyn VerificationAlgorithm, untrusted::Input) = match self {
            Key::Ecdsa {
                scheme: EcdsaScheme::EcdsaSha2Nistp256,
                keyval,
            } => (
                &ring::signature::ECDSA_P256_SHA256_ASN1,
                untrusted::Input::from(&keyval.public),
            ),
            Key::Ed25519 {
                scheme: Ed25519Scheme::Ed25519,
                keyval,
            } => (
                &ring::signature::ED25519,
                untrusted::Input::from(&keyval.public),
            ),
            Key::Rsa {
                scheme: RsaScheme::RsassaPssSha256,
                keyval,
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
                keyval: RsaKey { public },
                scheme: RsaScheme::RsassaPssSha256,
            })
        } else if let Ok(public) = serde_plain::from_str::<Decoded<Hex>>(s) {
            if public.len() == ring::signature::ED25519_PUBLIC_KEY_LEN {
                Ok(Key::Ed25519 {
                    keyval: Ed25519Key { public },
                    scheme: Ed25519Scheme::Ed25519,
                })
            } else {
                Err(KeyParseError(()))
            }
        } else if let Ok(public) = serde_plain::from_str::<Decoded<EcdsaPem>>(s) {
            Ok(Key::Ecdsa {
                keyval: EcdsaKey { public },
                scheme: EcdsaScheme::EcdsaSha2Nistp256,
            })
        } else {
            Err(KeyParseError(()))
        }
    }
}

#[derive(Debug)]
pub struct KeyParseError(());

impl fmt::Display for KeyParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unrecognized or invalid public key")
    }
}

impl std::error::Error for KeyParseError {}
