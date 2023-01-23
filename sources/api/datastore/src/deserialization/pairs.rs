//! The goal of this module is to be able to turn a mapping of dotted keys -> values into a
//! populated structure.  The keys are of the form "a.b.c" and match up to nested structures
//! A { B { C } }.
//!
//! For example, with these structures:
//!    struct A {
//!        b: B,
//!    }
//!
//!    struct B {
//!        c: u64,
//!        d: u64,
//!    }
//!
//! An input map of {"a.b.c": 42, "a.b.d": 43} would return a populated structure:
//!    A {
//!      B {
//!        c: 42,
//!        d: 43,
//!      }
//!    }
//!
//! Note: serde deserialization is harder to understand than serialization, so this implementation
//! was kept as simple as possible rather than taking advantage of all of the structure that serde
//! provides.  forward_to_deserialize_any lets us omit most type-specific functions so we can
//! handle all scalars the same and all compound structures the same; see ValueDeserializer.
//!
//! The primary work is done by serde's MapDeserializer; it abstracts away the need to build the
//! visitor that serde expects.  It gives us the name of a field in a structure, and we have to
//! provide the value.  We use it recursively, and at each recursion, append a dot and the name of
//! the field to our "path" string.  In the example above, when we're looking at field "c", path
//! would be "a.b", so we know we should look for "a.b.c" in our input mapping.

use log::{error, trace};
use serde::de::{value::MapDeserializer, IntoDeserializer, Visitor};
use serde::{forward_to_deserialize_any, Deserialize};
use snafu::ResultExt;
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;

use super::{error, Error, Result};
use crate::{deserializer_for_scalar, Key, KeyType, ScalarDeserializer};

/// This is the primary interface to deserialization.  We turn the input map into the requested
/// output type, assuming all non-Option fields are provided, etc.
///
/// This only allows for deserialization into structs; to deserialize into maps, see
/// from_map_with_prefix.
///
/// The BuildHasher bound on the input HashMap lets you use a HashMap with any hashing
/// implementation.  This is just an implementation detail and not something you have to specify
/// about your input HashMap - any HashMap using string-like key/value types is fine.
pub fn from_map<'de, K, S, T, BH>(map: &'de HashMap<K, S, BH>) -> Result<T>
where
    K: Borrow<Key> + Eq + Hash,
    S: AsRef<str>,
    T: Deserialize<'de>,
    BH: std::hash::BuildHasher,
{
    let de = CompoundDeserializer::new(map, map.keys().map(|s| s.borrow().clone()).collect(), None);
    trace!("Deserializing keys: {:?}", de.keys);
    T::deserialize(de)
}

/// This is an alternate interface to deserialization that allows deserializing into maps.
///
/// To use this, you need to provide a string prefix, which represents the prefix of the map keys
/// that needs to be stripped away in order to match the map's expected fields.
///
/// For example, if you have `type Services = HashMap<String, Service>` and you have map keys like
/// "services.x.y.z", then you need to strip away the "services" component that represents the
/// map's "name", otherwise we'd think you have a "services" key in the map itself.  (The dot is
/// removed automatically, you don't need to specify it.)
///
/// This isn't necessary for structs because serde knows the struct's name, so we
/// can strip it automatically.
pub fn from_map_with_prefix<'de, K, S, T, BH>(
    prefix: Option<String>,
    map: &'de HashMap<K, S, BH>,
) -> Result<T>
where
    K: Borrow<Key> + Eq + Hash,
    S: AsRef<str>,
    T: Deserialize<'de>,
    BH: std::hash::BuildHasher,
{
    let key_prefix = match prefix {
        None => None,
        Some(ref p) => {
            Some(Key::new(KeyType::Data, p).context(error::InvalidPrefixSnafu { prefix: p })?)
        }
    };
    let de = CompoundDeserializer::new(
        map,
        map.keys().map(|s| s.borrow().clone()).collect(),
        key_prefix,
    );
    trace!(
        "Deserializing keys with prefix {:?}: {:?}",
        de.path,
        de.keys
    );
    T::deserialize(de)
}

/// ValueDeserializer is what interfaces with serde's MapDeserializer, which expects to receive a
/// key name and a deserializer for it on each iteration, i.e. for each field.  Based on whether
/// the key name has a dot, we know if we need to recurse again or just deserialize a final value,
/// which we represent as the two arms of the enum.
enum ValueDeserializer<'de, K, S, BH> {
    Scalar(ScalarDeserializer<'de>),
    Compound(CompoundDeserializer<'de, K, S, BH>),
}

impl<'de, K, S, BH> serde::de::Deserializer<'de> for ValueDeserializer<'de, K, S, BH>
where
    K: Borrow<Key> + Eq + Hash,
    S: AsRef<str>,
    BH: std::hash::BuildHasher,
{
    type Error = Error;

    /// Here we either pass off a scalar value to actually turn into a Rust data type, or
    /// recursively call our CompoundDeserializer to handle nested structure.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            ValueDeserializer::Scalar(mut scalar_deserializer) => {
                trace!("Handing off to scalar deserializer for deserialize_any");
                scalar_deserializer
                    .deserialize_any(visitor)
                    .context(error::DeserializeScalarSnafu)
            }
            ValueDeserializer::Compound(compound_deserializer) => {
                compound_deserializer.deserialize_map(visitor)
            }
        }
    }

    /// Here we deserialize values into Some(value) for any Option fields to represent that
    /// yes, we do indeed have the data.
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            ValueDeserializer::Scalar(mut scalar_deserializer) => {
                trace!("Handing off to scalar deserializer for deserialize_option");
                scalar_deserializer
                    .deserialize_option(visitor)
                    .context(error::DeserializeScalarSnafu)
            }
            ValueDeserializer::Compound(compound_deserializer) => {
                compound_deserializer.deserialize_option(visitor)
            }
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de, K, S, BH> IntoDeserializer<'de, Error> for ValueDeserializer<'de, K, S, BH>
where
    K: Borrow<Key> + Eq + Hash,
    S: AsRef<str>,
    BH: std::hash::BuildHasher,
{
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

/// CompoundDeserializer is our main structure that drives serde's MapDeserializer and stores the
/// state we need to understand the recursive structure of the output.
struct CompoundDeserializer<'de, K, S, BH> {
    /// A reference to the input data we're deserializing.
    map: &'de HashMap<K, S, BH>,
    /// The keys that we need to consider in this iteration.  Starts out the same as the keys
    /// of the input map, but on recursive calls it's only the keys that are relevant to the
    /// sub-struct we're handling, with the duplicated prefix (the 'path') removed.
    keys: HashSet<Key>,
    /// The path tells us where we are in our recursive structures.
    path: Option<Key>,
}

impl<'de, K, S, BH> CompoundDeserializer<'de, K, S, BH>
where
    BH: std::hash::BuildHasher,
{
    fn new(
        map: &'de HashMap<K, S, BH>,
        keys: HashSet<Key>,
        path: Option<Key>,
    ) -> CompoundDeserializer<'de, K, S, BH> {
        CompoundDeserializer { map, keys, path }
    }
}

fn bad_root<T>() -> Result<T> {
    error::BadRootSnafu.fail()
}

impl<'de, K, S, BH> serde::de::Deserializer<'de> for CompoundDeserializer<'de, K, S, BH>
where
    K: Borrow<Key> + Eq + Hash,
    S: AsRef<str>,
    BH: std::hash::BuildHasher,
{
    type Error = Error;

    fn deserialize_struct<V>(
        mut self,
        name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // On the first interaction for a struct, we won't have a prefix yet, unless the user called
        // from_map_with_prefix and specified it.  We can make the prefix from the struct name.
        // (Recursive calls will have a path but no name, because we always treat nested structures
        // as maps, because we don't need any nested struct names and it lets us use the nice
        // MapDeserializer.)
        if !name.is_empty() {
            trace!("Path before name check: {:?}", self.path);
            if self.path.is_none() {
                self.path = Some(
                    // to_lowercase handles the discrepancy between key naming and struct naming;
                    // this initial 'path' creation is the only place we take the struct name from
                    // serde, per above comment.
                    Key::from_segments(KeyType::Data, &[name.to_lowercase()])
                        .context(error::InvalidPrefixSnafu { prefix: name })?,
                );
            }
            trace!("Path after name check: {:?}", self.path);
        }

        if let Some(ref path) = self.path {
            // Remove the known path from the beginning of the keys. serde doesn't care about the
            // name of the top-level struct, just the fields inside, so we have to remove it before
            // handing it to the MapDeserializer.  (Our real customer is the one specifying the
            // dotted keys, and we always use the struct name there for clarity.)
            trace!("Keys before path strip: {:?}", self.keys);
            let mut new_keys = HashSet::new();
            for key in self.keys {
                new_keys.insert(key.strip_prefix_segments(path.segments()).context(
                    error::StripPrefixSnafu {
                        prefix: path.name(),
                        name: key.name(),
                    },
                )?);
            }
            self.keys = new_keys;
            trace!("Keys after path strip: {:?}", self.keys);
        }

        // We have to track which structs we've already handled and skip over them.  This is
        // because we could get keys like "a.b.c" and "a.b.d", so we'll see that "a" prefix
        // twice at the top level, but by the time we see the second one we've already recursed
        // and handled all of "a" from the first one.
        let mut structs_done = HashSet::new();

        // As mentioned above, MapDeserializer does a lot of nice work for us.  We just need to
        // give it an iterator that yields (key, deserializer) pairs.  The nested deserializers
        // have the appropriate 'path' and a subset of 'keys' so they can do their job.
        visitor.visit_map(MapDeserializer::new(self.keys.iter().filter_map(|key| {
            let mut segments: VecDeque<_> = key.segments().clone().into();
            // Inside this filter_map closure, we can't return early from the outer function, so we
            // log an error and skip the key.  Errors in this path are generally logic errors
            // rather than user errors, so this isn't so bad.
            let struct_name = match segments.pop_front() {
                Some(s) => s,
                None => {
                    error!("Logic error - Key segments.pop_front failed, empty Key?");
                    return None;
                }
            };
            trace!("Visiting key '{}', struct name '{}'", key, &struct_name);

            // At the top level (None path) we start with struct_name as Key, otherwise append
            // struct_name.
            trace!("Old path: {:?}", &self.path);
            let path = match self.path {
                None => match Key::from_segments(KeyType::Data, &[&struct_name]) {
                    Ok(key) => key,
                    Err(e) => {
                        error!(
                            "Tried to construct invalid key from struct name '{}', skipping: {}",
                            &struct_name, e
                        );
                        return None;
                    }
                },
                Some(ref old_path) => match old_path.append_segments(&[&struct_name]) {
                    Ok(key) => key,
                    Err(e) => {
                        error!(
                            "Appending '{}' to existing key '{}' resulted in invalid key, skipping: {}",
                            old_path, &struct_name, e
                        );
                        return None;
                    }
                }
            };
            trace!("New path: {}", &path);

            if !segments.is_empty() {
                if structs_done.contains(&struct_name) {
                    // We've handled this structure with a recursive call, so we're done.
                    trace!("Already handled struct '{}', skipping", &struct_name);
                    None
                } else {
                    // Otherwise, mark it, and recurse.
                    structs_done.insert(struct_name.clone());

                    // Subset the keys so the recursive call knows what it needs to handle -
                    // only things starting with the new path.
                    let keys = self
                        .keys
                        .iter()
                        .filter(|new_key| new_key.starts_with_segments(&[&struct_name]))
                        // Remove the prefix - should always work, but log and skip the key otherwise
                        .filter_map(|new_key| new_key
                                    .strip_prefix(&struct_name)
                                    .map_err(|e| error!("Key starting with segment '{}' couldn't remove it as prefix: {}", &struct_name, e)).ok())
                        .collect();

                    // And here's what MapDeserializer expects, the key and deserializer for it
                    trace!(
                        "Recursing for struct '{}' with keys: {:?}",
                        &struct_name,
                        keys
                    );
                    Some((
                        struct_name,
                        ValueDeserializer::Compound(CompoundDeserializer::new(
                            self.map,
                            keys,
                            Some(path),
                        )),
                    ))
                }
            } else {
                // No dot, so we have a scalar; hand the data to a scalar deserializer.
                trace!(
                    "Key '{}' is scalar, getting '{}' from input to deserialize",
                    struct_name,
                    path
                );
                let val = self.map.get(&path)?;
                Some((
                    struct_name,
                    ValueDeserializer::Scalar(deserializer_for_scalar(val.as_ref())),
                ))
            }
        })))
    }

    /// We use deserialize_map for all maps, including top-level maps, but to allow top-level maps
    /// we require that the user specified a prefix for us using from_map_with_prefix.
    ///
    /// We also use it for structs below the top level, because you don't need a name once you're
    /// recursing - you'd always be pointed to by a struct field or map key whose name we use.
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.path {
            Some(_) => self.deserialize_struct("", &[], visitor),
            None => bad_root(),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    /// Scalar types, and compound types we can't use at the root, are forwarded here to be
    /// rejected.  (Compound types need to have a name to serve at the root level.)
    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        bad_root()
    }

    // This gives us the rest of the implementations needed to compile, and forwards them to the
    // function above that will reject them.
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct enum identifier ignored_any
    }
}

#[cfg(test)]
mod test {
    use super::{from_map, from_map_with_prefix};
    use crate::{deserialization::Error, Key, KeyType};

    use maplit::hashmap;
    use serde::Deserialize;
    use std::collections::HashMap;

    // Helper macro for making a data Key for testing whose name we know is valid.
    macro_rules! key {
        ($name:expr) => {
            Key::new(KeyType::Data, $name).unwrap()
        };
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct A {
        id: Option<u64>,
        name: String,
        list: Vec<u8>,
        nested: B,
        map: HashMap<String, String>,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct B {
        a: String,
        b: bool,
        c: Option<i64>,
        d: Option<C>,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct C {
        boolean: bool,
    }

    #[test]
    fn basic_struct_works() {
        let c: C = from_map(&hashmap! {
            key!("c.boolean") => "true".to_string(),
        })
        .unwrap();
        assert_eq!(c, C { boolean: true });
    }

    #[test]
    fn deep_struct_works() {
        let a: A = from_map(&hashmap! {
            key!("a.id") => "1".to_string(),
            key!("a.name") => "\"it's my name\"".to_string(),
            key!("a.list") => "[1,2, 3, 4]".to_string(),
            key!("a.map.a") => "\"answer is always map\"".to_string(),
            key!("a.nested.a") => "\"quite nested\"".to_string(),
            key!("a.nested.b") => "false".to_string(),
            key!("a.nested.c") => "null".to_string(),
            key!("a.nested.d.boolean") => "true".to_string(),
        })
        .unwrap();
        assert_eq!(
            a,
            A {
                id: Some(1),
                name: "it's my name".to_string(),
                list: vec![1, 2, 3, 4],
                map: hashmap! {
                    "a".to_string() => "answer is always map".to_string(),
                },
                nested: B {
                    a: "quite nested".to_string(),
                    b: false,
                    c: None,
                    d: Some(C { boolean: true })
                }
            }
        );
    }

    #[test]
    fn map_doesnt_work_at_root() {
        let a: Result<HashMap<String, String>, Error> = from_map(&hashmap! {
            key!("a") => "\"it's a\"".to_string(),
            key!("b") => "\"it's b\"".to_string(),
        });
        a.unwrap_err();
    }

    #[test]
    fn map_works_at_root_with_prefix() {
        let map = &hashmap! {
            key!("x.boolean") => "true".to_string()
        };
        let x: HashMap<String, bool> = from_map_with_prefix(Some("x".to_string()), map).unwrap();
        assert_eq!(
            x,
            hashmap! {
                "boolean".to_string() => true,
            }
        );
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct Bad {
        id: u64,
    }

    #[test]
    fn disallowed_data_type() {
        let bad: Result<Bad, Error> = from_map(&hashmap! {
            key!("id") => "42".to_string(),
        });
        bad.unwrap_err();
    }
}
