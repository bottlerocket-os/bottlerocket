/// The updates settings can be used to configure settings related to updates, e.g. the
/// seed that determines in which wave the instance will update, etc.
pub mod generate;

use bottlerocket_settings_sdk::{GenerateResult, SettingsModel};
use model_derive::model;
use modeled_types::{FriendlyVersion, Url};
use std::convert::Infallible;

#[model(impl_default = true)]
pub struct UpdatesSettingsV1 {
    metadata_base_url: Url,
    targets_base_url: Url,
    seed: u32,
    // Version to update to when updating via the API.
    version_lock: FriendlyVersion,
    ignore_waves: bool,
}

type Result<T> = std::result::Result<T, Infallible>;

impl SettingsModel for UpdatesSettingsV1 {
    type PartialKind = Self;
    type ErrorKind = Infallible;

    fn get_version() -> &'static str {
        "v1"
    }

    fn set(_current_value: Option<Self>, _target: Self) -> Result<()> {
        // allow anything that parses as UpdatesSettingsV1
        Ok(())
    }

    fn generate(
        existing_partial: Option<Self::PartialKind>,
        _dependent_settings: Option<serde_json::Value>,
    ) -> Result<GenerateResult<Self::PartialKind, Self>> {
        let partial = existing_partial.unwrap_or_default();

        Ok(GenerateResult::Complete(UpdatesSettingsV1 {
            seed: Some(partial.seed.unwrap_or_else(generate::generate_seed)),
            ..partial
        }))
    }

    fn validate(_value: Self, _validated_settings: Option<serde_json::Value>) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_generate_updates() {
        if let GenerateResult::Complete(generated_settings) =
            UpdatesSettingsV1::generate(None, None).unwrap()
        {
            assert!(generated_settings.seed.unwrap() < 2048);
            assert!(generated_settings.metadata_base_url.is_none());
            assert!(generated_settings.targets_base_url.is_none());
            assert!(generated_settings.version_lock.is_none());
            assert!(generated_settings.ignore_waves.is_none());
        } else {
            panic!("generate() should return GenerateResult::Complete")
        }
    }

    #[test]
    fn test_serde_updates() {
        let test_json = r#"{
            "metadata-base-url": "https://example.net",
            "targets-base-url": "https://example.net",
            "seed": 1,
            "version-lock": "latest",
            "ignore-waves": false
        }"#;

        let updates: UpdatesSettingsV1 = serde_json::from_str(test_json).unwrap();

        assert_eq!(
            updates,
            UpdatesSettingsV1 {
                metadata_base_url: Some(Url::try_from("https://example.net").unwrap()),
                targets_base_url: Some(Url::try_from("https://example.net").unwrap()),
                seed: Some(1),
                version_lock: Some(FriendlyVersion::try_from("latest").unwrap()),
                ignore_waves: Some(false),
            }
        );
    }
}
