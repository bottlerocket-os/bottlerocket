use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;
use std::fmt::{self, Debug, Display};
use std::marker::PhantomData;

pub(crate) struct Conv<T> {
    bytes: Vec<u8>,
    original: String,
    spooky: PhantomData<T>,
}

impl<T> Conv<T> {
    pub(crate) fn as_slice(&self) -> &[u8] {
        &self.bytes
    }

    pub(crate) fn into_vec(self) -> Vec<u8> {
        self.bytes
    }
}

impl<T> Debug for Conv<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.original, f)
    }
}

impl<T> Display for Conv<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.original, f)
    }
}

impl<T> Clone for Conv<T> {
    fn clone(&self) -> Self {
        Self {
            bytes: self.bytes.clone(),
            original: self.original.clone(),
            spooky: PhantomData,
        }
    }
}

impl<T> PartialEq for Conv<T> {
    fn eq(&self, other: &Self) -> bool {
        self.bytes.eq(&other.bytes)
    }
}

impl<T> Eq for Conv<T> {}

impl<T> PartialOrd for Conv<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.bytes.partial_cmp(&other.bytes)
    }
}

impl<T> Ord for Conv<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.bytes.cmp(&other.bytes)
    }
}

impl<'de, T: ConvTrait> Deserialize<'de> for Conv<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let original = String::deserialize(deserializer)?;
        Ok(Self {
            bytes: T::parse::<D>(&original)?,
            original,
            spooky: PhantomData,
        })
    }
}

impl<T: ConvTrait> Serialize for Conv<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.original)
    }
}

pub(crate) trait ConvTrait: Sized {
    fn parse<'de, D>(s: &str) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>;
}

pub(crate) struct Hex;

impl ConvTrait for Hex {
    fn parse<'de, D>(s: &str) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        hex::decode(s).map_err(|err| D::Error::custom(format!("invalid hex string: {}", err)))
    }
}

pub(crate) struct Pem;

impl ConvTrait for Pem {
    fn parse<'de, D>(s: &str) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        pem::parse(s)
            .map(|pem| pem.contents)
            .map_err(|err| D::Error::custom(format!("invalid PEM string: {}", err)))
    }
}
