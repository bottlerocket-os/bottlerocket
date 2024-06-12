/// Settings related to networking configuration.
use bottlerocket_settings_sdk::{GenerateResult, SettingsModel};
use model_derive::model;
use modeled_types::{EtcHostsEntries, SingleLineString, Url, ValidLinuxHostname};
use std::convert::Infallible;

#[model(impl_default = true)]
struct NetworkSettingsV1 {
    hostname: ValidLinuxHostname,
    hosts: EtcHostsEntries,
    https_proxy: Url,
    no_proxy: Vec<SingleLineString>,
}

type Result<T> = std::result::Result<T, Infallible>;

impl SettingsModel for NetworkSettingsV1 {
    type PartialKind = Self;
    type ErrorKind = Infallible;

    fn get_version() -> &'static str {
        "v1"
    }

    fn set(_current_value: Option<Self>, _target: Self) -> Result<()> {
        // Set anything that can be parsed as NetworkSettingsV1.
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
        // NetworkSettingsV1 is validated during deserialization.
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_generate_network_settings() {
        assert_eq!(
            NetworkSettingsV1::generate(None, None),
            Ok(GenerateResult::Complete(NetworkSettingsV1 {
                hostname: None,
                hosts: None,
                https_proxy: None,
                no_proxy: None,
            }))
        )
    }

    #[test]
    fn test_serde_network() {
        let test_json = r#"{
            "hostname": "foo",
            "hosts": [["127.0.0.1", ["localhost"]]],
            "https-proxy": "https://example.net",
            "no-proxy": ["foo"]
        }"#;

        let network: NetworkSettingsV1 = serde_json::from_str(test_json).unwrap();

        assert_eq!(
            network,
            NetworkSettingsV1 {
                hostname: Some(ValidLinuxHostname::try_from("foo").unwrap()),
                hosts: Some(
                    serde_json::from_str::<EtcHostsEntries>(r#"[["127.0.0.1", ["localhost"]]]"#)
                        .unwrap()
                ),
                https_proxy: Some(Url::try_from("https://example.net").unwrap()),
                no_proxy: Some(vec![SingleLineString::try_from("foo").unwrap()]),
            }
        );
    }
}
