// Note: this only allows reading and writing UTF-8 keys and values; is that OK?

use lazy_static::lazy_static;
use regex::Regex;
use std::borrow::Borrow;
use std::fmt;
use std::ops::Deref;

use super::{DataStoreError, Result};

pub const KEY_SEPARATOR: &str = ".";

/// Maximum key name length matches the maximum filename length of 255; if we need to have longer
/// keys (up to 4096) we could make prefixes not count against this limit.
const MAX_KEY_NAME_LENGTH: usize = 255;

#[rustfmt::skip]
lazy_static! {
    /// Pattern to validate a data key.
    static ref DATA_KEY: Regex = Regex::new(r"(?x)^
        (
            [a-zA-Z0-9_-]+
        \.)*                # optional dot-separated prefix segments
        [a-zA-Z0-9_-]+      # final name segment
    $").unwrap();

    /// Pattern to validate a metadata key.
    static ref METADATA_KEY: Regex = Regex::new(r"(?x)^
        [a-zA-Z0-9_-]+      # no prefixes, just one name segment
    $").unwrap();
}

/// KeyType represents whether we want to check a Key as a data key or metadata key.
#[derive(Debug, Copy, Clone)]
pub enum KeyType {
    Data,
    Meta,
}

/// A Key is a pointer into the datastore with a convenient name.  Their names are simply dotted
/// strings ("a.b.c") with the dots implying hierarchy, so "a.b.c" and "a.b.d" are probably
/// related.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Key {
    name: String,
}

impl Key {
    pub fn new<S: AsRef<str>>(key_type: KeyType, name: S) -> Result<Key> {
        let name = name.as_ref();
        if name.len() > MAX_KEY_NAME_LENGTH {
            return Err(DataStoreError::InvalidKey(format!(
                "Key name beyond maximum length {}: {}...",
                MAX_KEY_NAME_LENGTH,
                &name[0..50]
            )));
        }

        let name_pattern = match key_type {
            KeyType::Data => &*DATA_KEY,
            KeyType::Meta => &*METADATA_KEY,
        };

        if !name_pattern.is_match(name) {
            // Showing the real regex is ugly because of ?x and its formatting
            return Err(DataStoreError::InvalidKey(format!(
                "Key name '{}' has invalid format, should be 1 or more dot-separated [a-zA-Z0-9_-]+",
                name
            )));
        }

        let copy = name.to_string();
        Ok(Key { name: copy })
    }
}

// These trait implementations let you treat a Key like a string most of the time.

impl Deref for Key {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.name
    }
}

impl Borrow<String> for Key {
    fn borrow(&self) -> &String {
        &self.name
    }
}

impl Borrow<str> for Key {
    fn borrow(&self) -> &str {
        &self.name
    }
}

impl AsRef<str> for Key {
    fn as_ref(&self) -> &str {
        &self.name
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[cfg(test)]
mod test {
    use super::{Key, KeyType, MAX_KEY_NAME_LENGTH};

    // Helper macro for testing conditions that apply to both data and metadata keys
    macro_rules! data_and_meta {
        ($fn:expr) => {
            $fn(KeyType::Data);
            $fn(KeyType::Meta);
        };
    }

    #[test]
    fn short_key_ok() {
        data_and_meta!(|t| assert!(Key::new(t, "a").is_ok()));
    }

    #[test]
    fn nested_data_key_ok() {
        assert!(Key::new(KeyType::Data, "a.b.c.d.e.f.g").is_ok());
    }

    #[test]
    fn nested_metadata_key_fails() {
        assert!(Key::new(KeyType::Meta, "a.b.c.d.e.f.g").is_err());
    }

    #[test]
    fn key_with_special_chars_ok() {
        data_and_meta!(|t| assert!(Key::new(t, "a-b_c").is_ok()));
    }

    #[test]
    fn long_key_ok() {
        data_and_meta!(|t| assert!(Key::new(t, "a".repeat(MAX_KEY_NAME_LENGTH)).is_ok()));
    }

    #[test]
    fn key_too_long() {
        data_and_meta!(|t| assert!(Key::new(t, "a".repeat(MAX_KEY_NAME_LENGTH + 1)).is_err()));
    }

    #[test]
    fn key_bad_chars() {
        data_and_meta!(|t| assert!(Key::new(t, "!").is_err()));
        data_and_meta!(|t| assert!(Key::new(t, "$").is_err()));
        data_and_meta!(|t| assert!(Key::new(t, "&").is_err()));
        data_and_meta!(|t| assert!(Key::new(t, ";").is_err()));
        data_and_meta!(|t| assert!(Key::new(t, "|").is_err()));
        data_and_meta!(|t| assert!(Key::new(t, r"\").is_err()));
    }

    #[test]
    fn key_bad_format() {
        data_and_meta!(|t| assert!(Key::new(t, "a.").is_err()));
    }
}
