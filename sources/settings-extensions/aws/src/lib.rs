/// The aws settings can be used to configure settings related to AWS
use bottlerocket_settings_sdk::{GenerateResult, SettingsModel};
use model_derive::model;
use modeled_types::{SingleLineString, ValidBase64};
use std::convert::Infallible;

// Platform-specific settings
#[model(impl_default = true)]
pub struct AwsSettingsV1 {
    region: SingleLineString,
    config: ValidBase64,
    credentials: ValidBase64,
    profile: SingleLineString,
}

type Result<T> = std::result::Result<T, Infallible>;

impl SettingsModel for AwsSettingsV1 {
    type PartialKind = Self;
    type ErrorKind = Infallible;

    fn get_version() -> &'static str {
        "v1"
    }

    fn set(_current_value: Option<Self>, _target: Self) -> Result<()> {
        // allow anything that parses as AwsSettingsV1
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
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_generate_aws() {
        let generated = AwsSettingsV1::generate(None, None).unwrap();
        assert_eq!(
            generated,
            GenerateResult::Complete(AwsSettingsV1 {
                region: None,
                config: None,
                credentials: None,
                profile: None,
            })
        )
    }

    #[test]
    fn test_serde_aws() {
        let test_json = r#"{
            "region": "us-east-1",
            "config": "Zm9vCg==",
            "credentials": "Zm9vCg==",
            "profile": "foo"
        }"#;

        let aws: AwsSettingsV1 = serde_json::from_str(test_json).unwrap();

        assert_eq!(
            aws,
            AwsSettingsV1 {
                region: Some(SingleLineString::try_from("us-east-1").unwrap()),
                config: Some(ValidBase64::try_from("Zm9vCg==").unwrap()),
                credentials: Some(ValidBase64::try_from("Zm9vCg==").unwrap()),
                profile: Some(SingleLineString::try_from("foo").unwrap()),
            }
        );
    }
}
