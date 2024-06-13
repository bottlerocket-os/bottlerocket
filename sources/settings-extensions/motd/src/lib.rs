/// The motd setting is used to set the "message of the day" that is shown to users when logging
/// into the Bottlerocket control container.
use bottlerocket_settings_sdk::{GenerateResult, LinearlyMigrateable, NoMigration, SettingsModel};
use std::convert::Infallible;
use string_impls_for::string_impls_for;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MotdV1 {
    inner: String,
}

type Result<T> = std::result::Result<T, Infallible>;

impl SettingsModel for MotdV1 {
    /// We only have one value, so there's no such thing as a partial
    type PartialKind = Self;
    type ErrorKind = Infallible;

    fn get_version() -> &'static str {
        "v1"
    }

    fn set(_current_value: Option<Self>, _target: Self) -> Result<()> {
        // Allow anything that parses as MotdV1
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
        // No need to do any additional validation, any MotdV1 is acceptable
        Ok(())
    }
}

impl LinearlyMigrateable for MotdV1 {
    type ForwardMigrationTarget = NoMigration;
    type BackwardMigrationTarget = NoMigration;

    fn migrate_forward(&self) -> Result<Self::ForwardMigrationTarget> {
        NoMigration::no_defined_migration()
    }

    fn migrate_backward(&self) -> Result<Self::BackwardMigrationTarget> {
        NoMigration::no_defined_migration()
    }
}

impl TryFrom<&str> for MotdV1 {
    type Error = Infallible;

    fn try_from(input: &str) -> Result<Self> {
        Ok(MotdV1 {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(MotdV1, "MotdV1");

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_generate_motd() {
        assert_eq!(
            MotdV1::generate(None, None),
            Ok(GenerateResult::Complete(MotdV1 {
                inner: "".to_string()
            }))
        )
    }

    #[test]
    fn test_serde_motd() {
        let test_json = r#""This is a motd""#;

        let motd: MotdV1 = serde_json::from_str(test_json).unwrap();
        assert_eq!(motd.inner, "This is a motd".to_string());

        let results = serde_json::to_string(&motd).unwrap();
        assert_eq!(results, test_json);
    }
}
