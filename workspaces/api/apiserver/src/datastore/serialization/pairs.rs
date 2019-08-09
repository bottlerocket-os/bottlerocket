//! The goal of this module is to be able to turn the settings in 'model' into a form that can be
//! easily written to our data store, key by key, since we will often receive arbitrary subsets of
//! the valid keys.  We use serde to help walk through the structure, and use the Serializer's
//! associated types to keep track of where we are in the tree of nested structures.

//! The serialization pattern below could be used for other structures as well, but we're starting
//! out by orienting it toward settings.  As such, data types are oriented around TOML/JSON types,
//! to be sure we support the various forms of input/output we care about.

use serde::{ser, Serialize};
use snafu::{OptionExt, ResultExt};
use std::collections::HashMap;

use super::{error, Error, MapKeySerializer, Result};
use crate::datastore::{serialize_scalar, ScalarError, KEY_SEPARATOR};

/// This is the primary interface to our serialization.  We turn anything implementing Serialize
/// into pairs of datastore keys and serialized values.  For example, a nested struct like this:
///    Settings -> DockerSettings -> bridge_ip = u64
/// would turn into a key of "settings.docker-settings.bridge-ip" and a serialized String
/// representing the u64 data.
pub fn to_pairs<T: Serialize>(value: &T) -> Result<HashMap<String, String>> {
    let mut output = HashMap::new();
    let serializer = Serializer::new(&mut output, None);
    value.serialize(serializer)?;
    Ok(output)
}

/// Like to_pairs, but lets you add an arbitrary prefix to the resulting keys.  A separator will
/// automatically be added after the prefix.
pub fn to_pairs_with_prefix<T: Serialize>(
    prefix: String,
    value: &T,
) -> Result<HashMap<String, String>> {
    let mut output = HashMap::new();
    let serializer = Serializer::new(&mut output, Some(prefix));
    value.serialize(serializer)?;
    Ok(output)
}

/////

/// Serializer does most of the work by recursively serializing compound structures, and trivially
/// serializing scalars.
///
/// Caveat: for a list/tuple, the elements inside only have indexes, which doesn't work well with
/// the data store.  Lists are common enough that we need some answer, so we say that lists can
/// only contain scalars, not further compound objects.  That way we can serialize the list
/// directly (see FlatSerializer) rather than as a compound.
///
/// (We could handle lists as proper compound structures by improving the data store such that it
/// can store unnamed sub-components, perhaps by using a visible index ("a.b.c[0]", "a.b.c[1]").
struct Serializer<'a> {
    output: &'a mut HashMap<String, String>,
    prefix: Option<String>,
    // This is temporary storage for serializing maps, because serde gives us keys and values
    // separately.  See the SerializeMap implementation below.
    key: Option<String>,
}

impl<'a> Serializer<'a> {
    fn new(output: &'a mut HashMap<String, String>, prefix: Option<String>) -> Self {
        Self {
            output,
            prefix,
            key: None,
        }
    }
}

/// This helps us handle the cases where we have to have an existing prefix in order to output a
/// value.  It creates an explanatory error if the given prefix is None.
fn expect_prefix(maybe_prefix: Option<String>, value: &str) -> Result<String> {
    maybe_prefix.context(error::MissingPrefix { value })
}

/// Serializes a concrete value and saves it to the output, assuming we have a prefix.
macro_rules! concrete_output {
    ($self:expr, $value:expr) => {
        trace!("Serializing scalar at prefix {:?}", $self.prefix);
        let value =
            serialize_scalar::<_, ScalarError>(&$value).with_context(|| error::Serialization {
                given: format!("concrete value '{}'", $value),
            })?;
        let prefix = expect_prefix($self.prefix, &value)?;
        $self.output.insert(prefix, value);
        return Ok(());
    };
}

/// Several types are invalid for our serialization so we commonly need to return an error.  This
/// simplifies the creation of that error, with a customizable message for the type.
fn bad_type<T>(typename: &str) -> Result<T> {
    error::InvalidType { typename }.fail()
}

#[rustfmt::skip]
impl<'a> ser::Serializer for Serializer<'a> {
    type Ok = ();
    type Error = Error;

    // See the docs on Serializer for reasoning about this.
    type SerializeSeq = FlatSerializer<'a>;
    type SerializeTuple = ser::Impossible<(), Error>;
    type SerializeTupleStruct = ser::Impossible<(), Error>;
    type SerializeTupleVariant = ser::Impossible<(), Error>;
    type SerializeStructVariant = ser::Impossible<(), Error>;
    type SerializeMap = Self;
    type SerializeStruct = Self;

    // Simple concrete types.
    fn serialize_bool(self, v: bool) -> Result<()> { concrete_output!(self, v); }
    fn serialize_i8(self, v: i8) -> Result<()> { concrete_output!(self, v); }
    fn serialize_i16(self, v: i16) -> Result<()> { concrete_output!(self, v); }
    fn serialize_i32(self, v: i32) -> Result<()> { concrete_output!(self, v); }
    fn serialize_i64(self, v: i64) -> Result<()> { concrete_output!(self, v); }
    fn serialize_u8(self, v: u8) -> Result<()> { concrete_output!(self, v); }
    fn serialize_u16(self, v: u16) -> Result<()> { concrete_output!(self, v); }
    fn serialize_u32(self, v: u32) -> Result<()> { concrete_output!(self, v); }
    fn serialize_f32(self, v: f32) -> Result<()> { concrete_output!(self, v); }
    fn serialize_f64(self, v: f64) -> Result<()> { concrete_output!(self, v); }
    fn serialize_str(self, v: &str) -> Result<()> { concrete_output!(self, v); }

    // Don't serialize None at all; it should mean the key wasn't given.
    fn serialize_none(self) -> Result<()> { Ok(()) }
    // Serialize the Some(x) as x.  Our basic structure is that all settings are optional, so
    // the API is ergonomic to call with a subset of keys, and so Some just means they wanted this
    // key set.
    fn serialize_some<T>(self, value: &T) -> Result<()> where T: ?Sized + Serialize { value.serialize(self) }

    // Compound types
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(FlatSerializer::new(self.output, expect_prefix(self.prefix, "seq")?))
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(Serializer::new(self.output, self.prefix))
    }
    fn serialize_struct(self, name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        trace!("Serializing struct '{}' at prefix {:?}", name, self.prefix);
        // If we already have a prefix, use it - could be because we're in a nested struct, or the
        // user gave a prefix.  Otherwise, use the given name - this is a top-level struct.
        let prefix = self.prefix.or_else(|| {
            trace!("Had no prefix, starting with struct name: {}", name);
            Some(name.to_string())
        });
        Ok(Serializer::new(self.output, prefix))
    }

    // Types we can't (or don't want to) represent.
    // Can't fit u64 into signed 64-bit range.
    fn serialize_u64(self, _v: u64) -> Result<()> { bad_type("u64") }
    // No char type, and using String would lose the distinction you were trying to make by
    // using a char.
    fn serialize_char(self, _v: char) -> Result<()> { bad_type("char") }
    // No binary type; could use base64 or similar if we implement our own deserialization
    // that understands it.
    fn serialize_bytes(self, _v: &[u8]) -> Result<()> { bad_type("bytes") }
    // We just don't expect to need these, and we doesn't have a great way to represent them.
    fn serialize_unit(self) -> Result<()> { bad_type("unit") }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> { bad_type("unit struct") }
    fn serialize_unit_variant(self, _name: &'static str, _variant_index: u32, _variant: &'static str) -> Result<()> {
        bad_type("unit variant")
    }
    fn serialize_newtype_struct<T>(self, _name: &'static str, _value: &T) -> Result<()> where T: ?Sized + Serialize {
        bad_type("newtype struct")
    }
    fn serialize_newtype_variant<T>(self, _name: &'static str, _variant_index: u32, _variant: &'static str, _value: &T) -> Result<()> where T: ?Sized + Serialize {
        bad_type("newtype variant")
    }

    // We don't expect to need tuples, and we don't have a great way to represent them,
    // distinct from lists.
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        bad_type("tuple")
    }
    fn serialize_tuple_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeTupleStruct> {
        bad_type("tuple struct")
    }
    fn serialize_tuple_variant(self, _name: &'static str, _variant_index: u32, _variant: &'static str, _len: usize) -> Result<Self::SerializeTupleVariant> {
        bad_type("tuple variant")
    }
    fn serialize_struct_variant(self, _name: &'static str, _variant_index: u32, _variant: &'static str, _len: usize) -> Result<Self::SerializeStructVariant> {
        bad_type("struct variant")
    }

}

/// Helper that combines the existing prefix, if any, with a separator and the new key.
fn dotted_prefix(old_prefix: Option<String>, key: String) -> String {
    if let Some(old_prefix) = old_prefix {
        old_prefix + KEY_SEPARATOR + &key
    } else {
        key
    }
}

/// Serialize map structures, recursively handling any inner compound structures by using the key
/// name as the new prefix.
///
/// Two important notes here.
///
/// First, we can only allow map keys to be strings, for easy interoperability with TOML/JSON.
/// We delegate to the MapKeySerializer to handle that.
///
/// Second, serde is limited in the sense that it requires you to serialize keys and values
/// separately, whereas we'd prefer a single pass because we only need to store the output.  To
/// work around this, we use the Option 'key' in the struct to store the last-serialized key,
/// knowing that serde will serialize keys and values in that order.
impl<'a> ser::SerializeMap for Serializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        // Store the key to use later in serialize_value.
        trace!("Serializing map key at prefix {:?}", self.prefix);
        let key_str = key.serialize(&MapKeySerializer::new())?;
        self.key = Some(dotted_prefix(self.prefix.clone(), key_str));
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        // Pull out the stored key and serialize whatever's in the value using the key as its name
        // prefix.
        match self.key.take() {
            Some(key) => {
                trace!(
                    "Recursively serializing map value at prefix {:?}",
                    self.prefix
                );
                value.serialize(Serializer::new(self.output, Some(key)))
            }
            None => error::Internal {
                msg: "Attempted to serialize value without key",
            }
            .fail(),
        }
    }

    // No need to "end" the structure, we're not serializing to a single text format.
    fn end(self) -> Result<()> {
        Ok(())
    }
}

/// Serialize structs, recursively handling any inner compound structures by using the key name as
/// the new prefix.  (No need to use the struct's name; we're not at the root level, so it was
/// already pointed to by some name.)
impl<'a> ser::SerializeStruct for Serializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let new_root = dotted_prefix(self.prefix.clone(), key.to_string());
        trace!(
            "Recursively serializing struct with new root '{}' from prefix '{:?}' and key '{}'",
            new_root,
            self.prefix,
            key
        );
        value.serialize(Serializer::new(self.output, Some(new_root)))
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

/////

/// This serializes compound structures into a flat blob, for cases where recursively serializing
/// compound structures doesn't make sense.  See Serializer for detail on why it uses this.
///
/// Warning; this requires hacks.  serde gives you three callbacks during serialization - starting
/// the structure, for each element, and ending the structure.  There's no option to handle an
/// entire structure at once, which is exactly what we'd want.  We can't clone the elements, since
/// we only have a Serialize bound, and I couldn't figure out how to store the references, so we do
/// the unthinkable - serialize each element to a String, store those in a list during the
/// serialization steps, and then at the end, deserialize the strings back into a list of the
/// original type, and serialize the entire list.  Sorry.
struct FlatSerializer<'a> {
    output: &'a mut HashMap<String, String>,
    prefix: String,
    list: Vec<String>,
}

impl<'a> FlatSerializer<'a> {
    fn new(output: &'a mut HashMap<String, String>, prefix: String) -> Self {
        FlatSerializer {
            output,
            prefix,
            list: Vec::new(),
        }
    }
}

impl<'a> ser::SerializeSeq for FlatSerializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        trace!("Serializing element of list");
        self.list
            .push(serde_json::to_string(value).context(error::Serialization {
                given: "list element",
            })?);
        Ok(())
    }

    fn end(self) -> Result<()> {
        let mut originals: Vec<serde_json::Value> = Vec::new();
        trace!("Deserializing elements of list");
        for original in self.list {
            originals.push(original.parse().context(error::Deserialization {
                given: "list element",
            })?);
        }

        trace!("Serializing list");
        self.output.insert(
            self.prefix,
            serde_json::to_string(&originals).context(error::Serialization { given: "list" })?,
        );

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::{to_pairs, to_pairs_with_prefix};
    use maplit::hashmap;
    use serde::Serialize;

    #[derive(PartialEq, Serialize)]
    struct A {
        id: u8,
        b: Option<B>,
    }

    #[derive(PartialEq, Serialize)]
    struct B {
        list: Vec<u8>,
        boolean: bool,
    }

    #[test]
    fn basic_struct_keys() {
        let b = B {
            list: vec![3, 4, 5],
            boolean: true,
        };
        let keys = to_pairs(&b).unwrap();
        assert_eq!(
            keys,
            hashmap!(
                "B.list".to_string() => "[3,4,5]".to_string(),
                "B.boolean".to_string() => "true".to_string(),
            )
        );
    }

    #[test]
    fn empty_value() {
        let val: toml::Value = toml::from_str("").unwrap();
        let keys = to_pairs(&val).unwrap();
        assert_eq!(keys, hashmap!())
    }

    #[test]
    fn nested_struct_keys() {
        let b = B {
            list: vec![5, 6, 7],
            boolean: true,
        };
        let a = A { id: 42, b: Some(b) };
        let keys = to_pairs(&a).unwrap();
        assert_eq!(
            keys,
            hashmap!(
                "A.b.list".to_string() => "[5,6,7]".to_string(),
                "A.b.boolean".to_string() => "true".to_string(),
                "A.id".to_string() => "42".to_string(),
            )
        );
    }

    #[test]
    fn map() {
        let m = hashmap!(
            "A".to_string() => hashmap!(
                "id".to_string() => 42,
                "ie".to_string() => 43,
            ),
        );
        let keys = to_pairs_with_prefix("map".to_string(), &m).unwrap();
        assert_eq!(
            keys,
            hashmap!(
                "map.A.id".to_string() => "42".to_string(),
                "map.A.ie".to_string() => "43".to_string(),
            )
        );
    }

    #[test]
    fn map_no_root() {
        let m = hashmap!(
            "A".to_string() => hashmap!(
                "id".to_string() => 42,
                "ie".to_string() => 43,
            ),
        );
        let keys = to_pairs(&m).unwrap();
        assert_eq!(
            keys,
            hashmap!(
                "A.id".to_string() => "42".to_string(),
                "A.ie".to_string() => "43".to_string(),
            )
        );
    }

    #[test]
    fn concrete_fails() {
        let i = 42;
        to_pairs(&i).unwrap_err();
    }
}
