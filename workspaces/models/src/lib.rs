/*!
# API models

Thar has different variants supporting different features and use cases.
Each variant has its own set of software, and therefore needs its own configuration.
We support having an API model for each variant to support these different configurations.

Each model defines a top-level `Settings` structure.
It can use pre-defined structures inside, or custom ones as needed.

This `Settings` essentially becomes the schema for the variant's data store.
`apiserver::datastore` offers serialization and deserialization modules that make it easy to map between Rust types and the data store, and thus, all inputs and outputs are type-checked.

At the field level, standard Rust types can be used, or ["modeled types"](src/modeled_types) that add input validation.

## aws-k8s: Kubernetes

* [Model](src/aws-k8s/mod.rs)
* [Defaults](src/aws-k8s/defaults.toml)

## aws-dev: Development build

* [Model](src/aws-dev/mod.rs)
* [Defaults](src/aws-dev/defaults.toml)

# This directory

We use `build.rs` to symlink the proper API model source code for Cargo to build.
We determine the "proper" model by using the `VARIANT` environment variable.

If a developer is doing a local `cargo build`, they need to set `VARIANT`.

When building with the Thar build system, `VARIANT` is based on `BUILDSYS_VARIANT` from the top-level `Makefile.toml`, which can be overridden on the command line with `cargo make -e BUILDSYS_VARIANT=bla`.

Note: when building with the build system, we can't create the symlink in the source directory during a build - the directories are owned by `root`, but we're `builder`.
We can't use a read/write bind mount with current Docker syntax.
To get around this, in the top-level `Dockerfile`, we mount a "cache" directory at `src/variant` that we can modify, and create a `current` symlink inside.
The code in `src/lib.rs` then imports the requested model using `variant/current`.

Note: for the same reason, we symlink `variant/mod.rs` to `variant_mod.rs`.
Rust needs a `mod.rs` file to understand that a directory is part of the module structure, so we have to have `variant/mod.rs`.
`variant/` is the cache mount that starts empty, so we have to store the file elsewhere and link it in.

Note: all models share the same `Cargo.toml`.
*/

// "Modeled types" are types with special ser/de behavior used for validation.
pub mod modeled_types;

// The "variant" module is just a directory where we symlink in the user's requested build
// variant; each variant defines a top-level Settings structure and we re-export the current one.
mod variant;
pub use variant::Settings;

// Below, we define common structures used in the API surface; specific variants build a Settings
// structure based on these, and that's what gets exposed via the API.  (Specific variants' models
// are in subdirectories and linked into place by build.rs at variant/current.)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::Ipv4Addr;

use crate::modeled_types::{
    KubernetesClusterName, KubernetesLabelKey, KubernetesLabelValue, KubernetesTaintValue,
    SingleLineString, Url, ValidBase64,
};

// Note: fields are marked with skip_serializing_if=Option::is_none so that settings GETs don't
// show field=null for everything that isn't set in the relevant group of settings.

// Kubernetes related settings. The dynamic settings are retrieved from
// IMDS via Sundog's child "Pluto".
#[rustfmt::skip]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct KubernetesSettings {
    // Settings we require the user to specify, likely via user data.

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster_name: Option<KubernetesClusterName>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster_certificate: Option<ValidBase64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_server: Option<Url>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_labels: Option<HashMap<KubernetesLabelKey, KubernetesLabelValue>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_taints: Option<HashMap<KubernetesLabelKey, KubernetesTaintValue>>,

    // Dynamic settings.

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_pods: Option<SingleLineString>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster_dns_ip: Option<Ipv4Addr>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_ip: Option<Ipv4Addr>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub pod_infra_container_image: Option<SingleLineString>,
}

// Updog settings. Taken from userdata. The 'seed' setting is generated
// by the "Bork" settings generator at runtime.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct UpdatesSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_base_url: Option<Url>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_base_url: Option<Url>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u32>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct ContainerImage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Url>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub superpowered: Option<bool>,
}

// NTP settings
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct NtpSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_servers: Option<Vec<Url>>,
}

///// Internal services

// Note: Top-level objects that get returned from the API should have a serde "rename" attribute
// matching the struct name, but in kebab-case, e.g. ConfigurationFiles -> "configuration-files".
// This lets it match the datastore name.
// Objects that live inside those top-level objects, e.g. Service lives in Services, should have
// rename="" so they don't add an extra prefix to the datastore path that doesn't actually exist.
// This is important because we have APIs that can return those sub-structures directly.

pub type Services = HashMap<String, Service>;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename = "", rename_all = "kebab-case")]
pub struct Service {
    pub configuration_files: Vec<SingleLineString>,
    pub restart_commands: Vec<String>,
}

pub type ConfigurationFiles = HashMap<String, ConfigurationFile>;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename = "", rename_all = "kebab-case")]
pub struct ConfigurationFile {
    pub path: SingleLineString,
    pub template_path: SingleLineString,
}

///// Metadata

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename = "metadata", rename_all = "kebab-case")]
pub struct Metadata {
    pub key: SingleLineString,
    pub md: SingleLineString,
    pub val: toml::Value,
}
