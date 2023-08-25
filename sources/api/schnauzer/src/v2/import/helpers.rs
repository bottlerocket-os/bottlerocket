//! This module contains the interface `HelperResolver` for registering Handlebars helpers via the Bottlerocket API.
//!
//! Currently the interface simply provides a static map of all existing helpers, but in the future this will be
//! modified to query the settings extensions on the host.
use super::as_std_err;
use crate::{helpers as handlebars_helpers, v2::ExtensionRequirement};
use async_trait::async_trait;
use handlebars::{Handlebars, HelperDef};
use maplit::hashmap;
use snafu::{ensure, OptionExt};
use std::collections::{HashMap, HashSet};

// Shorthand for HelperDef trait objects, which are required by the Handlebars library to register helpers.
macro_rules! helper {
    ( $var:expr ) => {
        Box::new($var) as Box<dyn HelperDef>
    };
}

// Type aliases to clarify the intent of string data.
type ExtensionName = &'static str;
type HelperName = &'static str;

/// This function provides the static map of existing handlebars helpers.
///
/// Extension-specific helpers must be requested via `[required-extensions]` in the template
/// frontmatter.
///
/// When settings extensions are merged into Bottlerocket, these helpers will be merged into
/// the extension that owns them.
///
/// Niether `const` nor `lazy_static` can be used to express this constant due to constraints of the `dyn HelperDef`
/// objects contained within.
fn all_helpers() -> HashMap<ExtensionName, HashMap<HelperName, Box<dyn HelperDef>>> {
    hashmap! {
        "aws" => hashmap! {
            "ecr-prefix" => helper!(handlebars_helpers::ecr_prefix),
        },

        "kubernetes" => hashmap! {
            "join_node_taints" => helper!(handlebars_helpers::join_node_taints),
            "kube_reserve_cpu" => helper!(handlebars_helpers::kube_reserve_cpu),
            "kube_reserve_memory" => helper!(handlebars_helpers::kube_reserve_memory),
            "pause-prefix" => helper!(handlebars_helpers::pause_prefix),
        },

        "network" => hashmap! {
            "localhost_aliases" => helper!(handlebars_helpers::localhost_aliases),
            "etc_hosts_entries" => helper!(handlebars_helpers::etc_hosts_entries),
            "host" => helper!(handlebars_helpers::host),
        },

        "updates" => hashmap! {
            "tuf-prefix" => helper!(handlebars_helpers::tuf_prefix),
            "metadata-prefix" => helper!(handlebars_helpers::metadata_prefix),
        },

        "oci-defaults" => hashmap! {
            "oci_defaults" => helper!(handlebars_helpers::oci_defaults)
        },

        // globally helpful helpers will be included in a null extension called "std"
        "std" => hashmap! {
            "any_enabled" => helper!(handlebars_helpers::any_enabled),
            "base64_decode" => helper!(handlebars_helpers::base64_decode),
            "default" => helper!(handlebars_helpers::default),
            "join_array" => helper!(handlebars_helpers::join_array),
            "join_map" => helper!(handlebars_helpers::join_map),
            "goarch" => helper!(handlebars_helpers::goarch),
        },
    }
}

/// An interface which abstracts away the registration of handlebars helpers for template rendering.
#[async_trait]
pub trait HelperResolver {
    /// Registers requested helpers from a specific setting extension for a template.
    async fn register_template_helpers<'a>(
        &self,
        template_registry: &mut Handlebars<'a>,
        extension_requirement: &ExtensionRequirement,
    ) -> std::result::Result<(), Box<dyn std::error::Error>>;
}

/// A `HelperResolver` implementation that uses a static map of helpers compiled into `schnauzer`.
#[derive(Debug, Clone, Default)]
pub struct StaticHelperResolver;

#[async_trait]
impl HelperResolver for StaticHelperResolver {
    /// Registers all handlebars helpers requested by a template.
    ///
    /// This currently uses a global list of all available helpers, but will be changed to only use helpers exposed by
    /// settings extensions.
    async fn register_template_helpers<'a>(
        &self,
        template_registry: &mut Handlebars<'a>,
        extension_requirement: &ExtensionRequirement,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        if extension_requirement.helpers.is_empty() {
            return Ok(());
        }

        Self::ensure_helpers_exist(extension_requirement).map_err(as_std_err)?;

        extension_requirement
            .helpers
            .iter()
            .for_each(|helper_name| {
                template_registry.register_helper(
                    helper_name.as_ref(),
                    Box::new(SettingExtensionTemplateHelper::new(
                        extension_requirement.name.to_string(),
                        extension_requirement.version.to_string(),
                        helper_name.to_string(),
                    )),
                )
            });

        Ok(())
    }
}

impl StaticHelperResolver {
    /// Ensure that a set of requested helpers exist in a given setting extension.
    fn ensure_helpers_exist(extension_requirement: &ExtensionRequirement) -> Result<()> {
        let setting_extension = &extension_requirement.name;
        let version = &extension_requirement.version;
        let requested_helpers: HashSet<_> = extension_requirement.helpers.iter().cloned().collect();

        if requested_helpers.is_empty() {
            return Ok(());
        }

        let existing_helpers = Self::fetch_helper_names_for_extension(setting_extension, version)?
            .into_iter()
            .collect();

        let missing_settings: Vec<_> = requested_helpers.difference(&existing_helpers).collect();
        ensure!(
            missing_settings.is_empty(),
            error::NoSuchHelpersSnafu {
                setting_extension: setting_extension.to_string(),
                extension_version: version.to_string(),
                helpers: missing_settings
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>(),
            }
        );
        Ok(())
    }

    /// Returns the names of all handlebars helpers associated with a given setting extension version.
    ///
    /// This currently uses a global list of all available helpers, but will be changed to only use helpers exposed by
    /// settings extensions.
    fn fetch_helper_names_for_extension(
        setting_extension: &str,
        _version: &str,
    ) -> Result<Vec<String>> {
        let helpers = all_helpers();
        Ok(helpers
            .get(setting_extension)
            .context(error::NoSuchSettingExtensionSnafu { setting_extension })?
            .keys()
            .map(|helper| helper.to_string())
            .collect())
    }
}

/// A "handle" for a handlebars helper designed to reach out to a specific setting extension for rendering.
pub(crate) struct SettingExtensionTemplateHelper {
    /// The setting extension that owns the helper.
    setting_extension: String,
    /// The version of the setting extension to use. Currently unused.
    _version: String,
    /// The name of the helper to call.
    helper_name: String,
}

impl SettingExtensionTemplateHelper {
    fn new(setting_extension: String, version: String, helper_name: String) -> Self {
        Self {
            setting_extension,
            _version: version,
            helper_name,
        }
    }
}

// The `HelperDef` implementation for `SettingExtensionTemplateHelper` currently just invokes the already-defined
// global helper. This is a temporary workaround until settings extensions are merged into Bottlerocket.
impl HelperDef for SettingExtensionTemplateHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &handlebars::Helper<'reg, 'rc>,
        r: &'reg Handlebars<'reg>,
        ctx: &'rc handlebars::Context,
        rc: &mut handlebars::RenderContext<'reg, 'rc>,
        out: &mut dyn handlebars::Output,
    ) -> handlebars::HelperResult {
        let available_helpers = all_helpers();
        let referenced_helper = available_helpers
            .get(self.setting_extension.as_str())
            .ok_or(handlebars::RenderError::new(format!(
                "Requested setting extension '{}' not found",
                self.setting_extension
            )))?
            .get(self.helper_name.as_str())
            .ok_or(handlebars::RenderError::new(format!(
                "Requested helper '{}' not found",
                self.helper_name
            )))?;
        referenced_helper.call(h, r, ctx, rc, out)
    }
}

pub mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub))]
    pub enum Error {
        #[snafu(display("No such setting extension '{}'", setting_extension))]
        NoSuchSettingExtension { setting_extension: String },

        #[snafu(display(
            "No such helpers defined for extension '{}' at version '{}': {:?}",
            setting_extension,
            extension_version,
            helpers
        ))]
        NoSuchHelpers {
            setting_extension: String,
            extension_version: String,
            helpers: Vec<String>,
        },
    }
}

pub use error::Error;
type Result<T> = std::result::Result<T, error::Error>;

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_ensure_helpers_exist() {
        let fail_cases = &[
            ("reticulator", "v1", vec!["reticulate"]),
            ("network", "v1", vec!["host", "made-up-helper"]),
        ];

        let success_cases = &[
            ("network", "v1", vec!["host"]),
            ("empty-helpers-succeeds", "v1", vec![]),
            ("kubernetes", "v1", vec!["pause-prefix"]),
        ];

        for (setting_name, version, helpers) in fail_cases.into_iter() {
            println!(
                "Checking {}@{}.{}",
                setting_name,
                version,
                helpers.join(",")
            );
            let extension_requirement = ExtensionRequirement {
                name: setting_name.to_string(),
                version: version.to_string(),
                helpers: helpers.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            };
            assert!(StaticHelperResolver::ensure_helpers_exist(&extension_requirement).is_err());
        }

        for (setting_name, version, helpers) in success_cases.into_iter() {
            println!(
                "Checking {}@{}.{}",
                setting_name,
                version,
                helpers.join(",")
            );
            let extension_requirement = ExtensionRequirement {
                name: setting_name.to_string(),
                version: version.to_string(),
                helpers: helpers.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            };
            assert!(StaticHelperResolver::ensure_helpers_exist(&extension_requirement).is_ok());
        }
    }

    /// This test should be removed once settings extensions are merged into Bottlerocket.
    #[test]
    fn test_fetch_helper_names_for_extension() {
        let test_cases = &[
            (
                "network",
                "v1",
                vec!["localhost_aliases", "etc_hosts_entries", "host"],
            ),
            (
                "kubernetes",
                "v1",
                vec![
                    "join_node_taints",
                    "kube_reserve_cpu",
                    "kube_reserve_memory",
                    "pause-prefix",
                ],
            ),
        ];

        for (extension_name, version, expected_helpers) in test_cases.into_iter() {
            assert_eq!(
                StaticHelperResolver::fetch_helper_names_for_extension(extension_name, version)
                    .unwrap()
                    .into_iter()
                    .collect::<HashSet<_>>(),
                expected_helpers
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<HashSet<_>>()
            );
        }
    }

    #[test]
    fn test_setting_extension_template_helper() {
        // Given a registry using the `SettingExtensionTemplateHelper` helper,
        // When a template is registered which invokes that helper,
        // Then the template will be appropriately rendered.
        let mut registry = Handlebars::new();
        registry.register_helper(
            "default",
            Box::new(SettingExtensionTemplateHelper::new(
                "std".to_string(),
                "v1".to_string(),
                "default".to_string(),
            )),
        );

        assert_eq!(
            registry
                .render_template(
                    "{{ default \"foo\" no.such.value }}",
                    &serde_json::json!({})
                )
                .unwrap(),
            "foo".to_string()
        );
    }
}
