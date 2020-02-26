use semver::Version;
use serde::ser::Error as _;
use serde::{Serialize, Serializer};
use std::collections::BTreeMap;

pub(crate) fn serialize_migration<S>(
    value: &BTreeMap<(Version, Version), Vec<String>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map = BTreeMap::new();
    for ((from, to), val) in value {
        let key = format!(
            "({}, {})",
            serde_plain::to_string(&from).map_err(|e| S::Error::custom(format!(
                "Could not serialize 'from' version: {}",
                e
            )))?,
            serde_plain::to_string(&to).map_err(|e| S::Error::custom(format!(
                "Could not serialize 'to' version: {}",
                e
            )))?,
        );
        map.insert(key, val);
    }
    map.serialize(serializer)
}
