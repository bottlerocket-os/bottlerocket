use data_store_version::Version as DVersion;
use semver::{Version};
use serde::{Serialize, Serializer};
use std::collections::BTreeMap;

pub(crate) fn serialize_migration<S>(value: &BTreeMap<(DVersion, DVersion), Vec<String>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map = BTreeMap::new();
    for ((from, to), val) in value {
        let key = String::from(format!("({},{})", from, to));
        map.insert(key, val);
    }
    map.serialize(serializer)
}

pub(crate) fn serialize_datastore_map<S>(value: &BTreeMap<Version, DVersion>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map = BTreeMap::new();
    for (image, datastore) in value {
        map.insert(image.to_string(), datastore.to_string());
    }
    map.serialize(serializer)
}