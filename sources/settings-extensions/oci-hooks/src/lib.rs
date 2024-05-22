/// Settings related to host-provided OCI Hooks
use bottlerocket_settings_sdk::{GenerateResult, SettingsModel};
use model_derive::model;
use std::convert::Infallible;

/// The log4j hotpatch functionality is no longer included in Bottlerocket as of v1.15.0.
/// The setting still exists for backwards compatibility.
#[model(impl_default = true)]
pub struct OciHooksSettingsV1 {
    log4j_hotpatch_enabled: bool,
}

type Result<T> = std::result::Result<T, Infallible>;

impl SettingsModel for OciHooksSettingsV1 {
    type PartialKind = Self;
    type ErrorKind = Infallible;

    fn get_version() -> &'static str {
        "v1"
    }

    fn set(_current_value: Option<Self>, _target: Self) -> Result<()> {
        // Set anything that can be parsed as OciHooksSettingsV1.
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
        // OciHooksSettingsV1 is validated during deserialization.
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_generate_oci_hooks() {
        assert_eq!(
            OciHooksSettingsV1::generate(None, None).unwrap(),
            GenerateResult::Complete(OciHooksSettingsV1 {
                log4j_hotpatch_enabled: None,
            })
        )
    }

    #[test]
    fn test_serde_oci_hooks() {
        let test_json = r#"{"log4j-hotpatch-enabled":true}"#;

        let oci_hooks: OciHooksSettingsV1 = serde_json::from_str(test_json).unwrap();
        assert_eq!(
            oci_hooks,
            OciHooksSettingsV1 {
                log4j_hotpatch_enabled: Some(true),
            }
        );

        let results = serde_json::to_string(&oci_hooks).unwrap();
        assert_eq!(results, test_json);
    }
}
