/*!
# API models

Bottlerocket has different variants supporting different features and use cases.
Each variant has its own set of software, and therefore needs its own configuration.
We support having an API model for each variant to support these different configurations.

The model here defines a top-level `Settings` structure, and delegates the actual implementation to a ["settings plugin"](https://github.com/bottlerocket-os/bottlerocket-settings-sdk/tree/develop/bottlerocket-settings-plugin).
Settings plugin are written in Rust as a "cdylib" crate, and loaded at runtime.

Each settings plugin must define its own private `Settings` structure.
It can use pre-defined structures inside, or custom ones as needed.

`apiserver::datastore` offers serialization and deserialization modules that make it easy to map between Rust types and the data store, and thus, all inputs and outputs are type-checked.

At the field level, standard Rust types can be used, or ["modeled types"](https://github.com/bottlerocket-os/bottlerocket-settings-sdk/tree/develop/bottlerocket-settings-models/modeled-types) that add input validation.

The `#[model]` attribute on Settings and its sub-structs reduces duplication and adds some required metadata; see [its docs](model-derive/) for details.
*/

// Types used to communicate between client and server for 'apiclient exec'.
pub mod exec;

use bottlerocket_release::BottlerocketRelease;
use bottlerocket_settings_models::model_derive::model;
use bottlerocket_settings_plugin::BottlerocketSettings;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use bottlerocket_settings_models::modeled_types::SingleLineString;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Settings {
    inner: BottlerocketSettings,
}

// This is the top-level model exposed by the API system. It contains the common sections for all
// variants.  This allows a single API call to retrieve everything the API system knows, which is
// useful as a check and also, for example, as a data source for templated configuration files.
#[model]
pub struct Model {
    settings: Settings,
    services: Services,
    configuration_files: ConfigurationFiles,
    os: BottlerocketRelease,
}

///// Internal services

// Note: Top-level objects that get returned from the API should have a "rename" attribute
// matching the struct name, but in kebab-case, e.g. ConfigurationFiles -> "configuration-files".
// This lets it match the datastore name.
// Objects that live inside those top-level objects, e.g. Service lives in Services, should have
// rename="" so they don't add an extra prefix to the datastore path that doesn't actually exist.
// This is important because we have APIs that can return those sub-structures directly.

pub type Services = HashMap<String, Service>;

#[model(add_option = false, rename = "")]
struct Service {
    configuration_files: Vec<SingleLineString>,
    restart_commands: Vec<String>,
}

pub type ConfigurationFiles = HashMap<String, ConfigurationFile>;

#[model(add_option = false, rename = "")]
struct ConfigurationFile {
    path: SingleLineString,
    template_path: SingleLineString,
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<String>,
}

///// Metadata

#[model(add_option = false, rename = "metadata")]
struct Metadata {
    key: SingleLineString,
    md: SingleLineString,
    val: toml::Value,
}

#[model(add_option = false)]
struct Report {
    name: String,
    description: String,
}
