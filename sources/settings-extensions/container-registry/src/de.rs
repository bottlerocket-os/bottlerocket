use crate::RegistryMirrorV1;
use serde::de::value::SeqAccessDeserializer;
use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt::Formatter;

// Our standard representation of registry mirrors is a `Vec` of `RegistryMirror`; for backward compatibility, we also allow a `HashMap` of registry to endpoints.
pub(crate) fn deserialize_mirrors<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<RegistryMirrorV1>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct TableOrArray;

    impl<'de> Visitor<'de> for TableOrArray {
        type Value = Option<Vec<RegistryMirrorV1>>;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("TOML array or TOML table")
        }

        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            Ok(Some(Deserialize::deserialize(SeqAccessDeserializer::new(
                seq,
            ))?))
        }

        fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some((k, v)) = map.next_entry()? {
                vec.push(RegistryMirrorV1 {
                    registry: Some(k),
                    endpoint: Some(v),
                });
            }
            Ok(Some(vec))
        }
    }
    deserializer.deserialize_any(TableOrArray)
}
