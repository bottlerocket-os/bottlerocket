/// Settings related to auto scaling groups.
use bottlerocket_settings_sdk::{GenerateResult, SettingsModel};
use model_derive::model;
use std::convert::Infallible;

#[model(impl_default = true)]
pub struct AutoScalingSettingsV1 {
    should_wait: bool,
}

type Result<T> = std::result::Result<T, Infallible>;

impl SettingsModel for AutoScalingSettingsV1 {
    type PartialKind = Self;
    type ErrorKind = Infallible;

    fn get_version() -> &'static str {
        "v1"
    }

    fn set(_current_value: Option<Self>, _target: Self) -> Result<()> {
        // Set anything that can be parsed as AutoScalingSettingsV1.
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
        // AutoScalingSettingsV1 is validated during deserialization.
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_generate_oci_hooks() {
        assert_eq!(
            AutoScalingSettingsV1::generate(None, None).unwrap(),
            GenerateResult::Complete(AutoScalingSettingsV1 { should_wait: None })
        )
    }

    #[test]
    fn test_serde_oci_hooks() {
        let test_json = r#"{"should-wait":true}"#;

        let autoscaling: AutoScalingSettingsV1 = serde_json::from_str(test_json).unwrap();
        assert_eq!(
            autoscaling,
            AutoScalingSettingsV1 {
                should_wait: Some(true),
            }
        );

        let results = serde_json::to_string(&autoscaling).unwrap();
        assert_eq!(results, test_json);
    }
}
