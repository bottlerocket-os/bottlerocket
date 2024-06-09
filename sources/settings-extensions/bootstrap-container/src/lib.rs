/// Settings related to bootstrap containers.
use bottlerocket_settings_sdk::{GenerateResult, SettingsModel};
use model_derive::model;
use modeled_types::{BootstrapContainerMode, Identifier, Url, ValidBase64};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{collections::HashMap, convert::Infallible};

#[derive(Debug, Default, PartialEq)]
pub struct BootstrapContainerSettingsV1 {
    pub bootstrap_containers: HashMap<Identifier, BootstrapContainer>,
}

impl Serialize for BootstrapContainerSettingsV1 {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.bootstrap_containers.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for BootstrapContainerSettingsV1 {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bootstrap_containers = HashMap::deserialize(deserializer)?;
        Ok(Self {
            bootstrap_containers,
        })
    }
}

#[model(impl_default = true)]
struct BootstrapContainer {
    source: Url,
    mode: BootstrapContainerMode,
    user_data: ValidBase64,
    essential: bool,
}

type Result<T> = std::result::Result<T, Infallible>;

impl SettingsModel for BootstrapContainerSettingsV1 {
    type PartialKind = Self;
    type ErrorKind = Infallible;

    fn get_version() -> &'static str {
        "v1"
    }

    fn set(_current_value: Option<Self>, _target: Self) -> Result<()> {
        // Set anything that parses as BootstrapContainerSettingsV1.
        Ok(())
    }

    fn generate(
        existing_partial: Option<Self::PartialKind>,
        _dependent_settings: Option<serde_json::Value>,
    ) -> Result<GenerateResult<Self::PartialKind, Self>> {
        Ok(GenerateResult::Complete(
            existing_partial.unwrap_or_default(),
        ))
    }

    fn validate(_value: Self, _validated_settings: Option<serde_json::Value>) -> Result<()> {
        // Validate anything that parses as BootstrapContainerSettingsV1.
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_generate_bootstarp_container_settings() {
        let generated = BootstrapContainerSettingsV1::generate(None, None).unwrap();

        assert_eq!(
            generated,
            GenerateResult::Complete(BootstrapContainerSettingsV1 {
                bootstrap_containers: HashMap::new(),
            })
        )
    }

    #[test]
    fn test_serde_bootstrap_container() {
        let test_json = json!({
            "mybootstrap": {
                "source": "uri.to.container.in.oci-compatible-registry.example.com/foo:1.0.0",
                "mode": "once",
                "user-data": "dXNlcmRhdGE=",
                "essential": true,
            }
        });

        let test_json_str = test_json.to_string();

        let bootstrap_containers: BootstrapContainerSettingsV1 =
            serde_json::from_str(&test_json_str).unwrap();

        let mut expected_bootstrap_container: HashMap<Identifier, BootstrapContainer> =
            HashMap::new();
        expected_bootstrap_container.insert(
            Identifier::try_from("mybootstrap").unwrap(),
            BootstrapContainer {
                source: Some(
                    Url::try_from(
                        "uri.to.container.in.oci-compatible-registry.example.com/foo:1.0.0",
                    )
                    .unwrap(),
                ),
                mode: Some(BootstrapContainerMode::try_from("once").unwrap()),
                user_data: Some(ValidBase64::try_from("dXNlcmRhdGE=").unwrap()),
                essential: Some(true),
            },
        );

        assert_eq!(
            bootstrap_containers,
            BootstrapContainerSettingsV1 {
                bootstrap_containers: expected_bootstrap_container
            }
        );

        let serialized_json: serde_json::Value = serde_json::to_string(&bootstrap_containers)
            .map(|s| serde_json::from_str(&s).unwrap())
            .unwrap();

        assert_eq!(serialized_json, test_json);
    }
}
