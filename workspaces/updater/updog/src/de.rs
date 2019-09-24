use crate::error;
use chrono::{DateTime, Utc};
use data_store_version::Version as DVersion;
use regex::Regex;
use semver::Version;
use serde::{de::Error as _, Deserializer};
use snafu::{ensure, ResultExt};
use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

/// Converts the bound key to an integer before insertion and catches duplicates
pub(crate) fn deserialize_bound<'de, D>(
    deserializer: D,
) -> Result<BTreeMap<u64, DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    fn bound_to_int(
        key: String,
        time: DateTime<Utc>,
        map: &mut BTreeMap<u64, DateTime<Utc>>,
    ) -> Result<(), error::Error> {
        let bound = key
            .parse::<u64>()
            .context(error::BadBound { bound_str: key })?;
        ensure!(
            map.insert(bound, time).is_none(),
            error::DuplicateKeyId { keyid: bound }
        );
        Ok(())
    }

    // The rest of this is fitting the above function into serde and doing error type conversion.
    struct Visitor;

    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = BTreeMap<u64, DateTime<Utc>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map")
        }

        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: serde::de::MapAccess<'de>,
        {
            let mut map = BTreeMap::new();
            while let Some((bound, time)) = access.next_entry()? {
                bound_to_int(bound, time, &mut map).map_err(M::Error::custom)?;
            }
            Ok(map)
        }
    }

    deserializer.deserialize_map(Visitor)
}

/// Converts the tuple keys to a `DVersion` before insertion and catches duplicates
pub(crate) fn deserialize_migration<'de, D>(
    deserializer: D,
) -> Result<BTreeMap<(DVersion, DVersion), Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    fn parse_versions(key: &str) -> Result<(&str, &str), error::Error> {
        let r = Regex::new(r"\((?P<from_ver>[0-9.]+),[ ]+(?P<to_ver>[0-9.]+)\)");

        if let Ok(regex) = r {
            if let Some(captures) = regex.captures(&key) {
                if let (Some(from), Some(to)) = (captures.name("from_ver"), captures.name("to_ver"))
                {
                    return Ok((from.as_str(), to.as_str()));
                }
            }
        }
        error::BadDataVersion { key }.fail()
    }

    fn parse_tuple_key(
        key: String,
        list: Vec<String>,
        map: &mut BTreeMap<(DVersion, DVersion), Vec<String>>,
    ) -> Result<(), error::Error> {
        let (from, to) = parse_versions(&key)?;

        if let (Ok(from), Ok(to)) = (DVersion::from_str(from), DVersion::from_str(to)) {
            ensure!(
                map.insert((from, to), list).is_none(),
                error::DuplicateVersionKey { key }
            );
        } else {
            return error::BadDataVersion {
                key: format!("{}, {}", from, to),
            }
            .fail();
        }

        Ok(())
    }

    // The rest of this is fitting the above function into serde and doing error type conversion.
    struct Visitor;

    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = BTreeMap<(DVersion, DVersion), Vec<String>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map")
        }

        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: serde::de::MapAccess<'de>,
        {
            let mut map = BTreeMap::new();
            while let Some((tuple, list)) = access.next_entry()? {
                parse_tuple_key(tuple, list, &mut map).map_err(M::Error::custom)?;
            }
            Ok(map)
        }
    }

    deserializer.deserialize_map(Visitor)
}

/// Converts the key and value into a Version/DVersion pair before insertion and
/// catches duplicates
pub(crate) fn deserialize_datastore_version<'de, D>(
    deserializer: D,
) -> Result<BTreeMap<Version, DVersion>, D::Error>
where
    D: Deserializer<'de>,
{
    fn to_versions(
        key: String,
        value: String,
        map: &mut BTreeMap<Version, DVersion>,
    ) -> Result<(), error::Error> {
        let key_ver = Version::parse(&key);
        let value_ver = DVersion::from_str(&value);
        match (key_ver, value_ver) {
            (Ok(k), Ok(v)) => {
                ensure!(
                    map.insert(k, v).is_none(),
                    error::DuplicateVersionKey { key }
                );
            }
            _ => return error::BadMapVersion { key, value }.fail(),
        }
        Ok(())
    }

    // The rest of this is fitting the above function into serde and doing error type conversion.
    struct Visitor;

    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = BTreeMap<Version, DVersion>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map")
        }

        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: serde::de::MapAccess<'de>,
        {
            let mut map = BTreeMap::new();
            while let Some((t_ver, d_ver)) = access.next_entry()? {
                to_versions(t_ver, d_ver, &mut map).map_err(M::Error::custom)?;
            }
            Ok(map)
        }
    }

    deserializer.deserialize_map(Visitor)
}
