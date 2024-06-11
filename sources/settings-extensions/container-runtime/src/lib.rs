///Settings related to Container Runtime
use bottlerocket_settings_sdk::{GenerateResult, SettingsModel};
use model_derive::model;
use std::convert::Infallible;

#[model(impl_default = true)]
pub struct ContainerRuntimeSettingsV1 {
    max_container_log_line_size: i32,
    max_concurrent_downloads: i32,
    enable_unprivileged_ports: bool,
    enable_unprivileged_icmp: bool,
}

type Result<T> = std::result::Result<T, Infallible>;

impl SettingsModel for ContainerRuntimeSettingsV1 {
    type PartialKind = Self;
    type ErrorKind = Infallible;

    fn get_version() -> &'static str {
        "v1"
    }

    fn set(_current_value: Option<Self>, _target: Self) -> Result<()> {
        // Set anything that can be parsed as ContainerRuntimeSettingsV1.
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
        // ContainerRuntimeSettingsV1 is validated during deserialization.
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_generate_container_runtime_settings() {
        assert_eq!(
            ContainerRuntimeSettingsV1::generate(None, None),
            Ok(GenerateResult::Complete(ContainerRuntimeSettingsV1 {
                max_container_log_line_size: None,
                max_concurrent_downloads: None,
                enable_unprivileged_ports: None,
                enable_unprivileged_icmp: None,
            }))
        )
    }

    #[test]
    fn test_serde_container_runtime() {
        let test_json = json!({
            "max-container-log-line-size": 1024,
            "max-concurrent-downloads": 5,
            "enable-unprivileged-ports": true,
            "enable-unprivileged-icmp": false
        });

        let test_json_str = test_json.to_string();

        let container_runtime_settings: ContainerRuntimeSettingsV1 =
            serde_json::from_str(&test_json_str).unwrap();

        assert_eq!(
            container_runtime_settings,
            ContainerRuntimeSettingsV1 {
                max_container_log_line_size: Some(1024),
                max_concurrent_downloads: Some(5),
                enable_unprivileged_ports: Some(true),
                enable_unprivileged_icmp: Some(false),
            }
        );

        let serialized_json: serde_json::Value = serde_json::to_string(&container_runtime_settings)
            .map(|s| serde_json::from_str(&s).unwrap())
            .unwrap();

        assert_eq!(serialized_json, test_json);
    }
}
