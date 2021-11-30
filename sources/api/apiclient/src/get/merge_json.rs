use serde_json::{map::Entry, Value};

/// This modifies the first given JSON Value by inserting any values from the second Value.
///
/// This is done recursively.  Any time a scalar or array is seen, the left side is set to match
/// the right side.  Any time an object is seen, we iterate through the keys of the objects; if the
/// left side does not have the key from the right side, it's inserted, otherwise we recursively
/// merge the values in each object for that key.
// Logic and tests taken from storewolf::merge-toml, modified for serde_json.
pub(super) fn merge_json(merge_into: &mut Value, merge_from: Value) {
    match (merge_into, merge_from) {
        // If we see objects, we recursively merge each key.
        (Value::Object(merge_into), Value::Object(merge_from)) => {
            for (merge_from_key, merge_from_val) in merge_from.into_iter() {
                // Check if the left has the same key as the right.
                match merge_into.entry(merge_from_key) {
                    // If not, we can just insert the value.
                    Entry::Vacant(entry) => {
                        entry.insert(merge_from_val);
                    }
                    // If so, we need to recursively merge; we don't want to replace an entire
                    // table, for example, because the left may have some distinct inner keys.
                    Entry::Occupied(ref mut entry) => {
                        merge_json(entry.get_mut(), merge_from_val);
                    }
                }
            }
        }

        // If we see a scalar, we replace the left with the right.  We treat arrays like scalars so
        // behavior is clear - no question about whether we're appending right onto left, etc.
        (merge_into, merge_from) => {
            *merge_into = merge_from;
        }
    }
}

#[cfg(test)]
mod test {
    use super::merge_json;
    use serde_json::json;

    #[test]
    fn recursion() {
        let mut left = json! {{
            "top1": "left top1",
            "top2": "left top2",
            "settings": {
                "inner": {
                    "inner_setting1": "left inner_setting1",
                    "inner_setting2": "left inner_setting2"
                }
            }
        }};
        let right = json! {{
            "top1": "right top1",
            "settings": {
                "setting": "right setting",
                "inner": {
                    "inner_setting1": "right inner_setting1",
                    "inner_setting3": "right inner_setting3"
                }
            }
        }};
        let expected = json! {{
            // "top1" is being overwritten from right.
            "top1": "right top1",
            // "top2" is only in the left and remains.
            "top2": "left top2",
            "settings": {
                // "setting" is only in the right side.
                "setting": "right setting",
                // "inner" tests that recursion works.
                "inner": {
                    // inner_setting1 is replaced.
                    "inner_setting1": "right inner_setting1",
                    // 2 is untouched.
                    "inner_setting2": "left inner_setting2",
                    // 3 is new.
                    "inner_setting3": "right inner_setting3"
                }
            }
        }};
        merge_json(&mut left, right);
        assert_eq!(left, expected);
    }

    #[test]
    fn array() {
        let mut left = json!({"a": [1, 2, 3]});
        let right = json!({"a": [4, 5]});
        let expected = json!({"a": [4, 5]});
        merge_json(&mut left, right);
        assert_eq!(left, expected);
    }
}
