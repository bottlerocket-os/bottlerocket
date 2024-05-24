/// Settings related to Amazon ECS
use bottlerocket_settings_sdk::{GenerateResult, SettingsModel};
use model_derive::model;
use modeled_types::{
    ECSAgentImagePullBehavior, ECSAgentLogLevel, ECSAttributeKey, ECSAttributeValue,
    ECSDurationValue, SingleLineString,
};
use std::{collections::HashMap, convert::Infallible};

#[model(impl_default = true)]
pub struct ECSSettingsV1 {
    cluster: String,
    instance_attributes: HashMap<ECSAttributeKey, ECSAttributeValue>,
    allow_privileged_containers: bool,
    logging_drivers: Vec<SingleLineString>,
    loglevel: ECSAgentLogLevel,
    enable_spot_instance_draining: bool,
    image_pull_behavior: ECSAgentImagePullBehavior,
    container_stop_timeout: ECSDurationValue,
    task_cleanup_wait: ECSDurationValue,
    metadata_service_rps: i64,
    metadata_service_burst: i64,
    reserved_memory: u16,
    image_cleanup_wait: ECSDurationValue,
    image_cleanup_delete_per_cycle: i64,
    image_cleanup_enabled: bool,
    image_cleanup_age: ECSDurationValue,
    backend_host: String,
    awsvpc_block_imds: bool,
    enable_container_metadata: bool,
}

type Result<T> = std::result::Result<T, Infallible>;

impl SettingsModel for ECSSettingsV1 {
    type PartialKind = Self;
    type ErrorKind = Infallible;

    fn get_version() -> &'static str {
        "v1"
    }

    fn set(_current_value: Option<Self>, _target: Self) -> Result<()> {
        // Set anything that can be parsed as ECSSettingsV1.
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
        // ECSSettingsV1 is validated during deserialization.
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_generate_ecs_settings() {
        assert_eq!(
            ECSSettingsV1::generate(None, None),
            Ok(GenerateResult::Complete(ECSSettingsV1 {
                cluster: None,
                instance_attributes: None,
                allow_privileged_containers: None,
                logging_drivers: None,
                loglevel: None,
                enable_spot_instance_draining: None,
                image_pull_behavior: None,
                container_stop_timeout: None,
                task_cleanup_wait: None,
                metadata_service_rps: None,
                metadata_service_burst: None,
                reserved_memory: None,
                image_cleanup_wait: None,
                image_cleanup_delete_per_cycle: None,
                image_cleanup_enabled: None,
                image_cleanup_age: None,
                backend_host: None,
                awsvpc_block_imds: None,
                enable_container_metadata: None,
            }))
        )
    }

    #[test]
    fn test_serde_ecs() {
        let test_json = json!({
            "cluster": "test-cluster",
            "instance-attributes": {
                "attribute1": "value1",
                "attribute2": "value2"
            },
            "allow-privileged-containers": true,
            "logging-drivers": ["json-file", "awslogs"],
            "loglevel": "info",
            "enable-spot-instance-draining": true,
            "image-pull-behavior": "always",
            "container-stop-timeout": "30s",
            "task-cleanup-wait": "1h",
            "metadata-service-rps": 50,
            "metadata-service-burst": 100,
            "reserved-memory": 512,
            "image-cleanup-wait": "1h",
            "image-cleanup-delete-per-cycle": 2,
            "image-cleanup-enabled": true,
            "image-cleanup-age": "1h",
            "backend-host": "ecs.us-east-1.amazonaws.com",
            "awsvpc-block-imds": true,
            "enable-container-metadata": true,
        });

        let test_json_str = test_json.to_string();

        let ecs_settings: ECSSettingsV1 = serde_json::from_str(&test_json_str).unwrap();

        let mut expected_instance_attributes: HashMap<ECSAttributeKey, ECSAttributeValue> =
            HashMap::new();
        expected_instance_attributes.insert(
            ECSAttributeKey::try_from("attribute1").unwrap(),
            ECSAttributeValue::try_from("value1").unwrap(),
        );
        expected_instance_attributes.insert(
            ECSAttributeKey::try_from("attribute2").unwrap(),
            ECSAttributeValue::try_from("value2").unwrap(),
        );

        let expected_ecs_settings = ECSSettingsV1 {
            cluster: Some("test-cluster".to_string()),
            instance_attributes: Some(expected_instance_attributes),
            allow_privileged_containers: Some(true),
            logging_drivers: Some(vec![
                SingleLineString::try_from("json-file").unwrap(),
                SingleLineString::try_from("awslogs").unwrap(),
            ]),
            loglevel: Some(ECSAgentLogLevel::Info),
            enable_spot_instance_draining: Some(true),
            image_pull_behavior: Some(ECSAgentImagePullBehavior::Always),
            container_stop_timeout: Some(ECSDurationValue::try_from("30s").unwrap()),
            task_cleanup_wait: Some(ECSDurationValue::try_from("1h").unwrap()),
            metadata_service_rps: Some(50),
            metadata_service_burst: Some(100),
            reserved_memory: Some(512),
            image_cleanup_wait: Some(ECSDurationValue::try_from("1h").unwrap()),
            image_cleanup_delete_per_cycle: Some(2),
            image_cleanup_enabled: Some(true),
            image_cleanup_age: Some(ECSDurationValue::try_from("1h").unwrap()),
            backend_host: Some("ecs.us-east-1.amazonaws.com".to_string()),
            awsvpc_block_imds: Some(true),
            enable_container_metadata: Some(true),
        };

        assert_eq!(ecs_settings, expected_ecs_settings);

        let serialized_json: serde_json::Value = serde_json::to_string(&ecs_settings)
            .map(|s| serde_json::from_str(&s).unwrap())
            .unwrap();

        assert_eq!(serialized_json, test_json);
    }
}
