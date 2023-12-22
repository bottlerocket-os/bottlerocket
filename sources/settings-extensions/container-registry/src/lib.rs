/// The container-registry settings can be used to configure settings related to container
/// registries, including credentials for logging into a registry, or mirrors to use when
/// pulling from a registry.
mod de;

use crate::de::deserialize_mirrors;
use bottlerocket_settings_sdk::{GenerateResult, SettingsModel};
use model_derive::model;
use modeled_types::{SingleLineString, Url, ValidBase64};
use std::convert::Infallible;

#[model(impl_default = true)]
struct RegistryMirrorV1 {
    registry: SingleLineString,
    endpoint: Vec<Url>,
}

#[model(impl_default = true)]
struct RegistryCredentialV1 {
    registry: SingleLineString,
    username: SingleLineString,
    password: SingleLineString,
    // This is the base64 encoding of "username:password"
    auth: ValidBase64,
    identitytoken: SingleLineString,
}

#[model(impl_default = true)]
struct RegistrySettingsV1 {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_mirrors"
    )]
    mirrors: Vec<RegistryMirrorV1>,
    #[serde(alias = "creds", default, skip_serializing_if = "Option::is_none")]
    credentials: Vec<RegistryCredentialV1>,
}

type Result<T> = std::result::Result<T, Infallible>;

impl SettingsModel for RegistrySettingsV1 {
    type PartialKind = Self;
    type ErrorKind = Infallible;

    fn get_version() -> &'static str {
        "v1"
    }

    fn set(
        _current_value: Option<Self>,
        _target: Self,
    ) -> std::result::Result<(), Self::ErrorKind> {
        // Anything that correctly deserializes to RegistrySettingsV1 is ok
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

    fn validate(
        _value: Self,
        _validated_settings: Option<serde_json::Value>,
    ) -> std::result::Result<(), Self::ErrorKind> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_generate_container_registry_settings() {
        assert_eq!(
            RegistrySettingsV1::generate(None, None),
            Ok(GenerateResult::Complete(RegistrySettingsV1 {
                mirrors: None,
                credentials: None,
            }))
        )
    }

    #[test]
    fn test_serde_container_registry_with_mirrors() {
        let test_json =
            r#"{"mirrors": [{"registry": "foo", "endpoint": ["https://example.net"]}]}"#;

        let container_registry: RegistrySettingsV1 = serde_json::from_str(test_json).unwrap();
        let mirrors = container_registry.mirrors.unwrap();

        assert_eq!(mirrors.len(), 1);
        assert_eq!(
            mirrors[0].registry.clone().unwrap(),
            SingleLineString::try_from("foo").unwrap(),
        );
        assert_eq!(
            mirrors[0].endpoint.clone().unwrap(),
            vec!(Url::try_from("https://example.net").unwrap()),
        );
    }

    #[test]
    fn test_serde_container_registry_with_credentials() {
        let test_json = r#"{"credentials": [{"registry": "foo", "auth": "Ym90dGxlcm9ja2V0"}]}"#;

        let container_registry: RegistrySettingsV1 = serde_json::from_str(test_json).unwrap();
        let credentials = container_registry.credentials.unwrap();

        assert_eq!(credentials.len(), 1);
        assert_eq!(
            credentials[0].registry.clone().unwrap(),
            SingleLineString::try_from("foo").unwrap(),
        );
        assert_eq!(
            credentials[0].auth.clone().unwrap(),
            ValidBase64::try_from("Ym90dGxlcm9ja2V0").unwrap(),
        );
        assert!(credentials[0].username.is_none());
        assert!(credentials[0].password.is_none());
        assert!(credentials[0].identitytoken.is_none());
    }
}
