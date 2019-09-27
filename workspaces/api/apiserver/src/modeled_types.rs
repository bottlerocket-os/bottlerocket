//! This module contains data types that can be used in the model when special input/output
//! (ser/de) behavior is desired.  For example, the ValidBase64 type can be used for a model field
//! when we don't even want to accept an API call with invalid base64 data.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
// Just need serde's Error in scope to get its trait methods
use serde::de::Error as _;
use std::borrow::Borrow;
use std::fmt;
use std::ops::Deref;

/// ValidBase64 can only be created by deserializing from valid base64 text.  It stores the
/// original text, not the decoded form.  Its purpose is input validation, namely being used as a
/// field in a model structure so that you don't even accept a request with a field that has
/// invalid base64.
// Note: we use the default base64::STANDARD config which uses/allows "=" padding.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ValidBase64 {
    inner: String,
}

/// Validate base64 format before we accept the input.
impl<'de> Deserialize<'de> for ValidBase64 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let original = String::deserialize(deserializer)?;
        base64::decode(&original)
            .map_err(|e| D::Error::custom(format!("Invalid base64: {}", e)))?;
        Ok(ValidBase64 { inner: original })
    }
}

/// We want to serialize the original string back out, not our structure, which is just there to
/// force validation.
impl Serialize for ValidBase64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.inner)
    }
}

impl Deref for ValidBase64 {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Borrow<String> for ValidBase64 {
    fn borrow(&self) -> &String {
        &self.inner
    }
}

impl Borrow<str> for ValidBase64 {
    fn borrow(&self) -> &str {
        &self.inner
    }
}

impl AsRef<str> for ValidBase64 {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl fmt::Display for ValidBase64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[cfg(test)]
mod test {
    use super::ValidBase64;

    #[test]
    fn valid_base64() {
        let v: ValidBase64 = serde_json::from_str("\"aGk=\"").unwrap();
        let decoded_bytes = base64::decode(v.as_ref()).unwrap();
        let decoded = std::str::from_utf8(&decoded_bytes).unwrap();
        assert_eq!(decoded, "hi");
    }

    #[test]
    fn invalid_base64() {
        assert!(serde_json::from_str::<ValidBase64>("\"invalid base64\"").is_err());
        assert!(serde_json::from_str::<ValidBase64>("").is_err());
    }
}
