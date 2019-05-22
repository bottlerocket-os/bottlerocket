//! The serialization module implements generic serialization techniques that are particularly
//! useful for turning Rust structures into simpler types that are easy to write to a datastore.

mod pairs;

pub use pairs::{to_pairs, to_pairs_with_prefix};

use serde::{ser, Serialize};

use crate::datastore::{Key, KeyType};

/// Potential errors from serialization.
#[derive(Debug, Error)]
pub enum SerializationError {
    // This error variant is required to implement ser::Error for serde.
    #[error(msg_embedded, no_from, non_std)]
    /// Error during serialization
    Message(String),

    #[error(msg_embedded, no_from, non_std)]
    /// Serialization invariant violation
    Internal(String),

    #[error(msg_embedded, no_from, non_std)]
    /// Error creating valid datastore key
    InvalidKey(String),

    #[error(msg_embedded, no_from, non_std)]
    /// Type of given value cannot be serialized
    InvalidValue(String),

    /// Error serializing scalar value
    Json(serde_json::error::Error),
}

type Result<T> = std::result::Result<T, SerializationError>;

impl ser::Error for SerializationError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        SerializationError::Message(msg.to_string())
    }
}

// Below are serializers not specific to the pairs module that could be used for other serializers.
// For example, a 'keys' serializer that just returns a set of keys, without associated data.

/// We use this in serialize_key to ensure map keys are only strings, for easy interoperability
/// with TOML/JSON.
///
/// We also ensure map keys are valid datastore keys.
struct MapKeySerializer {}

impl MapKeySerializer {
    fn new() -> Self {
        Self {}
    }
}

/// Most types are invalid map keys (only strings are OK) so we need to return an error in most
/// cases.  This simplifies the creation of that error, with a customizable message for the type.
fn bad_key(msg: &str) -> Result<String> {
    Err(SerializationError::InvalidValue(
        msg.to_string() + "s not allowed as map key",
    ))
}

#[rustfmt::skip]
impl ser::Serializer for &MapKeySerializer {
    type Ok = String;
    type Error = SerializationError;

    type SerializeSeq = ser::Impossible<String, SerializationError>;
    type SerializeTuple = ser::Impossible<String, SerializationError>;
    type SerializeTupleStruct = ser::Impossible<String, SerializationError>;
    type SerializeTupleVariant = ser::Impossible<String, SerializationError>;
    type SerializeMap = ser::Impossible<String, SerializationError>;
    type SerializeStruct = ser::Impossible<String, SerializationError>;
    type SerializeStructVariant = ser::Impossible<String, SerializationError>;

    // Allow serialization of strings for map keys, but nothing else.

    fn serialize_str(self, value: &str) -> Result<String> {
        // Make sure string is valid as a key.
        // Note: we check as a metadata key here, because metadata keys are simpler - no dotted
        // components, just one.  We wouldn't want dotted components in a single map key because
        // it would falsely imply nesting.
        let key = Key::new(KeyType::Meta, value)
            .map_err(|e| {
                debug!("MapKeySerializer got invalid key name: {}", value);
                SerializationError::InvalidKey(format!("Invalid datastore key: {}", e))
            })?;
        trace!("MapKeySerializer got OK key: {}", key);
        Ok(key.to_string())
    }

    fn serialize_bool(self, _value: bool) -> Result<String> { bad_key("bool") }
    fn serialize_i8(self, _value: i8) -> Result<String> { bad_key("i8") }
    fn serialize_i16(self, _value: i16) -> Result<String> { bad_key("i16") }
    fn serialize_i32(self, _value: i32) -> Result<String> { bad_key("i32") }
    fn serialize_i64(self, _value: i64) -> Result<String> { bad_key("i64") }
    fn serialize_u8(self, _value: u8) -> Result<String> { bad_key("u8") }
    fn serialize_u16(self, _value: u16) -> Result<String> { bad_key("u16") }
    fn serialize_u32(self, _value: u32) -> Result<String> { bad_key("u32") }
    fn serialize_u64(self, _value: u64) -> Result<String> { bad_key("u64") }
    fn serialize_f32(self, _value: f32) -> Result<String> { bad_key("f32") }
    fn serialize_f64(self, _value: f64) -> Result<String> { bad_key("f64") }
    fn serialize_char(self, _value: char) -> Result<String> { bad_key("char") }
    fn serialize_bytes(self, _value: &[u8]) -> Result<String> { bad_key("bytes") }
    fn serialize_unit(self) -> Result<String> { bad_key("unit") }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<String> { bad_key("unit_struct") }
    fn serialize_unit_variant( self, _name: &'static str, _variant_index: u32, _variant: &'static str) -> Result<String> { bad_key("unit_variant") }
    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, _value: &T) -> Result<String> where T: Serialize { bad_key("newtype_struct") }
    fn serialize_newtype_variant<T: ?Sized>( self, _name: &'static str, _variant_index: u32, _variant: &'static str, _value: &T) -> Result<String> where T: Serialize { bad_key("newtype_variant") }
    fn serialize_none(self) -> Result<String> { bad_key("none") }
    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<String> where T: Serialize { bad_key("some") }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(SerializationError::InvalidValue("seqs not allowed as map key".to_string()))
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(SerializationError::InvalidValue("tuples not allowed as map key".to_string()))
    }
    fn serialize_tuple_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeTupleStruct> {
        Err(SerializationError::InvalidValue("tuple structs not allowed as map key".to_string()))
    }
    fn serialize_tuple_variant( self, _name: &'static str, _variant_index: u32, _variant: &'static str, _len: usize) -> Result<Self::SerializeTupleVariant> {
        Err(SerializationError::InvalidValue("tuple variants not allowed as map key".to_string()))
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(SerializationError::InvalidValue("maps not allowed as map key".to_string()))
    }
    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(SerializationError::InvalidValue("structs not allowed as map key".to_string()))
    }
    fn serialize_struct_variant( self, _name: &'static str, _variant_index: u32, _variant: &'static str, _len: usize) -> Result<Self::SerializeStructVariant> {
        Err(SerializationError::InvalidValue("struct variants not allowed as map key".to_string()))
    }
}

#[cfg(test)]
mod test {
    use super::MapKeySerializer;
    use serde::Serialize;

    #[test]
    fn ok_key() {
        let serializer = MapKeySerializer::new();
        let m = "A".to_string();
        let res = m.serialize(&serializer).unwrap();
        assert_eq!(res, "A");
    }

    #[test]
    fn bad_keys() {
        let serializer = MapKeySerializer::new();
        42u8.serialize(&serializer).unwrap_err();
        42i32.serialize(&serializer).unwrap_err();
        true.serialize(&serializer).unwrap_err();
        'q'.serialize(&serializer).unwrap_err();
        ().serialize(&serializer).unwrap_err();
        [1u8].serialize(&serializer).unwrap_err();
        (None as Option<u8>).serialize(&serializer).unwrap_err();
        Some(42).serialize(&serializer).unwrap_err();
    }
}
