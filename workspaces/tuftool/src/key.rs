use crate::error::{self, Result};
use ring::rand::SecureRandom;
use ring::signature::{KeyPair as _, RsaKeyPair};
use snafu::ResultExt;
use tough_schema::key::Key;

#[derive(Debug)]
pub(crate) enum KeyPair {
    Rsa(RsaKeyPair),
}

impl KeyPair {
    pub(crate) fn parse(key: &[u8]) -> Result<Self> {
        if let Ok(pem) = pem::parse(key) {
            match pem.tag.as_str() {
                "RSA PRIVATE KEY" => Ok(KeyPair::Rsa(
                    RsaKeyPair::from_der(&pem.contents).context(error::KeyRejected)?,
                )),
                _ => error::KeyUnrecognized.fail(),
            }
        } else {
            error::KeyUnrecognized.fail()
        }
    }

    pub(crate) fn sign(&self, msg: &[u8], rng: &dyn SecureRandom) -> Result<Vec<u8>> {
        match self {
            KeyPair::Rsa(key_pair) => {
                let mut signature = vec![0; key_pair.public_modulus_len()];
                key_pair
                    .sign(&ring::signature::RSA_PSS_SHA256, rng, msg, &mut signature)
                    .context(error::Sign)?;
                Ok(signature)
            }
        }
    }
}

impl PartialEq<Key> for KeyPair {
    fn eq(&self, key: &Key) -> bool {
        match (self, key) {
            (KeyPair::Rsa(key_pair), Key::Rsa { keyval, .. }) => {
                key_pair.public_key().as_ref() == keyval.public.as_ref()
            }
            _ => false,
        }
    }
}
