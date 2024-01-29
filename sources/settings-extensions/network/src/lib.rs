/// The network settings will affect host service component's network behavior
pub mod generate;

use bottlerocket_settings_sdk::{GenerateResult, SettingsModel};
use error::GenerateHostnameSnafu;
use model_derive::model;
use modeled_types::{EtcHostsEntries, SingleLineString, Url, ValidLinuxHostname};
use snafu::ResultExt;

#[model(impl_default = true)]
struct NetworkSettingsV1 {
    hostname: ValidLinuxHostname,
    hosts: EtcHostsEntries,
    https_proxy: Url,
    // We allow some flexibility in NO_PROXY values because different services support different formats.
    no_proxy: Vec<SingleLineString>,
}

type Result<T> = std::result::Result<T, error::Error>;

impl SettingsModel for NetworkSettingsV1 {
    type PartialKind = Self;
    type ErrorKind = error::Error;

    fn get_version() -> &'static str {
        "v1"
    }

    fn set(_current_value: Option<Self>, _target: Self) -> Result<()> {
        // Anything that parses as NetworkSettingsV1 is ok
        Ok(())
    }

    fn generate(
        existing_partial: Option<Self::PartialKind>,
        _dependent_settings: Option<serde_json::Value>,
    ) -> Result<GenerateResult<Self::PartialKind, Self>> {
        let partial = existing_partial.unwrap_or_default();

        Ok(GenerateResult::Complete(NetworkSettingsV1 {
            hostname: Some(
                partial
                    .hostname
                    .unwrap_or(generate::generate_hostname().context(GenerateHostnameSnafu)?),
            ),
            ..partial
        }))
    }

    fn validate(_value: Self, _validated_settings: Option<serde_json::Value>) -> Result<()> {
        // This setting's field types handle validation during deserialization
        Ok(())
    }
}

mod error {
    use super::generate;
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub))]
    pub enum Error {
        #[snafu(display("Failed to generate hostname: {}", source))]
        GenerateHostname {
            source: generate::error::GenerateError,
        },
    }
}

#[cfg(test)]
mod test {
    use super::*;

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

    #[test]
    fn test_serde_network_invalid() {
        // invalid hostname
        let test_json = r#"{
            "hostname": "$@%!"
        }"#;
        let err = serde_json::from_str::<NetworkSettingsV1>(test_json).unwrap_err();
        // the "data" error type is returned from serde_json when the input is syntactically valid
        // JSON, but the type of a field is incorrect.
        assert!(err.is_data());

        // invalid /etc/hosts entry
        let test_json = r#"{
            "hosts": [["not_an_ip", ["foo"]]]
        }"#;
        let err = serde_json::from_str::<NetworkSettingsV1>(test_json).unwrap_err();
        assert!(err.is_data());

        // invalid proxy
        let test_json = r#"{
            "https-proxy": "not a url"
        }"#;
        let err = serde_json::from_str::<NetworkSettingsV1>(test_json).unwrap_err();
        assert!(err.is_data());

        // invalid no-proxy
        let test_json = r#"{
            "no-proxy": ["two\nlines"]
        }"#;
        let err = serde_json::from_str::<NetworkSettingsV1>(test_json).unwrap_err();
        assert!(err.is_data());
    }
}
