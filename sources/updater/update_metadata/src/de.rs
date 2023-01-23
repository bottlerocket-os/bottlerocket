use crate::error;
use chrono::{DateTime, Utc};
use regex::Regex;
use semver::Version;
use serde::{de::Error as _, Deserializer};
use snafu::{ensure, ResultExt};
use std::collections::BTreeMap;
use std::fmt;

/// Converts the bound key to an integer before insertion and catches duplicates
pub(crate) fn deserialize_bound<'de, D>(
    deserializer: D,
) -> Result<BTreeMap<u32, DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    fn bound_to_int(
        key: String,
        time: DateTime<Utc>,
        map: &mut BTreeMap<u32, DateTime<Utc>>,
    ) -> Result<(), error::Error> {
        let bound = key
            .parse::<u32>()
            .context(error::BadBoundSnafu { bound_str: key })?;
        ensure!(
            map.insert(bound, time).is_none(),
            error::DuplicateKeyIdSnafu { keyid: bound }
        );
        Ok(())
    }

    // The rest of this is fitting the above function into serde and doing error type conversion.
    struct Visitor;

    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = BTreeMap<u32, DateTime<Utc>>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
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

/// Converts the tuple keys to a `Version` before insertion and catches duplicates
#[allow(clippy::type_complexity)]
pub(crate) fn deserialize_migration<'de, D>(
    deserializer: D,
) -> Result<BTreeMap<(Version, Version), Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    fn parse_versions(key: &str) -> Result<(&str, &str), error::Error> {
        // We don't need to be too strict about the version syntax here, it's parsed after we
        // return these strings.  We just want to make sure we have "(X, Y)" syntax and split it.
        let r = Regex::new(r"\((?P<from_ver>[^ ,]+),[ ]*(?P<to_ver>[^ ,]+)\)");

        if let Ok(regex) = r {
            if let Some(captures) = regex.captures(key) {
                if let (Some(from), Some(to)) = (captures.name("from_ver"), captures.name("to_ver"))
                {
                    return Ok((from.as_str(), to.as_str()));
                }
            }
        }
        error::BadDataVersionsFromToSnafu { key }.fail()
    }

    fn parse_tuple_key(
        key: String,
        list: Vec<String>,
        map: &mut BTreeMap<(Version, Version), Vec<String>>,
    ) -> Result<(), error::Error> {
        let (from, to) = parse_versions(&key)?;

        if let (Ok(from), Ok(to)) = (serde_plain::from_str(from), serde_plain::from_str(to)) {
            ensure!(
                map.insert((from, to), list).is_none(),
                error::DuplicateVersionKeySnafu { key }
            );
        } else {
            return error::BadDataVersionsFromToSnafu {
                key: format!("{from}, {to}"),
            }
            .fail();
        }

        Ok(())
    }

    // The rest of this is fitting the above function into serde and doing error type conversion.
    struct Visitor;

    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = BTreeMap<(Version, Version), Vec<String>>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
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
