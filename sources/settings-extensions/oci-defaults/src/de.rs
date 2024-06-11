use serde::de::Error;
use serde::{Deserialize, Deserializer};

/// This specifies that any non negative i64 integer, -1, and "unlimited"
/// are the valid resource-limits. The hard-limit set to "unlimited" or -1
/// and soft-limit set to "unlimited" or -1 are converted to u64::MAX in
/// the spec file for the container runtime which ultimately represents
/// unlimited for that resource
pub(crate) fn deserialize_limit<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt64 {
        String(String),
        Int(i64),
    }

    match StringOrInt64::deserialize(deserializer)? {
        StringOrInt64::String(s) => {
            if s == "unlimited" {
                Ok(-1)
            } else {
                Err(Error::custom(format!(
                    "Invalid rlimit {}, expected -1 to {} or \"unlimited\"",
                    s,
                    i64::MAX
                )))
            }
        }
        StringOrInt64::Int(i) => {
            if (-1..=i64::MAX).contains(&i) {
                Ok(i)
            } else {
                Err(Error::custom(format!(
                    "Invalid rlimit {}, expected -1 to {} or \"unlimited\"",
                    i,
                    i64::MAX
                )))
            }
        }
    }
}

#[cfg(test)]
mod oci_default_resource_limit_tests {
    use crate::OciDefaultsResourceLimitV1;

    #[test]
    fn valid_any_integer_i_64() {
        assert!(toml::from_str::<OciDefaultsResourceLimitV1>(
            r#"
          hard-limit = 200000
          soft-limit = 10000
       "#
        )
        .is_ok());
    }

    #[test]
    fn valid_string_unlimited() {
        assert!(toml::from_str::<OciDefaultsResourceLimitV1>(
            r#"
          hard-limit = 'unlimited'
          soft-limit = 10000
       "#
        )
        .is_ok());
    }

    #[test]
    fn valid_integer_i_64_max() {
        assert!(toml::from_str::<OciDefaultsResourceLimitV1>(
            r#"
          hard-limit = 9223372036854775807
          soft-limit = 10000
       "#
        )
        .is_ok());
    }

    #[test]
    fn valid_integer_minus_one() {
        assert!(toml::from_str::<OciDefaultsResourceLimitV1>(
            r#"
          hard-limit = -1
          soft-limit = 10000
       "#
        )
        .is_ok());
    }

    #[test]
    fn invalid_integer_greater_than_i_64_max() {
        assert!(toml::from_str::<OciDefaultsResourceLimitV1>(
            r#"
          hard-limit = 9223372036854775808
          soft-limit = 10000
       "#
        )
        .is_err());
    }

    #[test]
    fn invalid_minus_2() {
        assert!(toml::from_str::<OciDefaultsResourceLimitV1>(
            r#"
          hard-limit = -2
          soft-limit = 10000
       "#
        )
        .is_err());
    }

    #[test]
    fn invalid_string_abc() {
        assert!(toml::from_str::<OciDefaultsResourceLimitV1>(
            r#"
            hard-limit = 'abc'
            soft-limit = 10000
        "#
        )
        .is_err());
    }
}
