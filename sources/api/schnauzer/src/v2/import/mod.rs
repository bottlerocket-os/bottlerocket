#[cfg(feature = "testfakes")]
pub mod fake;
pub mod helpers;
pub mod settings;

use std::path::PathBuf;

pub use helpers::{HelperResolver, StaticHelperResolver};
pub use settings::{BottlerocketSettingsResolver, SettingsResolver};

/// Used to register helpers and fetch settings during template rendering.
pub trait TemplateImporter {
    type SettingsResolver: SettingsResolver;
    type HelperResolver: HelperResolver;

    fn settings_resolver(&self) -> &Self::SettingsResolver;
    fn helper_resolver(&self) -> &Self::HelperResolver;
}

#[macro_export]
/// Implements the `TemplateImporter` trait for a type, given the resolver types used to fetch settings and helpers.
macro_rules! impl_template_importer {
    ($t:ty, $s:ty, $h:ty) => {
        impl $crate::v2::import::TemplateImporter for $t {
            type SettingsResolver = $s;
            type HelperResolver = $h;

            fn settings_resolver(&self) -> &Self::SettingsResolver {
                &self.settings_resolver
            }

            fn helper_resolver(&self) -> &Self::HelperResolver {
                &self.helper_resolver
            }
        }
    };
}

/// A `TemplateImporter` that uses the Bottlerocket API to fetch settings and helpers.
#[derive(Debug, Clone, Default)]
pub struct BottlerocketTemplateImporter {
    settings_resolver: BottlerocketSettingsResolver,
    helper_resolver: StaticHelperResolver,
}

impl BottlerocketTemplateImporter {
    pub fn new(api_socket: PathBuf) -> Self {
        Self {
            settings_resolver: BottlerocketSettingsResolver::new(api_socket),
            ..Default::default()
        }
    }
}

impl_template_importer!(
    BottlerocketTemplateImporter,
    BottlerocketSettingsResolver,
    StaticHelperResolver
);

/// Utility that Boxes an error type to be returned by a generic trait interface.
///
/// Intended to be called as e.g. `fallible().map_err(as_std_err)`
fn as_std_err<'a, E: std::error::Error + 'a>(err: E) -> Box<dyn std::error::Error + 'a> {
    Box::new(err) as Box<dyn std::error::Error>
}
