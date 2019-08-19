use crate::error;
use chrono::{DateTime, Utc};
use serde::{de::Error as _, Deserializer,};
use snafu::{ensure, ResultExt};
use std::collections::BTreeMap;
use std::fmt;

/// Converts the bound key to an integer before insertion and catches duplicates
pub(crate) fn deserialize_keys<'de, D>(
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
        let bound = key.parse::<u64>().context(error::BadBound { bound_str: key })?;
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