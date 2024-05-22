/// Settings related to custom DNS settings
use bottlerocket_settings_sdk::{GenerateResult, SettingsModel};
use model_derive::model;
use modeled_types::ValidLinuxHostname;
use std::convert::Infallible;
use std::net::IpAddr;

#[model(impl_default = true)]
pub struct DnsSettingsV1 {
    name_servers: Vec<IpAddr>,
    search_list: Vec<ValidLinuxHostname>,
}

type Result<T> = std::result::Result<T, Infallible>;

impl SettingsModel for DnsSettingsV1 {
    type PartialKind = Self;
    type ErrorKind = Infallible;

    fn get_version() -> &'static str {
        "v1"
    }

    fn set(_current_value: Option<Self>, _target: Self) -> Result<()> {
        // Set anything that can be parsed as DnsSettingsV1.
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
        // DnsSettingsV1 is validated during deserialization.
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_generate_dns_settings() {
        assert_eq!(
            DnsSettingsV1::generate(None, None),
            Ok(GenerateResult::Complete(DnsSettingsV1 {
                name_servers: None,
                search_list: None,
            }))
        )
    }

    #[test]
    fn test_serde_dns() {
        let test_json =
            r#"{"name-servers":["1.2.3.4","5.6.7.8"],"search-list":["foo.bar","baz.foo"]}"#;

        let dns: DnsSettingsV1 = serde_json::from_str(test_json).unwrap();
        assert_eq!(
            dns.name_servers.clone().unwrap(),
            vec!(
                IpAddr::from_str("1.2.3.4").unwrap(),
                IpAddr::from_str("5.6.7.8").unwrap(),
            )
        );
        assert_eq!(
            dns.search_list.clone().unwrap(),
            vec!(
                ValidLinuxHostname::try_from("foo.bar").unwrap(),
                ValidLinuxHostname::try_from("baz.foo").unwrap(),
            )
        );

        let results = serde_json::to_string(&dns).unwrap();
        assert_eq!(results, test_json);
    }
}
