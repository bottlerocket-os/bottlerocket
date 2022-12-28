//! The serialization module implements generic serialization techniques that are particularly
//! useful for turning Rust structures into simpler types that are easy to write to a datastore.

mod error;
mod pairs;

pub use error::{Error, Result};
pub use pairs::{to_pairs, to_pairs_with_prefix};

use log::{debug, trace};
use serde::{ser, Serialize};
use snafu::{IntoError, NoneError as NoSource};

use crate::{Key, KeyType};

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
fn bad_key<T>(typename: &str) -> Result<T> {
    error::BadMapKeySnafu { typename }.fail()
}

impl ser::Serializer for &MapKeySerializer {
    type Ok = String;
    type Error = Error;

    type SerializeSeq = ser::Impossible<String, Error>;
    type SerializeTuple = ser::Impossible<String, Error>;
    type SerializeTupleStruct = ser::Impossible<String, Error>;
    type SerializeTupleVariant = ser::Impossible<String, Error>;
    type SerializeMap = ser::Impossible<String, Error>;
    type SerializeStruct = ser::Impossible<String, Error>;
    type SerializeStructVariant = ser::Impossible<String, Error>;

    // Allow serialization of strings for map keys, but nothing else.

    fn serialize_str(self, value: &str) -> Result<String> {
        // Make sure string is valid as a key.
        let key = Key::from_segments(KeyType::Data, &[value]).map_err(|e| {
            debug!("MapKeySerializer got invalid key name: {}", value);
            error::InvalidKeySnafu {
                msg: format!("{}", e),
            }
            .into_error(NoSource)
        })?;
        trace!("MapKeySerializer got OK key: {}", key);
        Ok(key.to_string())
    }

    fn serialize_bool(self, _value: bool) -> Result<String> {
        bad_key("bool")
    }

    fn serialize_i8(self, _value: i8) -> Result<String> {
        bad_key("i8")
    }

    fn serialize_i16(self, _value: i16) -> Result<String> {
        bad_key("i16")
    }

    fn serialize_i32(self, _value: i32) -> Result<String> {
        bad_key("i32")
    }

    fn serialize_i64(self, _value: i64) -> Result<String> {
        bad_key("i64")
    }

    fn serialize_u8(self, _value: u8) -> Result<String> {
        bad_key("u8")
    }

    fn serialize_u16(self, _value: u16) -> Result<String> {
        bad_key("u16")
    }

    fn serialize_u32(self, _value: u32) -> Result<String> {
        bad_key("u32")
    }

    fn serialize_u64(self, _value: u64) -> Result<String> {
        bad_key("u64")
    }

    fn serialize_f32(self, _value: f32) -> Result<String> {
        bad_key("f32")
    }

    fn serialize_f64(self, _value: f64) -> Result<String> {
        bad_key("f64")
    }

    fn serialize_char(self, _value: char) -> Result<String> {
        bad_key("char")
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<String> {
        bad_key("bytes")
    }

    fn serialize_unit(self) -> Result<String> {
        bad_key("unit")
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<String> {
        bad_key("unit_struct")
    }

    /// A simple enum can be used as if it were a string, so we allow these to serve as map keys.
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<String> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, _value: &T) -> Result<String>
    where
        T: Serialize,
    {
        bad_key("newtype_struct")
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<String>
    where
        T: Serialize,
    {
        bad_key("newtype_variant")
    }

    fn serialize_none(self) -> Result<String> {
        bad_key("none")
    }

    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<String>
    where
        T: Serialize,
    {
        bad_key("some")
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        bad_key("seq")
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        bad_key("tuple")
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        bad_key("tuple struct")
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        bad_key("tuple variant")
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        bad_key("map")
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        bad_key("struct")
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        bad_key("struct variant")
    }
}

#[cfg(test)]
mod test {
    use super::MapKeySerializer;
    use serde::Serialize;

    // This enum is fine because its variants are "simple", thus it can be represented as a simple
    // string and can be used as a map key.
    #[derive(Debug, Serialize)]
    enum TestEnum {
        Value,
    }

    // This enum cannot be used as a map key because it has a variant that can't be serialized as a
    // simple string.
    #[derive(Debug, Serialize)]
    enum BadEnum {
        Value(i32),
    }

    #[test]
    fn ok_key() {
        let serializer = MapKeySerializer::new();
        let m = "A".to_string();
        let res = m.serialize(&serializer).unwrap();
        assert_eq!(res, "A");
    }

    #[test]
    fn ok_enum_key() {
        let serializer = MapKeySerializer::new();
        let m = TestEnum::Value;
        let res = m.serialize(&serializer).unwrap();
        assert_eq!(res, "Value");
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
        BadEnum::Value(1).serialize(&serializer).unwrap_err();
    }
}
