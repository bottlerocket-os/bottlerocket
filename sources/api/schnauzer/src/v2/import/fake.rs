//! Provides a helpful "fake" implementation of TemplateImporter to use in test scenarios.
use super::{HelperResolver, JsonSettingsResolver};
use crate::v2::ExtensionRequirement;
use async_trait::async_trait;
use handlebars::Handlebars;
pub struct FakeImporter {
    settings_resolver: JsonSettingsResolver,
    helper_resolver: FakeHelperResolver,
}

impl FakeImporter {
    pub fn new(settings: serde_json::Value, helpers: Vec<(&'static str, StaticHelper)>) -> Self {
        Self {
            settings_resolver: JsonSettingsResolver::new(settings),
            helper_resolver: FakeHelperResolver::new(helpers),
        }
    }
}

crate::impl_template_importer!(FakeImporter, FakeSettingsResolver, FakeHelperResolver);

pub type FakeSettingsResolver = JsonSettingsResolver;

/// A handlebars helper defined by a static function.
type StaticHelper = fn(
    &handlebars::Helper,
    &handlebars::Handlebars,
    &handlebars::Context,
    &mut handlebars::RenderContext<'_, '_>,
    &mut dyn handlebars::Output,
) -> handlebars::HelperResult;

pub struct FakeHelperResolver {
    helpers: Vec<(&'static str, StaticHelper)>,
}

impl FakeHelperResolver {
    pub fn new(helpers: Vec<(&'static str, StaticHelper)>) -> Self {
        Self { helpers }
    }
}

/// A `HelperResolver` implementation for test scenarios that use explicit in-memory data.
#[async_trait]
impl HelperResolver for FakeHelperResolver {
    async fn register_template_helpers<'a>(
        &self,
        template_registry: &mut Handlebars<'a>,
        _extension_requirement: &ExtensionRequirement,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        for (name, helper) in &self.helpers {
            template_registry.register_helper(name, Box::new(*helper));
        }
        Ok(())
    }
}
