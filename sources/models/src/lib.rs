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

## aws-k8s-1.19: Kubernetes 1.19

* [Model](src/aws-k8s-1.22/mod.rs)
* [Default settings](src/aws-k8s-1.19/defaults.d/)

## aws-k8s-1.20: Kubernetes 1.20

* [Model](src/aws-k8s-1.22/mod.rs)
* [Default settings](src/aws-k8s-1.22/defaults.d/)

## aws-k8s-1.21: Kubernetes 1.21

* [Model](src/aws-k8s-1.22/mod.rs)
* [Default settings](src/aws-k8s-1.22/defaults.d/)

### aws-k8s-1.21-nvidia: Kubernetes 1.21 NVIDIA

* [Model](src/aws-k8s-1.22-nvidia/mod.rs)
* [Default settings](src/aws-k8s-1.22-nvidia/defaults.d/)

## aws-k8s-1.22: Kubernetes 1.22

* [Model](src/aws-k8s-1.22/mod.rs)
* [Default settings](src/aws-k8s-1.22/defaults.d/)

### aws-k8s-1.22-nvidia: Kubernetes 1.22 NVIDIA

* [Model](src/aws-k8s-1.22-nvidia/mod.rs)
* [Default settings](src/aws-k8s-1.22-nvidia/defaults.d/)

## aws-k8s-1.23: Kubernetes 1.23

* [Model](src/aws-k8s-1.23/mod.rs)
* [Default settings](src/aws-k8s-1.23/defaults.d/)

### aws-k8s-1.23-nvidia: Kubernetes 1.23 NVIDIA

* [Model](src/aws-k8s-1.23-nvidia/mod.rs)
* [Default settings](src/aws-k8s-1.23-nvidia/defaults.d/)

## aws-ecs-1: Amazon ECS

* [Model](src/aws-ecs-1/mod.rs)
* [Default settings](src/aws-ecs-1/defaults.d/)

## aws-dev: AWS development build

* [Model](src/aws-dev/mod.rs)
* [Default settings](src/aws-dev/defaults.d/)

## vmware-dev: VMware development build

* [Model](src/vmware-dev/mod.rs)
* [Default settings](src/vmware-dev/defaults.d/)

## vmware-k8s-1.20: VMware Kubernetes 1.20

* [Model](src/vmware-k8s-1.22/mod.rs)
* [Default settings](src/vmware-k8s-1.22/defaults.d/)

## vmware-k8s-1.21: VMware Kubernetes 1.21

* [Model](src/vmware-k8s-1.22/mod.rs)
* [Default settings](src/vmware-k8s-1.22/defaults.d/)

## vmware-k8s-1.22: VMware Kubernetes 1.22

* [Model](src/vmware-k8s-1.22/mod.rs)
* [Default settings](src/vmware-k8s-1.22/defaults.d/)

## vmware-k8s-1.23: VMware Kubernetes 1.23

* [Model](src/vmware-k8s-1.23/mod.rs)
* [Default settings](src/vmware-k8s-1.23/defaults.d/)

## metal-dev: Metal development build

* [Model](src/metal-dev/mod.rs)
* [Default settings](src/metal-dev/defaults.d/)

## metal-k8s-1.21: Metal Kubernetes 1.21

* [Model](src/metal-k8s-1.23/mod.rs)
* [Default settings](src/metal-k8s-1.22/defaults.d/)

## metal-k8s-1.22: Metal Kubernetes 1.22

* [Model](src/metal-k8s-1.23/mod.rs)
* [Default settings](src/metal-k8s-1.22/defaults.d/)

## metal-k8s-1.23: Metal Kubernetes 1.23

* [Model](src/metal-k8s-1.23/mod.rs)
* [Default settings](src/metal-k8s-1.23/defaults.d/)

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
// The "de" module contains custom deserialization trait implementation for models.
mod de;

pub use variant::*;

// Types used to communicate between client and server for 'apiclient exec'.
pub mod exec;

// Below, we define common structures used in the API surface; specific variants build a Settings
// structure based on these, and that's what gets exposed via the API.  (Specific variants' models
// are in subdirectories and linked into place by build.rs at variant/current.)

use model_derive::model;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;

use crate::de::{deserialize_mirrors, deserialize_node_taints};
use crate::modeled_types::{
    BootConfigKey, BootConfigValue, BootstrapContainerMode, CpuManagerPolicy, DNSDomain,
    ECSAgentImagePullBehavior, ECSAgentLogLevel, ECSAttributeKey, ECSAttributeValue,
    EtcHostsEntries, FriendlyVersion, Identifier, ImageGCHighThresholdPercent,
    ImageGCLowThresholdPercent, KubernetesAuthenticationMode, KubernetesBootstrapToken,
    KubernetesCloudProvider, KubernetesClusterDnsIp, KubernetesClusterName,
    KubernetesDurationValue, KubernetesEvictionHardKey, KubernetesLabelKey, KubernetesLabelValue,
    KubernetesQuantityValue, KubernetesReservedResourceKey, KubernetesTaintValue,
    KubernetesThresholdValue, Lockdown, PemCertificateString, SingleLineString, SysctlKey,
    TopologyManagerPolicy, TopologyManagerScope, Url, ValidBase64, ValidLinuxHostname,
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
    // Settings that must be specified via user data or through API requests.  Not all settings are
    // useful for all modes. For example, in standalone mode the user does not need to specify any
    // cluster information, and the bootstrap token is only needed for TLS authentication mode.
    cluster_name: KubernetesClusterName,
    cluster_certificate: ValidBase64,
    api_server: Url,
    node_labels: HashMap<KubernetesLabelKey, KubernetesLabelValue>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_node_taints"
    )]
    node_taints: HashMap<KubernetesLabelKey, Vec<KubernetesTaintValue>>,
    static_pods: HashMap<Identifier, StaticPod>,
    authentication_mode: KubernetesAuthenticationMode,
    bootstrap_token: KubernetesBootstrapToken,
    standalone_mode: bool,
    eviction_hard: HashMap<KubernetesEvictionHardKey, KubernetesThresholdValue>,
    kube_reserved: HashMap<KubernetesReservedResourceKey, KubernetesQuantityValue>,
    system_reserved: HashMap<KubernetesReservedResourceKey, KubernetesQuantityValue>,
    allowed_unsafe_sysctls: Vec<SingleLineString>,
    server_tls_bootstrap: bool,
    cloud_provider: KubernetesCloudProvider,
    registry_qps: i32,
    registry_burst: i32,
    event_qps: i32,
    event_burst: i32,
    kube_api_qps: i32,
    kube_api_burst: i32,
    container_log_max_size: KubernetesQuantityValue,
    container_log_max_files: i32,
    cpu_manager_policy: CpuManagerPolicy,
    cpu_manager_reconcile_period: KubernetesDurationValue,
    topology_manager_scope: TopologyManagerScope,
    topology_manager_policy: TopologyManagerPolicy,
    pod_pids_limit: i64,
    image_gc_high_threshold_percent: ImageGCHighThresholdPercent,
    image_gc_low_threshold_percent: ImageGCLowThresholdPercent,
    provider_id: Url,

    // Settings where we generate a value based on the runtime environment.  The user can specify a
    // value to override the generated one, but typically would not.
    max_pods: u32,
    cluster_dns_ip: KubernetesClusterDnsIp,
    cluster_domain: DNSDomain,
    node_ip: IpAddr,
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
    image_pull_behavior: ECSAgentImagePullBehavior,
}

#[model]
struct RegistryMirror {
    registry: SingleLineString,
    endpoint: Vec<Url>,
}

#[model]
struct RegistryCredential {
    registry: SingleLineString,
    username: SingleLineString,
    password: SingleLineString,
    // This is the base64 encoding of "username:password"
    auth: ValidBase64,
    identitytoken: SingleLineString,
}

// Image registry settings for the container runtimes.
#[model]
struct RegistrySettings {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_mirrors"
    )]
    mirrors: Vec<RegistryMirror>,
    #[serde(alias = "creds", default, skip_serializing_if = "Option::is_none")]
    credentials: Vec<RegistryCredential>,
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
struct HostContainer {
    source: Url,
    enabled: bool,
    superpowered: bool,
    user_data: ValidBase64,
}

// Network settings. These settings will affect host service components' network behavior
#[model]
struct NetworkSettings {
    hostname: ValidLinuxHostname,
    hosts: EtcHostsEntries,
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

// Kernel boot settings
#[model]
struct BootSettings {
    #[serde(
        alias = "kernel",
        rename(serialize = "kernel"),
        default,
        skip_serializing_if = "Option::is_none"
    )]
    kernel_parameters: HashMap<BootConfigKey, Vec<BootConfigValue>>,
    #[serde(
        alias = "init",
        rename(serialize = "init"),
        default,
        skip_serializing_if = "Option::is_none"
    )]
    init_parameters: HashMap<BootConfigKey, Vec<BootConfigValue>>,
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

// CloudFormation settings
#[model]
struct CloudFormationSettings {
    should_signal: bool,
    stack_name: SingleLineString,
    logical_resource_id: SingleLineString,
}

// AutoScaling settings
#[model]
struct AutoScalingSettings {
    should_wait: bool,
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

///// Bootstrap Containers

#[model]
struct BootstrapContainer {
    source: Url,
    mode: BootstrapContainerMode,
    user_data: ValidBase64,
    essential: bool,
}

///// PEM Certificates
#[model]
struct PemCertificate {
    data: PemCertificateString,
    trusted: bool,
}

///// OCI hooks
#[model]
struct OciHooks {
    log4j_hotpatch_enabled: bool,
}
