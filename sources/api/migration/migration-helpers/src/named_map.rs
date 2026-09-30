//! Helpers for migrating lists and named maps in flattened datastore data and metadata.

use crate::{error, MigrationData, Result};
use datastore::{Key, KeyType, Value};
use snafu::ResultExt;
use std::collections::{BTreeMap, HashMap};

/// Controls how a named map is represented when migrating it back to a list.
pub enum ListReplacement {
    /// Remove the named map without writing a list.
    Remove,
    /// Replace the named map with this list, including when the list is empty.
    Replace(Vec<Value>),
}

/// Converts a list setting into scalar fields below `prefix.<name>.<field>`.
///
/// The converter owns the value-specific mapping. Returning `None` leaves the input unchanged.
/// This helper only changes data; callers must migrate related metadata separately.
pub fn migrate_list_to_named_map<F>(
    mut input: MigrationData,
    list_key: &str,
    prefix: &[&str],
    converter: F,
) -> Result<MigrationData>
where
    F: FnOnce(Vec<Value>) -> Result<Option<BTreeMap<String, BTreeMap<String, Value>>>>,
{
    validate_prefix(prefix)?;
    let values = match input.data.get(list_key) {
        Some(Value::Array(values)) => values.clone(),
        Some(value) => {
            println!("Found invalid '{list_key}' value ('{value}'); leaving it unchanged");
            return Ok(input);
        }
        None => {
            println!("Found no '{list_key}' to migrate on upgrade");
            return Ok(input);
        }
    };

    let Some(replacement) = converter(values)? else {
        return Ok(input);
    };
    replace_flattened_named_map(&mut input.data, prefix, &replacement)?;
    input.data.remove(list_key);
    Ok(input)
}

/// Converts scalar fields below `prefix.<name>.<field>` into a list setting.
///
/// The converter owns the value-specific mapping. Returning `None` leaves the input unchanged.
/// This helper only changes data; callers must migrate related metadata separately.
pub fn migrate_named_map_to_list<F>(
    mut input: MigrationData,
    prefix: &[&str],
    list_key: &str,
    converter: F,
) -> Result<MigrationData>
where
    F: FnOnce(BTreeMap<String, BTreeMap<String, Value>>) -> Result<Option<ListReplacement>>,
{
    validate_prefix(prefix)?;
    if input.data.contains_key(list_key) {
        replace_flattened_named_map(&mut input.data, prefix, &BTreeMap::new())?;
        return Ok(input);
    }

    let Some(named_map) = read_flattened_named_map(&input.data, prefix)? else {
        println!(
            "Found no named map at '{}' to migrate on downgrade",
            prefix.join(".")
        );
        return Ok(input);
    };
    let Some(replacement) = converter(named_map)? else {
        return Ok(input);
    };

    replace_flattened_named_map(&mut input.data, prefix, &BTreeMap::new())?;
    if let ListReplacement::Replace(values) = replacement {
        input.data.insert(list_key.into(), Value::Array(values));
    }
    Ok(input)
}

/// Reads data or metadata stored below `prefix.<name>.<field>`.
pub fn read_flattened_named_map<V: Clone>(
    data: &HashMap<String, V>,
    prefix: &[&str],
) -> Result<Option<BTreeMap<String, BTreeMap<String, V>>>> {
    validate_prefix(prefix)?;
    let mut result: BTreeMap<String, BTreeMap<String, V>> = BTreeMap::new();

    for (name, value) in data {
        let key = Key::new(KeyType::Data, name).context(error::InvalidKeySnafu {
            key_type: KeyType::Data,
            key: name,
        })?;
        if !key.starts_with_segments(prefix) || key.segments().len() == prefix.len() {
            continue;
        }
        if key.segments().len() != prefix.len() + 2 {
            return error::MigrationSnafu {
                msg: format!("Named map key '{key}' must contain an item name and field"),
            }
            .fail();
        }

        let item = key.segments()[prefix.len()].clone();
        let field = key.segments()[prefix.len() + 1].clone();
        if result
            .entry(item.clone())
            .or_default()
            .insert(field.clone(), value.clone())
            .is_some()
        {
            return error::MigrationSnafu {
                msg: format!("Named map contains duplicate field '{item}.{field}'"),
            }
            .fail();
        }
    }

    Ok((!result.is_empty()).then_some(result))
}

fn validate_prefix(prefix: &[&str]) -> Result<()> {
    if prefix.is_empty() {
        return error::MigrationSnafu {
            msg: "Named map prefix cannot be empty".to_string(),
        }
        .fail();
    }
    Ok(())
}

/// Replaces data or metadata stored below `prefix.<name>.<field>`.
///
/// Output keys are validated before existing named-map fields are removed.
pub fn replace_flattened_named_map<V: Clone>(
    data: &mut HashMap<String, V>,
    prefix: &[&str],
    replacement: &BTreeMap<String, BTreeMap<String, V>>,
) -> Result<()> {
    validate_prefix(prefix)?;
    let mut replacement_data = Vec::new();
    for (item, fields) in replacement {
        for (field, value) in fields {
            let segments = prefix
                .iter()
                .copied()
                .chain([item.as_str(), field.as_str()])
                .collect::<Vec<_>>();
            let key =
                Key::from_segments(KeyType::Data, &segments).context(error::InvalidKeySnafu {
                    key_type: KeyType::Data,
                    key: segments.join("."),
                })?;
            replacement_data.push((key.name().clone(), value.clone()));
        }
    }

    let mut keys_to_remove = Vec::new();
    for name in data.keys() {
        let key = Key::new(KeyType::Data, name).context(error::InvalidKeySnafu {
            key_type: KeyType::Data,
            key: name,
        })?;
        if key.starts_with_segments(prefix) {
            if key.segments().len() == prefix.len() {
                continue;
            }
            if key.segments().len() != prefix.len() + 2 {
                return error::MigrationSnafu {
                    msg: format!("Named map key '{key}' must contain an item name and field"),
                }
                .fail();
            }
            keys_to_remove.push(name.clone());
        }
    }

    for key in keys_to_remove {
        data.remove(&key);
    }
    data.extend(replacement_data);
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Metadata;
    use maplit::{btreemap, hashmap};

    const PREFIX: &[&str] = &["settings", "items"];

    fn migration_data(data: HashMap<String, Value>) -> MigrationData {
        MigrationData {
            data,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn migrates_list_to_named_map() {
        let input = migration_data(hashmap! {
            "settings.items".into() => vec!["one", "two"].into(),
            "settings.items.stale.value".into() => "stale".into(),
            "settings.other".into() => true.into(),
        });

        let result = migrate_list_to_named_map(input, "settings.items", PREFIX, |values| {
            assert_eq!(values, vec![Value::from("one"), Value::from("two")]);
            Ok(Some(btreemap! {
                "first".into() => btreemap! {
                    "value".into() => "one".into(),
                },
                "second".into() => btreemap! {
                    "value".into() => "two".into(),
                },
            }))
        })
        .unwrap();

        assert_eq!(
            result.data,
            hashmap! {
                "settings.items.first.value".into() => "one".into(),
                "settings.items.second.value".into() => "two".into(),
                "settings.other".into() => true.into(),
            }
        );
    }

    #[test]
    fn migrates_named_map_to_list() {
        let input = migration_data(hashmap! {
            "settings.items.first.value".into() => "one".into(),
            "settings.items.second.value".into() => "two".into(),
            "settings.other".into() => true.into(),
        });

        let result = migrate_named_map_to_list(input, PREFIX, "settings.items", |named_map| {
            let values = named_map
                .into_values()
                .map(|fields| fields["value"].clone())
                .collect();
            Ok(Some(ListReplacement::Replace(values)))
        })
        .unwrap();

        assert_eq!(
            result.data,
            hashmap! {
                "settings.items".into() => vec!["one", "two"].into(),
                "settings.other".into() => true.into(),
            }
        );
    }

    #[test]
    fn converter_can_leave_input_unchanged() {
        let input = migration_data(hashmap! {
            "settings.items".into() => vec!["one"].into(),
        });

        let result =
            migrate_list_to_named_map(input.clone(), "settings.items", PREFIX, |_| Ok(None))
                .unwrap();
        assert_eq!(result, input);
    }

    #[test]
    fn named_map_converter_can_leave_input_unchanged() {
        let input = migration_data(hashmap! {
            "settings.items.first.value".into() => "one".into(),
        });

        let result =
            migrate_named_map_to_list(input.clone(), PREFIX, "settings.items", |_| Ok(None))
                .unwrap();
        assert_eq!(result, input);
    }

    #[test]
    fn writes_empty_list() {
        let input = migration_data(hashmap! {
            "settings.items.first.value".into() => "one".into(),
        });

        let result = migrate_named_map_to_list(input, PREFIX, "settings.items", |_| {
            Ok(Some(ListReplacement::Replace(Vec::new())))
        })
        .unwrap();
        assert_eq!(
            result.data,
            hashmap! {
                "settings.items".into() => Value::Array(Vec::new()),
            }
        );
    }

    #[test]
    fn removes_named_map_without_writing_list() {
        let input = migration_data(hashmap! {
            "settings.items.first.value".into() => "one".into(),
            "settings.other".into() => true.into(),
        });

        let result = migrate_named_map_to_list(input, PREFIX, "settings.items", |_| {
            Ok(Some(ListReplacement::Remove))
        })
        .unwrap();
        assert_eq!(
            result.data,
            hashmap! {
                "settings.other".into() => true.into(),
            }
        );
    }

    #[test]
    fn rejects_empty_prefix() {
        let input = migration_data(hashmap! {
            "settings.items".into() => vec!["one"].into(),
        });

        assert!(
            migrate_list_to_named_map(input, "settings.items", &[], |_| {
                Ok(Some(BTreeMap::new()))
            })
            .is_err()
        );

        let data = HashMap::<String, Value>::new();
        assert!(read_flattened_named_map(&data, &[]).is_err());

        let mut data = HashMap::<String, Value>::new();
        assert!(replace_flattened_named_map(&mut data, &[], &BTreeMap::new()).is_err());
    }

    #[test]
    fn reads_named_map() {
        let data: HashMap<String, Value> = hashmap! {
            "settings.items.first.value".into() => "one".into(),
            "settings.items.second.value".into() => "two".into(),
            "settings.other".into() => true.into(),
        };

        assert_eq!(
            read_flattened_named_map(&data, PREFIX).unwrap(),
            Some(btreemap! {
                "first".into() => btreemap! {
                    "value".into() => "one".into(),
                },
                "second".into() => btreemap! {
                    "value".into() => "two".into(),
                },
            })
        );
    }

    #[test]
    fn handles_quoted_item_names() {
        let data: HashMap<String, Value> = hashmap! {
            r#"settings.items."first.item".value"#.into() => "one".into(),
        };

        let map = read_flattened_named_map(&data, PREFIX).unwrap().unwrap();
        assert_eq!(map["first.item"]["value"], Value::String("one".into()));

        let mut output = HashMap::new();
        replace_flattened_named_map(&mut output, PREFIX, &map).unwrap();
        assert_eq!(output, data);
    }

    #[test]
    fn reads_and_replaces_metadata() {
        let strong: Metadata = hashmap! {
            "strength".into() => Value::String("strong".into()),
        };
        let weak: Metadata = hashmap! {
            "strength".into() => Value::String("weak".into()),
        };
        let data = hashmap! {
            "settings.items.first.value".into() => strong.clone(),
            "settings.items.second.value".into() => weak.clone(),
        };

        let map = read_flattened_named_map(&data, PREFIX).unwrap().unwrap();
        assert_eq!(map["first"]["value"], strong);
        assert_eq!(map["second"]["value"], weak);

        let mut output = HashMap::new();
        replace_flattened_named_map(&mut output, PREFIX, &map).unwrap();
        assert_eq!(output, data);
    }

    #[test]
    fn rejects_duplicate_logical_fields() {
        let data: HashMap<String, Value> = hashmap! {
            "settings.items.first.value".into() => "one".into(),
            r#"settings.items."first".value"#.into() => "two".into(),
        };

        assert!(read_flattened_named_map(&data, PREFIX).is_err());
    }

    #[test]
    fn ignores_missing_map_and_parent_value() {
        let data: HashMap<String, Value> = hashmap! {
            "settings.items".into() => vec!["legacy"].into(),
            "settings.other".into() => true.into(),
        };

        assert_eq!(read_flattened_named_map(&data, PREFIX).unwrap(), None);
    }

    #[test]
    fn rejects_nested_fields() {
        let mut data: HashMap<String, Value> = hashmap! {
            "settings.items.first.nested.value".into() => "one".into(),
        };

        assert!(read_flattened_named_map(&data, PREFIX).is_err());
        let original = data.clone();
        assert!(replace_flattened_named_map(&mut data, PREFIX, &BTreeMap::new()).is_err());
        assert_eq!(data, original);
    }

    #[test]
    fn replace_preserves_parent_and_unrelated_data() {
        let mut data: HashMap<String, Value> = hashmap! {
            "settings.items".into() => vec!["legacy"].into(),
            "settings.items.old.value".into() => "old".into(),
            "settings.other".into() => true.into(),
        };
        let replacement = btreemap! {
            "new".into() => btreemap! {
                "value".into() => "new".into(),
            },
        };

        replace_flattened_named_map(&mut data, PREFIX, &replacement).unwrap();
        assert_eq!(
            data,
            hashmap! {
                "settings.items".into() => vec!["legacy"].into(),
                "settings.items.new.value".into() => "new".into(),
                "settings.other".into() => true.into(),
            }
        );
    }
}
