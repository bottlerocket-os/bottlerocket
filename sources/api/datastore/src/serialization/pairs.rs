//! The goal of this module is to be able to turn serializable structures, primarily the API model,
//! into a form that can be easily written to our data store, key by key, since we will often
//! receive arbitrary subsets of the valid keys.  We use serde to help walk through the structure,
//! and use the Serializer's associated types to keep track of where we are in the tree of nested
//! structures.

//! The serialization pattern below could be used for other structures as well, but we're starting
//! out by orienting it toward the API model.  As such, data types are oriented around TOML/JSON
//! types, to be sure we support the various forms of input/output we care about.

use log::trace;
use serde::{ser, Serialize};
use snafu::{IntoError, NoneError as NoSource, OptionExt, ResultExt};
use std::collections::HashMap;

use super::{error, Error, MapKeySerializer, Result};
use crate::{serialize_scalar, Key, KeyType, ScalarError};

/// This is the primary interface to our serialization.  We turn anything implementing Serialize
/// into pairs of datastore keys and serialized values.  For example, a nested struct like this:
///    Settings -> DockerSettings -> bridge_ip = u64
/// would turn into a key of "settings.docker-settings.bridge-ip" and a serialized String
/// representing the u64 data.
pub fn to_pairs<T: Serialize>(value: &T) -> Result<HashMap<Key, String>> {
    let mut output = HashMap::new();
    let serializer = Serializer::new(&mut output, None);
    value.serialize(serializer)?;
    Ok(output)
}

/// Like to_pairs, but lets you add an arbitrary prefix to the resulting keys.  A separator will
/// automatically be added after the prefix.
pub fn to_pairs_with_prefix<S, T>(prefix: S, value: &T) -> Result<HashMap<Key, String>>
where
    S: AsRef<str>,
    T: Serialize,
{
    let prefix = prefix.as_ref();
    let prefix_key = Key::new(KeyType::Data, prefix).map_err(|e| {
        error::InvalidKeySnafu {
            msg: format!("Prefix '{}' not valid as Key: {}", prefix, e),
        }
        .into_error(NoSource)
    })?;

    let mut output = HashMap::new();
    let serializer = Serializer::new(&mut output, Some(prefix_key));
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
/// It's more common to use a HashMap in the model, and then to use named keys instead of indexes,
/// which works fine.)
struct Serializer<'a> {
    output: &'a mut HashMap<Key, String>,
    prefix: Option<Key>,
    // This is temporary storage for serializing maps, because serde gives us keys and values
    // separately.  See the SerializeMap implementation below.
    key: Option<Key>,
}

impl<'a> Serializer<'a> {
    fn new(output: &'a mut HashMap<Key, String>, prefix: Option<Key>) -> Self {
        Self {
            output,
            prefix,
            key: None,
        }
    }
}

/// This helps us handle the cases where we have to have an existing prefix in order to output a
/// value.  It creates an explanatory error if the given prefix is None.
fn expect_prefix(maybe_prefix: Option<Key>, value: &str) -> Result<Key> {
    maybe_prefix.context(error::MissingPrefixSnafu { value })
}

/// Serializes a concrete value and saves it to the output, assuming we have a prefix.
macro_rules! concrete_output {
    ($self:expr, $value:expr) => {
        trace!("Serializing scalar at prefix {:?}", $self.prefix);
        let value = serialize_scalar::<_, ScalarError>(&$value).with_context(|_| {
            error::SerializationSnafu {
                given: format!("concrete value '{}'", $value),
            }
        })?;
        let prefix = expect_prefix($self.prefix, &value)?;
        $self.output.insert(prefix, value);
        return Ok(());
    };
}

/// Several types are invalid for our serialization so we commonly need to return an error.  This
/// simplifies the creation of that error, with a customizable message for the type.
fn bad_type<T>(typename: &str) -> Result<T> {
    error::InvalidTypeSnafu { typename }.fail()
}

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
    fn serialize_bool(self, v: bool) -> Result<()> {
        concrete_output!(self, v);
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        concrete_output!(self, v);
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        concrete_output!(self, v);
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        concrete_output!(self, v);
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        concrete_output!(self, v);
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        concrete_output!(self, v);
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        concrete_output!(self, v);
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        concrete_output!(self, v);
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        concrete_output!(self, v);
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        concrete_output!(self, v);
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        concrete_output!(self, v);
    }

    // Don't serialize None at all; it should mean the key wasn't given.
    fn serialize_none(self) -> Result<()> {
        Ok(())
    }

    // Serialize the Some(x) as x.  Our basic structure is that all settings are optional, so
    // the API is ergonomic to call with a subset of keys, and so Some just means they wanted this
    // key set.
    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // Compound types
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(FlatSerializer::new(
            self.output,
            expect_prefix(self.prefix, "seq")?,
        ))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(Serializer::new(self.output, self.prefix))
    }

    fn serialize_struct(self, name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        trace!("Serializing struct '{}' at prefix {:?}", name, self.prefix);
        // If we already have a prefix, use it - could be because we're in a nested struct, or the
        // user gave a prefix.  Otherwise, use the given name - this is a top-level struct.
        let prefix = match self.prefix {
            p @ Some(_) => p,
            None => {
                trace!("Had no prefix, starting with struct name: {}", name);
                let key = Key::from_segments(KeyType::Data, &[&name]).map_err(|e| {
                    error::InvalidKeySnafu {
                        msg: format!("struct '{}' not valid as Key: {}", name, e),
                    }
                    .into_error(NoSource)
                })?;
                Some(key)
            }
        };
        Ok(Serializer::new(self.output, prefix))
    }

    // Types we can't (or don't want to) represent.
    // Can't fit u64 into signed 64-bit range.
    fn serialize_u64(self, _v: u64) -> Result<()> {
        bad_type("u64")
    }

    // No char type, and using String would lose the distinction you were trying to make by
    // using a char.
    fn serialize_char(self, _v: char) -> Result<()> {
        bad_type("char")
    }

    // No binary type; could use base64 or similar if we implement our own deserialization
    // that understands it.
    fn serialize_bytes(self, _v: &[u8]) -> Result<()> {
        bad_type("bytes")
    }

    // We just don't expect to need these, and we doesn't have a great way to represent them.
    fn serialize_unit(self) -> Result<()> {
        bad_type("unit")
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        bad_type("unit struct")
    }

    // When we use "simple" enums (those that only have "unit" variants), we represent them as
    // strings in the data model. As far as the API is concerned, these are string values, but in
    // the model we constrain them using an enum.
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        bad_type("newtype struct")
    }
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        bad_type("newtype variant")
    }

    // We don't expect to need tuples, and we don't have a great way to represent them,
    // distinct from lists.
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        bad_type("tuple")
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        bad_type("tuple struct")
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        bad_type("tuple variant")
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        bad_type("struct variant")
    }
}

/// Helper that combines the existing prefix, if any, with a separator and the new key.
fn key_append_or_create(old_prefix: &Option<Key>, key: &Key) -> Result<Key> {
    if let Some(old_prefix) = old_prefix {
        old_prefix.append_key(key).map_err(|e| {
            error::InvalidKeySnafu {
                msg: format!(
                    "appending '{}' to '{}' is invalid as Key: {}",
                    key, old_prefix, e
                ),
            }
            .into_error(NoSource)
        })
    } else {
        Ok(key.clone())
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
        trace!("Serializing map key at prefix {:?}", self.prefix);
        // We're given a serializable thing; need to serialize it to get a string we can work with.
        let key_str = key.serialize(&MapKeySerializer::new())?;
        // It should be valid as a Key.
        // Note: we use 'new', not 'from_segments', because we just serialized into a string,
        // meaning it's in quoted form.
        let key = Key::new(KeyType::Data, &key_str).map_err(|e| {
            error::InvalidKeySnafu {
                msg: format!("serialized map key '{}' not valid as Key: {}", &key_str, e),
            }
            .into_error(NoSource)
        })?;
        // Store the key to use later in serialize_value.
        self.key = Some(key_append_or_create(&self.prefix, &key)?);
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
            None => error::InternalSnafu {
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

    fn serialize_field<T>(&mut self, key_str: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let key = Key::from_segments(KeyType::Data, &[&key_str]).map_err(|e| {
            error::InvalidKeySnafu {
                msg: format!("struct field '{}' not valid as Key: {}", key_str, e),
            }
            .into_error(NoSource)
        })?;

        let new_root = key_append_or_create(&self.prefix, &key)?;
        trace!(
            "Recursively serializing struct with new root '{}' from prefix '{:?}' and key '{}'",
            new_root,
            self.prefix,
            &key
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
    output: &'a mut HashMap<Key, String>,
    prefix: Key,
    list: Vec<String>,
}

impl<'a> FlatSerializer<'a> {
    fn new(output: &'a mut HashMap<Key, String>, prefix: Key) -> Self {
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
        self.list.push(
            serde_json::to_string(value).context(error::SerializationSnafu {
                given: "list element",
            })?,
        );
        Ok(())
    }

    fn end(self) -> Result<()> {
        let mut originals: Vec<serde_json::Value> = Vec::new();
        trace!("Deserializing elements of list");
        for original in self.list {
            originals.push(original.parse().context(error::DeserializationSnafu {
                given: "list element",
            })?);
        }

        trace!("Serializing list");
        self.output.insert(
            self.prefix,
            serde_json::to_string(&originals)
                .context(error::SerializationSnafu { given: "list" })?,
        );

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::{to_pairs, to_pairs_with_prefix};
    use crate::{Key, KeyType};
    use maplit::hashmap;
    use serde::Serialize;

    // Helper macro for making a data Key for testing whose name we know is valid.
    macro_rules! key {
        ($name:expr) => {
            Key::new(KeyType::Data, $name).unwrap()
        };
    }

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
                key!("B.list") => "[3,4,5]".to_string(),
                key!("B.boolean") => "true".to_string(),
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
                key!("A.b.list") => "[5,6,7]".to_string(),
                key!("A.b.boolean") => "true".to_string(),
                key!("A.id") => "42".to_string(),
            )
        );
    }

    #[test]
    fn map() {
        let m = hashmap!(
            key!("A") => hashmap!(
                key!("id") => 42,
                key!("ie") => 43,
            ),
        );
        let keys = to_pairs_with_prefix("map", &m).unwrap();
        assert_eq!(
            keys,
            hashmap!(
                key!("map.A.id") => "42".to_string(),
                key!("map.A.ie") => "43".to_string(),
            )
        );
    }

    #[test]
    fn map_no_root() {
        let m = hashmap!(
            key!("A") => hashmap!(
                key!("id") => 42,
                key!("ie") => 43,
            ),
        );
        let keys = to_pairs(&m).unwrap();
        assert_eq!(
            keys,
            hashmap!(
                key!("A.id") => "42".to_string(),
                key!("A.ie") => "43".to_string(),
            )
        );
    }

    #[test]
    fn concrete_fails() {
        let i = 42;
        to_pairs(&i).unwrap_err();
    }

    #[test]
    fn string_values() {
        let m = hashmap!(
            key!("A") => hashmap!(
                key!("id") => "apples",
                key!("ie") => "oranges",
            ),
        );
        let keys = to_pairs(&m).unwrap();
        assert_eq!(
            keys,
            hashmap!(
                key!("A.id") => "\"apples\"".to_string(),
                key!("A.ie") => "\"oranges\"".to_string(),
            )
        );
    }

    #[derive(Serialize)]
    #[serde(rename_all = "kebab-case")]
    enum TestEnum {
        Alpha,
        Beta,
    }

    #[test]
    fn enum_values() {
        let m = hashmap!(
            key!("A") => hashmap!(
                key!("id") => TestEnum::Alpha,
                key!("ie") => TestEnum::Beta,
            ),
        );
        let keys = to_pairs(&m).unwrap();
        assert_eq!(
            keys,
            hashmap!(
                key!("A.id") => "\"alpha\"".to_string(),
                key!("A.ie") => "\"beta\"".to_string(),
            )
        );
    }
}
