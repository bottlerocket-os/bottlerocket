// Note: this only allows reading and writing UTF-8 keys and values; is that OK?

use log::trace;
use serde::{Serialize, Serializer};
use snafu::ensure;
use std::fmt;
use std::hash::{Hash, Hasher};

use super::{error, Result};

pub const KEY_SEPARATOR: char = '.';
// String refs are more convenient for some Rust functions
pub const KEY_SEPARATOR_STR: &str = ".";

/// Maximum key name length matches the maximum filename length of 255; if we need to have longer
/// keys (up to 4096) we could make prefixes not count against this limit.
const MAX_KEY_NAME_LENGTH: usize = 255;

/// KeyType represents whether we want to check a Key as a data key or metadata key.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum KeyType {
    Data,
    Meta,
}

/// A Key is a pointer into the datastore with a convenient name.  Their names are simply dotted
/// strings ("a.b.c") with the dots implying hierarchy, so "a.b.c" and "a.b.d" are probably
/// related.
///
/// Keys that need to include dots in the name can quote that segment of the name, for example the
/// key a."b.c".d has three segments: "a", "b.c", and "d".
#[derive(Clone, Debug)]
pub struct Key {
    name: String,
    segments: Vec<String>,
}

impl Key {
    /// Returns a list of the segments that make up the key name.
    ///
    /// Examples:
    /// * a.b.c -> ["a", "b", "c"]
    /// * "a.b".c -> ["a.b", "c"]
    pub fn segments(&self) -> &Vec<String> {
        &self.segments
    }

    /// Returns the name of the key.
    ///
    /// If you created the Key using with_segments(), the segments are quoted as necessary to
    /// handle special characters.  Examples:
    /// * created with segments ["a", "b", "c"] -> a.b.c
    /// * created with segments ["a.b", "c"] -> "a.b".c
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Creates a Key of the given type from the given name.
    ///
    /// If there are special characters in the name, like "." which is used as a separator,
    /// then you should quote that segment, for example: a."b.c".d to represent three segments
    /// "a", "b.c", and "d".  If possible, you should use `Key::from_segments` instead, to more
    /// accurately represent the individual segments.
    pub fn new<S: AsRef<str>>(key_type: KeyType, name: S) -> Result<Self> {
        let segments = Self::parse_name_segments(&name)?;

        Self::check_key(key_type, &name, &segments)?;

        Ok(Self {
            name: name.as_ref().to_string(),
            segments,
        })
    }

    /// Creates a Key of the given type from the given name segments.
    ///
    /// For example, passing &["a", "b.c", "c"] will create a key named: a."b.c".c
    pub fn from_segments<S>(key_type: KeyType, segments: &[S]) -> Result<Self>
    where
        S: AsRef<str>,
    {
        let name = Self::encode_name_segments(segments)?;

        Self::check_key(key_type, &name, segments)?;

        Ok(Self {
            name,
            segments: segments.iter().map(|s| s.as_ref().into()).collect(),
        })
    }

    /// Removes the given prefix from the key name, returning a new Key.
    ///
    /// This is intended to remove key name segments from the beginning of the name, therefore
    /// this only makes sense for Data keys, not Meta keys.  A Data key will be returned.
    ///
    /// You should not include an ending separator (dot), it will be removed for you.
    ///
    /// If the key name does not begin with the given prefix, the returned key will be
    /// identical.
    ///
    /// Fails if the new key would be invalid, e.g. if the prefix is the entire key.
    pub(super) fn strip_prefix<S>(&self, prefix: S) -> Result<Self>
    where
        S: AsRef<str>,
    {
        let prefix = prefix.as_ref();
        ensure!(
            prefix != self.name,
            error::InvalidKeySnafu {
                name: "",
                msg: format!("strip_prefix of '{}' matches key", prefix)
            }
        );

        let strip = prefix.to_string() + ".";

        // Check starts_with so we don't replace in the middle of the string...
        let name = if self.name.starts_with(&strip) {
            self.name.replacen(&strip, "", 1)
        } else {
            self.name.clone()
        };

        Self::new(KeyType::Data, name)
    }

    /// Removes the given key segments from the beginning of the key, returning a new Key.
    ///
    /// This only makes sense for Data keys because Meta keys only have one segment.  A Data key
    /// will be returned.
    ///
    /// If the key does not begin with all of the given segments, no segments will be removed,
    /// so the returned key will be identical.
    ///
    /// Fails if the new key would be invalid, e.g. if the given segments are the entire key.
    pub(super) fn strip_prefix_segments<S>(&self, prefix: &[S]) -> Result<Self>
    where
        S: AsRef<str>,
    {
        // We walk through the given prefix segments, looking for anything that doesn't match
        // our segments, at which point we know we're going to return an unchanged key.
        for (i, theirs) in prefix.iter().enumerate() {
            match self.segments().get(i) {
                // If we run out of our segments, the prefix is longer than the existing key,
                // and therefore can't match; we return an unchanged key.
                None => return Ok(self.clone()),
                Some(ours) => {
                    // Difference found; return an unchanged key.
                    if ours != theirs.as_ref() {
                        return Ok(self.clone());
                    }
                }
            }
        }

        // No differences were found, so we remove the given segments.
        Self::from_segments(KeyType::Data, &self.segments[prefix.len()..])
    }

    /// Adds the given segments to the key name, returning a new Key.
    ///
    /// The given segments should not be quoted even if they contain the separator character;
    /// using a segment list allows us to be precise about the distinction between segments.
    ///
    /// Fails if the new key would be invalid, e.g. the suffix contains invalid characters.
    pub(super) fn append_segments<S>(&self, segments: &[S]) -> Result<Self>
    where
        S: AsRef<str>,
    {
        let our_segments = self.segments().iter().map(|s| s.as_ref());
        let their_segments = segments.iter().map(|s| s.as_ref());

        let new_segments: Vec<_> = our_segments.chain(their_segments).collect();
        Self::from_segments(KeyType::Data, &new_segments)
    }

    /// Adds the given key's name to this key name and returns a new Key.
    ///
    /// This is done precisely using each key's segments, so handling of separators and quoting
    /// is automatic.
    ///
    /// Fails if the new key would be invalid, e.g. too long.
    pub(super) fn append_key(&self, key: &Key) -> Result<Self> {
        let our_segments = self.segments().iter();
        let their_segments = key.segments().iter();

        let new_segments: Vec<_> = our_segments.chain(their_segments).collect();
        Self::from_segments(KeyType::Data, &new_segments)
    }

    /// Additional safety checks for parsed or generated keys.
    fn check_key<S1, S2>(key_type: KeyType, name: S1, segments: &[S2]) -> Result<()>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
    {
        let name = name.as_ref();

        ensure!(
            name.len() <= MAX_KEY_NAME_LENGTH,
            error::KeyTooLongSnafu {
                name,
                max: MAX_KEY_NAME_LENGTH,
            }
        );

        match key_type {
            KeyType::Data => {
                ensure!(
                    !segments.is_empty(),
                    error::InvalidKeySnafu {
                        name,
                        msg: "data keys must have at least one segment",
                    }
                );
            }
            KeyType::Meta => {
                ensure!(
                    segments.len() == 1,
                    error::InvalidKeySnafu {
                        name,
                        msg: "meta keys may only have one segment",
                    }
                );
            }
        }

        Ok(())
    }

    /// Determines whether a character is acceptable within a segment of a key name.  This is
    /// separate from quoting; if a character isn't valid, it isn't valid quoted, either.
    fn valid_character(c: char) -> bool {
        matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' | '/')
    }

    /// Given a key name, returns a list of its name segments, separated by KEY_SEPARATOR.
    /// Respects quoting of segments so they can contain dots.
    ///
    /// Examples:
    /// * a.b.c -> ["a", "b", "c"]
    /// * "a.b".c -> ["a.b", "c"]
    fn parse_name_segments<S: AsRef<str>>(name: S) -> Result<Vec<String>> {
        let name = name.as_ref();

        ensure!(
            !name.is_empty(),
            error::InvalidKeySnafu {
                name,
                msg: "cannot be empty",
            }
        );

        // The full list of name segments we're going to return.
        let mut segments = Vec::new();
        // The current name segment we're checking.
        let mut segment = String::new();
        // Track whether we're inside a quoted section of the key name
        let mut in_quotes = false;

        // Walk through each character, looking for quotes or separators to update state
        for c in name.chars() {
            if c == '"' {
                // Quotes don't go into the name segments, so we just flip the flag.
                in_quotes = !in_quotes;
            } else if c == KEY_SEPARATOR {
                if in_quotes {
                    // If we see a separator inside quotes, it's just like any other character.
                    segment.push(c);
                } else {
                    // If we see a separator outside quotes, it should be ending a segment.
                    // Segments can't be empty.
                    ensure!(
                        !segment.is_empty(),
                        error::InvalidKeySnafu {
                            name,
                            msg: "empty key segment",
                        }
                    );
                    // Save the segment we just saw and start a new one.
                    segments.push(segment);
                    segment = String::new();
                }
            } else {
                // Not a special character; make sure it's a valid part of a name segment.
                if Self::valid_character(c) {
                    segment.push(c);
                } else {
                    return error::InvalidKeySnafu {
                        name,
                        msg: format!("invalid character in key: '{}'", c),
                    }
                    .fail();
                }
            }
        }

        ensure!(
            !in_quotes,
            error::InvalidKeySnafu {
                name,
                msg: "unbalanced quotes",
            }
        );
        ensure!(
            !segment.is_empty(),
            error::InvalidKeySnafu {
                name,
                msg: "ends with separator",
            }
        );

        // Push final segment (keys don't end with a dot, which is when we normally push)
        segments.push(segment);

        trace!("Parsed key name '{}' to segments {:?}", name, segments);
        Ok(segments)
    }

    /// Given a list of key name segments, encodes them into a name string.  Any segments with
    /// special characters (like the separator) are quoted.
    fn encode_name_segments<S: AsRef<str>>(segments: &[S]) -> Result<String> {
        let segments: Vec<_> = segments.iter().map(|s| s.as_ref()).collect();
        let mut outputs = Vec::new();

        // Check whether we need quoting for each segment.
        for segment in segments.iter() {
            for chr in segment.chars() {
                ensure!(
                    chr == KEY_SEPARATOR || Self::valid_character(chr),
                    error::InvalidKeySnafu {
                        // Give an understandable key name in the error, even if it's invalid
                        name: segments.join("."),
                        msg: format!("Segment '{}' contains invalid character '{}'", segment, chr),
                    }
                );
            }

            if segment.chars().any(|c| c == KEY_SEPARATOR) {
                // Includes separator; quote the segment.
                outputs.push(format!("\"{}\"", segment));
            } else {
                // No special characters, no escaping needed.
                outputs.push(segment.to_string());
            }
        }

        // Join the (possibly quoted) segments with our separator.
        let name = outputs.join(KEY_SEPARATOR_STR);
        trace!("Encoded key '{}' from segments {:?}", name, segments);
        Ok(name)
    }

    pub fn starts_with_segments<S>(&self, segments: &[S]) -> bool
    where
        S: AsRef<str>,
    {
        if self.segments.len() < segments.len() {
            return false;
        }

        let ours = self.segments()[0..segments.len()].iter();
        let theirs = segments.iter().map(|s| s.as_ref());

        ours.zip(theirs).all(|(a, b)| a == b)
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

// We can't implement Deserialize for Key because Key doesn't store its key type, but we can
// serialize it to its name.
impl Serialize for Key {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.name)
    }
}

// The segments are our source of truth.
impl PartialEq for Key {
    fn eq(&self, other: &Key) -> bool {
        self.segments == other.segments
    }
}
impl Eq for Key {}
impl Hash for Key {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.segments.hash(state);
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
    fn dotted_data_key_ok() {
        assert!(Key::new(KeyType::Data, "a.b.c.d.e.f.g").is_ok());
    }

    #[test]
    fn dotted_metadata_key_fails() {
        assert!(Key::new(KeyType::Meta, "a.b.c.d.e.f.g").is_err());
    }

    #[test]
    fn quoted_data_key_ok() {
        let name = "a.\"b.c\".d";
        let key = Key::new(KeyType::Data, name).unwrap();
        assert_eq!(key.name(), name);
        assert_eq!(key.segments(), &["a", "b.c", "d"]);
    }

    #[test]
    fn quoted_metadata_key_ok() {
        // Metadata keys can only have one segment, but it can be quoted
        let name = "\"b.c\"";
        let key = Key::new(KeyType::Data, name).unwrap();
        assert_eq!(key.name(), name);
        assert_eq!(key.segments(), &["b.c"]);
    }

    #[test]
    fn from_segments() {
        let name = "a.\"b.c\".d";
        let segments = &["a", "b.c", "d"];
        let key = Key::from_segments(KeyType::Data, segments).unwrap();
        assert_eq!(key.name(), name);
        assert_eq!(key.segments(), segments);
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

    #[test]
    fn strip_prefix_ok() {
        // Remove plain prefix
        let key = Key::new(KeyType::Data, "a.b.c.d").unwrap();
        let prefix = "a.b";
        assert_eq!(key.strip_prefix(prefix).unwrap().name(), "c.d");

        // Don't remove non-matching prefix; no change
        let key = Key::new(KeyType::Data, "a.b.c.d").unwrap();
        let prefix = "x.y";
        assert_eq!(key.strip_prefix(prefix).unwrap().name(), "a.b.c.d");

        // Don't remove prefix that doesn't match whole quoted segment
        let key = Key::new(KeyType::Data, "a.\"b.c\".d").unwrap();
        let prefix = "a.b";
        assert_eq!(key.strip_prefix(prefix).unwrap().name(), "a.\"b.c\".d");

        // Do remove prefix that does match whole quoted segment
        let key = Key::new(KeyType::Data, "a.\"b.c\".d").unwrap();
        let prefix = "a.\"b.c\"";
        assert_eq!(key.strip_prefix(prefix).unwrap().name(), "d");
    }

    #[test]
    fn strip_prefix_err() {
        let key = Key::new(KeyType::Data, "a.b.c.d").unwrap();
        let prefix = "a.b.c.d";
        key.strip_prefix(prefix).unwrap_err();
    }

    #[test]
    fn strip_prefix_segments_ok() {
        // Remove plain prefix
        let key = Key::new(KeyType::Data, "a.b.c.d").unwrap();
        let prefix = &["a", "b"];
        assert_eq!(key.strip_prefix_segments(prefix).unwrap().name(), "c.d");

        // Don't remove non-matching prefix; no change
        let key = Key::new(KeyType::Data, "a.b.c.d").unwrap();
        let prefix = &["x", "y"];
        assert_eq!(key.strip_prefix_segments(prefix).unwrap().name(), "a.b.c.d");

        // Don't remove prefix that doesn't match whole quoted segment
        let key = Key::new(KeyType::Data, "a.\"b.c\".d").unwrap();
        let prefix = &["a", "b"];
        assert_eq!(
            key.strip_prefix_segments(prefix).unwrap().name(),
            "a.\"b.c\".d"
        );

        // Do remove prefix that does match whole quoted segment
        let key = Key::new(KeyType::Data, "a.\"b.c\".d").unwrap();
        let prefix = &["a", "b.c"];
        assert_eq!(key.strip_prefix_segments(prefix).unwrap().name(), "d");
    }

    #[test]
    fn strip_prefix_segments_err() {
        let key = Key::new(KeyType::Data, "a.b.c.d").unwrap();
        let prefix = &["a", "b", "c", "d"];
        key.strip_prefix_segments(prefix).unwrap_err();
    }

    #[test]
    fn append_segments_ok() {
        let key = Key::new(KeyType::Data, "a.b").unwrap();
        let new = key.append_segments(&["x"]).unwrap();
        assert_eq!(new.name(), "a.b.x");

        let new = key.append_segments(&["x.y"]).unwrap();
        assert_eq!(new.name(), "a.b.\"x.y\"");

        let new = key.append_segments(&["x", "y"]).unwrap();
        assert_eq!(new.name(), "a.b.x.y");
    }

    #[test]
    fn append_segments_err() {
        let key = Key::new(KeyType::Data, "a.b").unwrap();
        key.append_segments(&["@"]).unwrap_err();
    }

    #[test]
    fn append_key_ok() {
        let key = Key::new(KeyType::Data, "a.b").unwrap();
        let key2 = Key::new(KeyType::Data, "c.d").unwrap();
        let new = key.append_key(&key2).unwrap();
        assert_eq!(new.name(), "a.b.c.d");

        let key2 = Key::new(KeyType::Data, "\"c.d\"").unwrap();
        let new = key.append_key(&key2).unwrap();
        assert_eq!(new.name(), "a.b.\"c.d\"");
    }

    #[test]
    fn append_key_err() {
        let long_key = Key::new(KeyType::Data, "a".repeat(MAX_KEY_NAME_LENGTH)).unwrap();
        let key2 = Key::new(KeyType::Data, "b").unwrap();
        long_key.append_key(&key2).unwrap_err();
    }

    #[test]
    fn starts_with_segments() {
        let key = Key::new(KeyType::Data, "a.b").unwrap();
        assert!(key.starts_with_segments(&["a"]));
        assert!(!key.starts_with_segments(&["\"a.b\""]));
        assert!(!key.starts_with_segments(&["a."]));
    }
}
