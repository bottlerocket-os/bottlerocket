/// host-containers settings allow users to configure multiple host containers
use bottlerocket_settings_sdk::{GenerateResult, SettingsModel};
use model_derive::model;
use modeled_types::{Identifier, Url, ValidBase64};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::convert::Infallible;

#[derive(Debug, Default, PartialEq)]
pub struct HostContainersSettingsV1 {
    pub host_containers: HashMap<Identifier, HostContainer>,
}

impl Serialize for HostContainersSettingsV1 {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.host_containers.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for HostContainersSettingsV1 {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let host_containers = HashMap::deserialize(deserializer)?;
        Ok(Self { host_containers })
    }
}

#[model(impl_default = true)]
struct HostContainer {
    source: Url,
    enabled: bool,
    superpowered: bool,
    user_data: ValidBase64,
}

type Result<T> = std::result::Result<T, Infallible>;

impl SettingsModel for HostContainersSettingsV1 {
    type PartialKind = Self;
    type ErrorKind = Infallible;

    fn get_version() -> &'static str {
        "v1"
    }

    fn set(_current_value: Option<Self>, _target: Self) -> Result<()> {
        // Set anything that can be parsed as HostContainersSettingsV1.
        Ok(())
    }

    fn generate(
        _existing_partial: Option<Self::PartialKind>,
        _dependent_settings: Option<serde_json::Value>,
    ) -> Result<GenerateResult<Self::PartialKind, Self>> {
        Ok(GenerateResult::Complete(HostContainersSettingsV1 {
            host_containers: HashMap::new(),
        }))
    }

    fn validate(_value: Self, _validated_settings: Option<serde_json::Value>) -> Result<()> {
        // HostContainersSettingsV1 is validated during deserialization.
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_generate_host_containers() {
        let generated = HostContainersSettingsV1::generate(None, None).unwrap();

        assert_eq!(
            generated,
            GenerateResult::Complete(HostContainersSettingsV1 {
                host_containers: HashMap::new(),
            })
        )
    }

    #[test]
    fn test_serde_host_containers() {
        let input_json = r#"{
            "foo": {
                "source": "public.ecr.aws/example/example",
                "enabled": true,
                "superpowered": true,
                "user-data": "Zm9vCg=="
            }
        }"#;

        let host_containers: HostContainersSettingsV1 = serde_json::from_str(input_json).unwrap();

        let mut expected_host_containers: HashMap<Identifier, HostContainer> = HashMap::new();
        expected_host_containers.insert(
            Identifier::try_from("foo").unwrap(),
            HostContainer {
                source: Some(Url::try_from("public.ecr.aws/example/example").unwrap()),
                enabled: Some(true),
                superpowered: Some(true),
                user_data: Some(ValidBase64::try_from("Zm9vCg==").unwrap()),
            },
        );

        assert_eq!(
            host_containers,
            HostContainersSettingsV1 {
                host_containers: expected_host_containers,
            }
        );
    }
}
