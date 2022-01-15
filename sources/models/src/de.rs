use crate::RegistryMirror;
use serde::de::value::SeqAccessDeserializer;
use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt::Formatter;

// Our standard representation of registry mirrors is a `Vec` of `RegistryMirror`; for backward compatibility, we also allow a `HashMap` of registry to endpoints.
pub(crate) fn deserialize_mirrors<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<RegistryMirror>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct TableOrArray;

    impl<'de> Visitor<'de> for TableOrArray {
        type Value = Option<Vec<RegistryMirror>>;

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
                vec.push(RegistryMirror {
                    registry: Some(k),
                    endpoint: Some(v),
                });
            }
            Ok(Some(vec))
        }
    }
    deserializer.deserialize_any(TableOrArray)
}

#[cfg(test)]
mod mirrors_tests {
    use crate::RegistrySettings;
    static TEST_MIRRORS_ARRAY: &str = include_str!("../tests/data/mirrors-array");
    static TEST_MIRRORS_TABLE: &str = include_str!("../tests/data/mirrors-table");

    #[test]
    fn registry_mirrors_array_representation() {
        assert!(toml::from_str::<RegistrySettings>(TEST_MIRRORS_ARRAY).is_ok());
    }

    #[test]
    fn registry_mirrors_table_representation() {
        assert!(toml::from_str::<RegistrySettings>(TEST_MIRRORS_TABLE).is_ok());
    }

    #[test]
    fn registry_mirrors_none_representation() {
        let registry_settings = toml::from_str::<RegistrySettings>("").unwrap();
        assert!(registry_settings.mirrors.is_none());
    }

    #[test]
    fn representation_equal() {
        assert_eq!(
            toml::from_str::<RegistrySettings>(TEST_MIRRORS_TABLE).unwrap(),
            toml::from_str::<RegistrySettings>(TEST_MIRRORS_ARRAY).unwrap()
        );
    }
}
