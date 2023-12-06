//!  Provides an in-memory `SettingsResolver`.
use super::SettingsResolver;
use crate::v2::ExtensionRequirement;
use async_trait::async_trait;

/// A `SettingsResolver` implementation for scenarios that use explicit in-memory data.
#[derive(Debug, Clone)]
pub struct JsonSettingsResolver {
    settings: serde_json::Value,
}

impl JsonSettingsResolver {
    pub fn new(settings: serde_json::Value) -> Self {
        Self { settings }
    }
}

#[async_trait]
impl SettingsResolver for JsonSettingsResolver {
    async fn fetch_settings<I>(
        &self,
        _extension_requirements: I,
    ) -> std::result::Result<serde_json::Value, Box<dyn std::error::Error>>
    where
        I: Iterator<Item = ExtensionRequirement> + Send,
    {
        Ok(self.settings.clone())
    }
}
