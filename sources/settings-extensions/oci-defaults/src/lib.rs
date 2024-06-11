/// Settings related to orchestrated containers for overriding the OCI runtime spec defaults
mod de;

use crate::de::deserialize_limit;
use bottlerocket_settings_sdk::{GenerateResult, SettingsModel};
use model_derive::model;
use modeled_types::{OciDefaultsCapability, OciDefaultsResourceLimitType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;

///// OCI defaults specifies the default values that will be used in cri-base-json.
#[model(impl_default = true)]
struct OciDefaultsV1 {
    capabilities: HashMap<OciDefaultsCapability, bool>,
    resource_limits: HashMap<OciDefaultsResourceLimitType, OciDefaultsResourceLimitV1>,
}

///// The hard and soft limit values for an OCI defaults resource limit.
#[model(add_option = false)]
#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, Ord, PartialOrd, PartialEq)]
struct OciDefaultsResourceLimitV1 {
    #[serde(deserialize_with = "deserialize_limit")]
    hard_limit: i64,
    #[serde(deserialize_with = "deserialize_limit")]
    soft_limit: i64,
}

type Result<T> = std::result::Result<T, Infallible>;

impl SettingsModel for OciDefaultsV1 {
    type PartialKind = Self;
    type ErrorKind = Infallible;

    fn get_version() -> &'static str {
        "v1"
    }

    fn set(_current_value: Option<Self>, _target: Self) -> Result<()> {
        // Set anything that can be parsed as OciDefaultsV1.
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
        // OciDefaultsV1 is validated during deserialization.
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_generate_oci_defaults() {
        assert_eq!(
            OciDefaultsV1::generate(None, None),
            Ok(GenerateResult::Complete(OciDefaultsV1 {
                capabilities: None,
                resource_limits: None,
            }))
        )
    }

    #[test]
    fn test_serde_oci_defaults() {
        let test_json = json!({
            "capabilities": {
                "sys-admin": true,
                "net-admin": false
            },
            "resource-limits": {
                "max-cpu-time": {
                    "hard-limit": 1000,
                    "soft-limit": 500
                }
            }
        });

        let test_json_str = test_json.to_string();

        let oci_defaults: OciDefaultsV1 = serde_json::from_str(&test_json_str).unwrap();

        let mut expected_capabilities = HashMap::new();
        expected_capabilities.insert(OciDefaultsCapability::SysAdmin, true);
        expected_capabilities.insert(OciDefaultsCapability::NetAdmin, false);

        let mut expected_resource_limits = HashMap::new();
        expected_resource_limits.insert(
            OciDefaultsResourceLimitType::MaxCpuTime,
            OciDefaultsResourceLimitV1 {
                hard_limit: 1000,
                soft_limit: 500,
            },
        );

        assert_eq!(
            oci_defaults,
            OciDefaultsV1 {
                capabilities: Some(expected_capabilities),
                resource_limits: Some(expected_resource_limits),
            }
        );

        let serialized_json: serde_json::Value = serde_json::to_string(&oci_defaults)
            .map(|s| serde_json::from_str(&s).unwrap())
            .unwrap();

        assert_eq!(serialized_json, test_json);
    }
}
