/// The motd setting is used to set the "message of the day" that is shown to users when logging
/// into the Bottlerocket control container.
use bottlerocket_settings_sdk::{GenerateResult, LinearlyMigrateable, NoMigration, SettingsModel};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct MotdV1(pub Option<String>);

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
            existing_partial.unwrap_or(MotdV1::default()),
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_generate_motd() {
        assert!(matches!(
            MotdV1::generate(None, None),
            Ok(GenerateResult::Complete(MotdV1(None)))
        ))
    }
}
