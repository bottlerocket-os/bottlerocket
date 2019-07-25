use crate::serde::decoded::{Decoded, Hex, Pem, RsaPem};
use ring::signature::VerificationAlgorithm;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "keytype")]
pub enum Key {
    Ecdsa {
        keyval: EcdsaKey,
        scheme: EcdsaScheme,
    },
    Ed25519 {
        keyval: Ed25519Key,
        scheme: Ed25519Scheme,
    },
    Rsa {
        keyval: RsaKey,
        scheme: RsaScheme,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum EcdsaScheme {
    EcdsaSha2Nistp256,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct EcdsaKey {
    // FIXME: there's probably a difference between what TUF thinks is a valid ECDSA key and what
    // ring thinks is a valid ECDSA key (similar to the issue we had with RSA).
    public: Decoded<Pem>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Ed25519Scheme {
    Ed25519,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Ed25519Key {
    public: Decoded<Hex>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RsaScheme {
    RsassaPssSha256,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct RsaKey {
    public: Decoded<RsaPem>,
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

        ring::signature::verify(
            alg,
            public_key,
            untrusted::Input::from(msg),
            untrusted::Input::from(signature),
        )
        .is_ok()
    }
}
