use crate::error::{self, Result};
use crate::source::KeySource;
use olpc_cjson::CanonicalFormatter;
use ring::rand::SecureRandom;
use ring::signature::{KeyPair as _, RsaKeyPair};
use serde::Serialize;
use snafu::ResultExt;
use std::collections::HashMap;
use tough::schema::decoded::{Decoded, Hex};
use tough::schema::key::Key;
use tough::schema::{Role, RoleType, Root, Signature, Signed};

#[derive(Debug)]
pub(crate) enum KeyPair {
    Rsa(RsaKeyPair),
}

impl KeyPair {
    pub(crate) fn parse(key: &[u8]) -> Result<Self> {
        if let Ok(pem) = pem::parse(key) {
            match pem.tag.as_str() {
                "PRIVATE KEY" => {
                    if let Ok(key_pair) = RsaKeyPair::from_pkcs8(&pem.contents) {
                        Ok(KeyPair::Rsa(key_pair))
                    } else {
                        error::KeyUnrecognized.fail()
                    }
                }
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

    pub(crate) fn public_key(&self) -> Key {
        use tough::schema::key::{RsaKey, RsaScheme};

        match self {
            KeyPair::Rsa(key_pair) => Key::Rsa {
                keyval: RsaKey {
                    public: key_pair.public_key().as_ref().to_vec().into(),
                    _extra: HashMap::new(),
                },
                scheme: RsaScheme::RsassaPssSha256,
                _extra: HashMap::new(),
            },
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

pub(crate) type RootKeys = HashMap<Decoded<Hex>, KeyPair>;

pub(crate) fn keys_for_root(keys: &[KeySource], root: &Root) -> Result<RootKeys> {
    let mut map = HashMap::new();
    for source in keys {
        let key_pair = source.as_keypair()?;
        if let Some((keyid, _)) = root.keys.iter().find(|(_, key)| key_pair == **key) {
            map.insert(keyid.clone(), key_pair);
        }
    }
    Ok(map)
}

pub(crate) fn sign_metadata<T: Role + Serialize>(
    root: &Root,
    keys: &RootKeys,
    role: &mut Signed<T>,
    rng: &dyn SecureRandom,
) -> Result<()> {
    sign_metadata_inner(root, keys, T::TYPE, role, rng)
}

pub(crate) fn sign_metadata_inner<T: Serialize>(
    root: &Root,
    keys: &RootKeys,
    role_type: RoleType,
    role: &mut Signed<T>,
    rng: &dyn SecureRandom,
) -> Result<()> {
    if let Some(role_keys) = root.roles.get(&role_type) {
        for (keyid, key) in keys {
            if role_keys.keyids.contains(&keyid) {
                let mut data = Vec::new();
                let mut ser =
                    serde_json::Serializer::with_formatter(&mut data, CanonicalFormatter::new());
                role.signed.serialize(&mut ser).context(error::SignJson)?;
                let sig = key.sign(&data, rng)?;
                role.signatures.push(Signature {
                    keyid: keyid.clone(),
                    sig: sig.into(),
                });
            }
        }
    }

    Ok(())
}
