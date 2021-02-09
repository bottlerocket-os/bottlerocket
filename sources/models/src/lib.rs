/*!
# API models

Bottlerocket has different variants supporting different features and use cases.
Each variant has its own set of software, and therefore needs its own configuration.
We support having an API model for each variant to support these different configurations.

Each model defines a top-level `Settings` structure.
It can use pre-defined structures inside, or custom ones as needed.

This `Settings` essentially becomes the schema for the variant's data store.
`apiserver::datastore` offers serialization and deserialization modules that make it easy to map between Rust types and the data store, and thus, all inputs and outputs are type-checked.

At the field level, standard Rust types can be used, or ["modeled types"](src/modeled_types) that add input validation.

Default values are specified in .toml files in each variant's `defaults.d` directory under [src](src).
(For example, see the [aws-ecs-1 defaults](src/aws-ecs-1/defaults.d/).)
Entries are sorted by filename, and later entries take precedence.

The `#[model]` attribute on Settings and its sub-structs reduces duplication and adds some required metadata; see [its docs](model-derive/) for details.

## aws-k8s-1.15: Kubernetes 1.15

* [Model](src/aws-k8s-1.15/mod.rs)
* [Default settings](src/aws-k8s-1.15/defaults.d/)

## aws-k8s-1.16: Kubernetes 1.16

* [Model](src/aws-k8s-1.16/mod.rs)
* [Default settings](src/aws-k8s-1.16/defaults.d/)

## aws-k8s-1.17: Kubernetes 1.17

* [Model](src/aws-k8s-1.17/mod.rs)
* [Default settings](src/aws-k8s-1.17/defaults.d/)

## aws-k8s-1.18: Kubernetes 1.18

* [Model](src/aws-k8s-1.18/mod.rs)
* [Default settings](src/aws-k8s-1.18/defaults.d/)

## aws-k8s-1.19: Kubernetes 1.19

* [Model](src/aws-k8s-1.19/mod.rs)
* [Default settings](src/aws-k8s-1.19/defaults.d/)

## aws-ecs-1: Amazon ECS

* [Model](src/aws-ecs-1/mod.rs)
* [Default settings](src/aws-ecs-1/defaults.d/)

## aws-dev: Development build

* [Model](src/aws-dev/mod.rs)
* [Default settings](src/aws-dev/defaults.d/)

# This directory

We use `build.rs` to symlink the proper API model source code for Cargo to build.
We determine the "proper" model by using the `VARIANT` environment variable.

If a developer is doing a local `cargo build`, they need to set `VARIANT`.

When building with the Bottlerocket build system, `VARIANT` is based on `BUILDSYS_VARIANT` from the top-level `Makefile.toml`, which can be overridden on the command line with `cargo make -e BUILDSYS_VARIANT=bla`.

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
pub use variant::*;

// Below, we define common structures used in the API surface; specific variants build a Settings
// structure based on these, and that's what gets exposed via the API.  (Specific variants' models
// are in subdirectories and linked into place by build.rs at variant/current.)

use model_derive::model;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::Ipv4Addr;

use crate::modeled_types::{
    DNSDomain, ECSAgentLogLevel, ECSAttributeKey, ECSAttributeValue, FriendlyVersion, Identifier,
    KubernetesClusterName, KubernetesLabelKey, KubernetesLabelValue, KubernetesTaintValue,
    Lockdown, SingleLineString, SysctlKey, Url, ValidBase64,
};

// Kubernetes static pod manifest settings
#[model]
struct StaticPod {
    enabled: bool,
    manifest: ValidBase64,
}

// Kubernetes related settings. The dynamic settings are retrieved from
// IMDS via Sundog's child "Pluto".
#[model]
struct KubernetesSettings {
    // Settings we require the user to specify, likely via user data.
    cluster_name: KubernetesClusterName,
    cluster_certificate: ValidBase64,
    api_server: Url,
    node_labels: HashMap<KubernetesLabelKey, KubernetesLabelValue>,
    node_taints: HashMap<KubernetesLabelKey, KubernetesTaintValue>,
    static_pods: HashMap<Identifier, StaticPod>,

    // Dynamic settings.
    max_pods: u32,
    cluster_dns_ip: Ipv4Addr,
    cluster_domain: DNSDomain,
    node_ip: Ipv4Addr,
    pod_infra_container_image: SingleLineString,
}

// ECS settings.
#[model]
struct ECSSettings {
    cluster: String,
    instance_attributes: HashMap<ECSAttributeKey, ECSAttributeValue>,
    allow_privileged_containers: bool,
    logging_drivers: Vec<SingleLineString>,
    loglevel: ECSAgentLogLevel,
    enable_spot_instance_draining: bool,
}

// Update settings. Taken from userdata. The 'seed' setting is generated
// by the "Bork" settings generator at runtime.
#[model]
struct UpdatesSettings {
    metadata_base_url: Url,
    targets_base_url: Url,
    seed: u32,
    // Version to update to when updating via the API.
    version_lock: FriendlyVersion,
    ignore_waves: bool,
}

#[model]
struct ContainerImage {
    source: Url,
    enabled: bool,
    superpowered: bool,
    user_data: ValidBase64,
}

// Network settings. These settings will affect host service components' network behavior
#[model]
struct NetworkSettings {
    https_proxy: Url,
    // We allow some flexibility in NO_PROXY values because different services support different formats.
    no_proxy: Vec<SingleLineString>,
}

// NTP settings
#[model]
struct NtpSettings {
    time_servers: Vec<Url>,
}

// Kernel settings
#[model]
struct KernelSettings {
    lockdown: Lockdown,
    // Values are almost always a single line and often just an integer... but not always.
    sysctl: HashMap<SysctlKey, String>,
}

// Platform-specific settings
#[model]
struct AwsSettings {
    region: SingleLineString,
}

// Metrics settings
#[model]
struct MetricsSettings {
    metrics_url: Url,
    send_metrics: bool,
    service_checks: Vec<String>,
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
}

///// Metadata

#[model(add_option = false, rename = "metadata")]
struct Metadata {
    key: SingleLineString,
    md: SingleLineString,
    val: toml::Value,
}
