use crate::{KubernetesLabelKey, KubernetesTaintValue, RegistryMirror};
use serde::de::value::SeqAccessDeserializer;
use serde::de::{Error, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::Formatter;
use toml::Value;

// Our standard representation of node-taints is a `HashMap` of label keys to a list of taint values and effects;
// for backward compatibility, we also allow a `HashMap` of label keys to a singular taint value/effect.
pub(crate) fn deserialize_node_taints<'de, D>(
    deserializer: D,
) -> Result<Option<HashMap<KubernetesLabelKey, Vec<KubernetesTaintValue>>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct NodeTaints;

    impl<'de> Visitor<'de> for NodeTaints {
        type Value = Option<HashMap<KubernetesLabelKey, Vec<KubernetesTaintValue>>>;
        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("TOML table")
        }

        fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut node_taints: HashMap<KubernetesLabelKey, Vec<KubernetesTaintValue>> =
                HashMap::new();
            while let Some((k, v)) = map.next_entry()? {
                match v {
                    // If we encounter a singular mapped value, convert it into a list of one
                    Value::String(taint_val) => {
                        node_taints.insert(
                            k,
                            vec![KubernetesTaintValue::try_from(taint_val)
                                .map_err(M::Error::custom)?],
                        );
                    }
                    // If we encounter a list of values, just insert it as is
                    Value::Array(taint_values) => {
                        let taint_values = taint_values
                            .iter()
                            .map(|v| v.to_owned().try_into().map_err(M::Error::custom))
                            .collect::<Result<Vec<KubernetesTaintValue>, _>>()?;
                        if taint_values.is_empty() {
                            return Err(M::Error::custom("empty taint value"));
                        }
                        node_taints.insert(k, taint_values);
                    }
                    _ => {
                        return Err(M::Error::custom("unsupported taint value type"));
                    }
                }
            }
            Ok(Some(node_taints))
        }
    }

    deserializer.deserialize_map(NodeTaints)
}

#[cfg(test)]
mod node_taint_tests {
    use crate::{KubernetesSettings, KubernetesTaintValue};
    use std::convert::TryFrom;
    static TEST_NODE_TAINT_LIST: &str = include_str!("../tests/data/node-taint-list-val");
    static TEST_NODE_TAINT_SINGLE: &str = include_str!("../tests/data/node-taint-single-val");
    static TEST_NODE_TAINT_EMPTY_LIST: &str = include_str!("../tests/data/node-taint-empty-list");

    #[test]
    fn node_taints_list_representation() {
        let k8s_settings = toml::from_str::<KubernetesSettings>(TEST_NODE_TAINT_LIST).unwrap();
        assert_eq!(
            k8s_settings
                .node_taints
                .as_ref()
                .unwrap()
                .get("key1")
                .unwrap()
                .to_owned(),
            vec![
                KubernetesTaintValue::try_from("value1:NoSchedule").unwrap(),
                KubernetesTaintValue::try_from("value1:NoExecute").unwrap()
            ]
        );
        assert_eq!(
            k8s_settings
                .node_taints
                .as_ref()
                .unwrap()
                .get("key2")
                .unwrap()
                .to_owned(),
            vec![KubernetesTaintValue::try_from("value2:NoSchedule").unwrap()]
        );
    }

    #[test]
    fn node_taint_single_representation() {
        let k8s_settings = toml::from_str::<KubernetesSettings>(TEST_NODE_TAINT_SINGLE).unwrap();
        assert_eq!(
            k8s_settings
                .node_taints
                .as_ref()
                .unwrap()
                .get("key1")
                .unwrap()
                .to_owned(),
            vec![KubernetesTaintValue::try_from("value1:NoSchedule").unwrap()]
        );
        assert_eq!(
            k8s_settings
                .node_taints
                .as_ref()
                .unwrap()
                .get("key2")
                .unwrap()
                .to_owned(),
            vec![KubernetesTaintValue::try_from("value2:NoExecute").unwrap()]
        );
    }

    #[test]
    fn node_taint_none_representation() {
        let k8s_settings = toml::from_str::<KubernetesSettings>("").unwrap();
        assert!(k8s_settings.node_taints.is_none());
    }

    #[test]
    fn node_taint_empty_list() {
        assert!(toml::from_str::<KubernetesSettings>(TEST_NODE_TAINT_EMPTY_LIST).is_err());
    }
}

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
