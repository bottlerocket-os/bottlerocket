use data_store_version::Version as DataVersion;
use semver::Version as SemVer;
use serde::{Serialize, Serializer};
use std::collections::BTreeMap;

pub(crate) fn serialize_migration<S>(
    value: &BTreeMap<(DataVersion, DataVersion), Vec<String>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map = BTreeMap::new();
    for ((from, to), val) in value {
        let key = format!(
            "({},{})",
            from.to_string().trim_start_matches('v'),
            to.to_string().trim_start_matches('v')
        );
        map.insert(key, val);
    }
    map.serialize(serializer)
}

pub(crate) fn serialize_datastore_map<S>(
    value: &BTreeMap<SemVer, DataVersion>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map = BTreeMap::new();
    for (image, datastore) in value {
        let datastore = String::from(datastore.to_string().trim_start_matches('v'));
        map.insert(image.to_string(), datastore);
    }
    map.serialize(serializer)
}
