use crate::error::{self, Compat, Error};
use ring::io::der;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use snafu::ResultExt;
use std::cmp::Ordering;
use std::fmt::{self, Display};
use std::marker::PhantomData;
use std::ops::Deref;

/// Represents bytes decoded from a string.
///
/// The type parameter `T` represents what kind of data the original string stores (e.g.
/// hex-encoded bytes, or a PEM-encoded key).
///
/// The original string is stored so that it can be re-`Serialize`d for the purposes of verifying
/// signatures.
#[derive(Debug, Clone)]
pub(crate) struct Decoded<T: Decode> {
    bytes: Vec<u8>,
    original: String,
    spooky: PhantomData<T>,
}

impl<T: Decode> Decoded<T> {
    /// Consume this object and return its decoded bytes.
    pub(crate) fn into_vec(self) -> Vec<u8> {
        self.bytes
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// A trait that represents how data can be converted from a string to bytes.
///
/// Generally structs that implement `Decode` will be unit-like structs that just implement the one
/// required method.
pub(crate) trait Decode {
    /// Convert a string to bytes.
    ///
    /// The "error" string returned from this method will immediately be wrapped into a
    /// [`serde::de::Error`].
    fn decode(s: &str) -> Result<Vec<u8>, Error>;
}

/// [`Decode`] implementation for hex-encoded strings.
#[derive(Debug, Clone)]
pub(crate) struct Hex;

impl Decode for Hex {
    fn decode(s: &str) -> Result<Vec<u8>, Error> {
        hex::decode(s).context(error::HexDecode)
    }
}

/// [`Decode`] implementation for PEM-encoded keys.
#[derive(Debug, Clone)]
pub(crate) struct Pem;

impl Decode for Pem {
    fn decode(s: &str) -> Result<Vec<u8>, Error> {
        pem::parse(s)
            .map(|pem| pem.contents)
            .map_err(Compat)
            .context(error::PemDecode)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RsaPem;

impl Decode for RsaPem {
    fn decode(s: &str) -> Result<Vec<u8>, Error> {
        let pem = pem::parse(s).map_err(Compat).context(error::PemDecode)?;
        // All TUF says about RSA keys is that they are "in PEM format and a string", but tests in
        // TUF's source code repository [1] imply that they are the sort of output you expect from
        // `openssl genrsa`. This is the SubjectPublicKeyInfo format, and ring wants the
        // RSAPublicKey format [2].
        //
        // If you run the public key from [1] through `openssl asn1parse -i`, you get:
        //
        // ```
        //    0:d=0  hl=4 l= 418 cons: SEQUENCE
        //    4:d=1  hl=2 l=  13 cons:  SEQUENCE
        //    6:d=2  hl=2 l=   9 prim:   OBJECT            :rsaEncryption
        //   17:d=2  hl=2 l=   0 prim:   NULL
        //   19:d=1  hl=4 l= 399 prim:  BIT STRING
        // ```
        //
        // The BIT STRING (here, at offset 19) happens to be the RSAPublicKey format. Here, we use
        // ring's (undocumented but public?!) DER-parsing methods to get there.
        //
        // [1]: https://github.com/theupdateframework/tuf/blob/49e75ffe5adfc1f883f53f658ace596d14dc0879/tests/repository_data/repository/metadata/root.json#L20
        // [2]: https://docs.rs/ring/0.14.6/ring/signature/index.html#signing-and-verifying-with-rsa-pkcs1-15-padding
        match untrusted::Input::from(&pem.contents).read_all(ring::error::Unspecified, |input| {
            der::expect_tag_and_get_value(input, der::Tag::Sequence).and_then(|spki_inner| {
                spki_inner.read_all(ring::error::Unspecified, |input| {
                    der::expect_tag_and_get_value(input, der::Tag::Sequence)?;
                    der::bit_string_with_no_unused_bits(input)
                })
            })
        }) {
            Ok(key_value) => Ok(key_value.as_slice_less_safe().to_owned()),
            Err(_) => error::RsaDecode.fail(),
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

impl<'de, T: Decode> Deserialize<'de> for Decoded<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let original = String::deserialize(deserializer)?;
        Ok(Self {
            bytes: T::decode(&original).map_err(D::Error::custom)?,
            original,
            spooky: PhantomData,
        })
    }
}

impl<T: Decode> Serialize for Decoded<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.original)
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

impl<T: Decode> AsRef<[u8]> for Decoded<T> {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl<T: Decode> Deref for Decoded<T> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.bytes
    }
}

impl<T: Decode> Display for Decoded<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.original, f)
    }
}

impl<T: Decode> PartialEq for Decoded<T> {
    fn eq(&self, other: &Self) -> bool {
        self.bytes.eq(&other.bytes)
    }
}

impl<T: Decode> Eq for Decoded<T> {}

impl<T: Decode> PartialOrd for Decoded<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.bytes.partial_cmp(&other.bytes)
    }
}

impl<T: Decode> Ord for Decoded<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.bytes.cmp(&other.bytes)
    }
}

impl<T: Decode> PartialEq<[u8]> for Decoded<T> {
    fn eq(&self, other: &[u8]) -> bool {
        self.bytes.eq(&other)
    }
}
