// This is only used in the build script for now, but it's in a separate crate so we can have unit
// tests.

use snafu::ensure;
use toml::{map::Entry, Value};

/// This modifies the first given toml Value by inserting any values from the second Value.
///
/// This is done recursively.  Any time a scalar or array is seen, the left side is set to match
/// the right side.  Any time a table is seen, we iterate through the keys of the tables; if the
/// left side does not have the key from the right side, it's inserted, otherwise we recursively
/// merge the values in each table for that key.
///
/// If at any point in the recursion the data types of the two values does not match, we error.
pub fn merge_values<'a>(merge_into: &'a mut Value, merge_from: &'a Value) -> Result<()> {
    // If the types of left and right don't match, we have inconsistent models, and shouldn't try
    // to merge them.
    ensure!(
        merge_into.same_type(merge_from),
        error::DataTypeMismatchSnafu
    );

    match merge_from {
        // If we see a scalar, we replace the left with the right.  We treat arrays like scalars so
        // behavior is clear - no question about whether we're appending right onto left, etc.
        Value::String(_)
        | Value::Integer(_)
        | Value::Float(_)
        | Value::Boolean(_)
        | Value::Datetime(_)
        | Value::Array(_) => *merge_into = merge_from.clone(),

        // If we see a table, we recursively merge each key.
        Value::Table(t2) => {
            // We know the other side is a table because of the `ensure` above.
            let t1 = merge_into.as_table_mut().unwrap();
            for (k2, v2) in t2.iter() {
                // Check if the left has the same key as the right.
                match t1.entry(k2) {
                    // If not, we can just insert the value.
                    Entry::Vacant(e) => {
                        e.insert(v2.clone());
                    }
                    // If so, we need to recursively merge; we don't want to replace an entire
                    // table, for example, because the left may have some distinct inner keys.
                    Entry::Occupied(ref mut e) => {
                        merge_values(e.get_mut(), v2)?;
                    }
                }
            }
        }
    }

    Ok(())
}

mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub enum Error {
        #[snafu(display("Cannot merge mismatched data types in given TOML"))]
        DataTypeMismatch {},
    }
}

pub use error::Error;
type Result<T> = std::result::Result<T, error::Error>;

#[cfg(test)]
mod test {
    use super::merge_values;
    use toml::{toml, Value};

    #[test]
    fn merge() {
        let mut left = Value::Table(toml! {
            top1 = "left top1"
            top2 = "left top2"
            [settings.inner]
            inner_setting1 = "left inner_setting1"
            inner_setting2 = "left inner_setting2"
        });
        let right = Value::Table(toml! {
            top1 = "right top1"
            [settings]
            setting = "right setting"
            [settings.inner]
            inner_setting1 = "right inner_setting1"
            inner_setting3 = "right inner_setting3"
        });
        // Can't comment inside this toml, unfortunately.
        // "top1" is being overwritten from right.
        // "top2" is only in the left and remains.
        // "setting" is only in the right side.
        // "inner" tests that recursion works; inner_setting1 is replaced, 2 is untouched, and
        // 3 is new.
        let expected = Value::Table(toml! {
            top1 = "right top1"
            top2 = "left top2"
            [settings]
            setting = "right setting"
            [settings.inner]
            inner_setting1 = "right inner_setting1"
            inner_setting2 = "left inner_setting2"
            inner_setting3 = "right inner_setting3"
        });
        merge_values(&mut left, &right).unwrap();
        assert_eq!(left, expected);
    }
}
