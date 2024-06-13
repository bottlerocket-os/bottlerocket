/*!
# API models

Bottlerocket has different variants supporting different features and use cases.
Each variant has its own set of software, and therefore needs its own configuration.
We support having an API model for each variant to support these different configurations.

The model here defines a top-level `Settings` structure, and delegates the actual implementation to a ["settings plugin"](https://github.com/bottlerocket/bottlerocket-settings-sdk/tree/settings-plugins).
Settings plugin are written in Rust as a "cdylib" crate, and loaded at runtime.

Each settings plugin must define its own private `Settings` structure.
It can use pre-defined structures inside, or custom ones as needed.

`apiserver::datastore` offers serialization and deserialization modules that make it easy to map between Rust types and the data store, and thus, all inputs and outputs are type-checked.

At the field level, standard Rust types can be used, or ["modeled types"](src/modeled_types) that add input validation.

The `#[model]` attribute on Settings and its sub-structs reduces duplication and adds some required metadata; see [its docs](model-derive/) for details.
*/

// Clippy has a false positive in the presence of the Scalar macro.
#![allow(clippy::derived_hash_with_manual_eq)]

// The "de" module contains custom deserialization trait implementation for models.
mod de;

pub use modeled_types;
use modeled_types::KubernetesCPUManagerPolicyOption;
use modeled_types::KubernetesEvictionKey;
use modeled_types::KubernetesMemoryManagerPolicy;
use modeled_types::KubernetesMemoryReservation;
use modeled_types::NonNegativeInteger;

// Types used to communicate between client and server for 'apiclient exec'.
pub mod exec;

// Below, we define common structures used in the API surface; specific variants build a Settings
// structure based on these, and that's what gets exposed via the API.  (Specific variants' models
// are in subdirectories and linked into place by build.rs at variant/current.)

use bottlerocket_release::BottlerocketRelease;
use bottlerocket_settings_plugin::BottlerocketSettings;
use model_derive::model;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;

use crate::de::{deserialize_limit, deserialize_mirrors, deserialize_node_taints};
use modeled_types::{
    BootConfigKey, BootConfigValue, BootstrapContainerMode, CpuManagerPolicy, CredentialProvider,
    DNSDomain, EtcHostsEntries, Identifier, IntegerPercent, KernelCpuSetValue,
    KubernetesAuthenticationMode, KubernetesBootstrapToken, KubernetesCloudProvider,
    KubernetesClusterDnsIp, KubernetesClusterName, KubernetesDurationValue, KubernetesLabelKey,
    KubernetesLabelValue, KubernetesQuantityValue, KubernetesReservedResourceKey,
    KubernetesTaintValue, KubernetesThresholdValue, OciDefaultsCapability,
    OciDefaultsResourceLimitType, PemCertificateString, SingleLineString, TopologyManagerPolicy,
    TopologyManagerScope, Url, ValidBase64, ValidLinuxHostname,
};

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
    eviction_hard: HashMap<KubernetesEvictionKey, KubernetesThresholdValue>,
    eviction_soft: HashMap<KubernetesEvictionKey, KubernetesThresholdValue>,
    eviction_soft_grace_period: HashMap<KubernetesEvictionKey, KubernetesDurationValue>,
    eviction_max_pod_grace_period: NonNegativeInteger,
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
    cpu_cfs_quota_enforced: bool,
    cpu_manager_policy: CpuManagerPolicy,
    cpu_manager_reconcile_period: KubernetesDurationValue,
    cpu_manager_policy_options: Vec<KubernetesCPUManagerPolicyOption>,
    topology_manager_scope: TopologyManagerScope,
    topology_manager_policy: TopologyManagerPolicy,
    pod_pids_limit: i64,
    image_gc_high_threshold_percent: IntegerPercent,
    image_gc_low_threshold_percent: IntegerPercent,
    provider_id: Url,
    log_level: u8,
    credential_providers: HashMap<Identifier, CredentialProvider>,
    server_certificate: ValidBase64,
    server_key: ValidBase64,
    shutdown_grace_period: KubernetesDurationValue,
    shutdown_grace_period_for_critical_pods: KubernetesDurationValue,
    memory_manager_reserved_memory: HashMap<Identifier, KubernetesMemoryReservation>,
    memory_manager_policy: KubernetesMemoryManagerPolicy,
    reserved_cpus: KernelCpuSetValue,

    // Settings where we generate a value based on the runtime environment.  The user can specify a
    // value to override the generated one, but typically would not.
    max_pods: u32,
    cluster_dns_ip: KubernetesClusterDnsIp,
    cluster_domain: DNSDomain,
    node_ip: IpAddr,
    pod_infra_container_image: SingleLineString,
    // Generated in `aws-k8s-1.26*` variants only
    hostname_override: ValidLinuxHostname,
    // Generated in `k8s-1.25+` variants only
    seccomp_default: bool,
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

#[model]
struct HostContainer {
    source: Url,
    enabled: bool,
    superpowered: bool,
    user_data: ValidBase64,
}

// Network settings. These settings will affect host service components' network behavior
#[model(impl_default = true)]
struct NetworkSettings {
    hostname: ValidLinuxHostname,
    hosts: EtcHostsEntries,
    https_proxy: Url,
    // We allow some flexibility in NO_PROXY values because different services support different formats.
    no_proxy: Vec<SingleLineString>,
}

// Kernel module settings
#[model]
struct KmodSetting {
    allowed: bool,
    autoload: bool,
}

// Kernel boot settings
#[model(impl_default = true)]
struct BootSettings {
    reboot_to_reconcile: bool,
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

// AutoScaling settings
#[model]
struct AutoScalingSettings {
    should_wait: bool,
}

// Container runtime settings
#[model]
struct ContainerRuntimeSettings {
    max_container_log_line_size: i32,
    max_concurrent_downloads: i32,
    enable_unprivileged_ports: bool,
    enable_unprivileged_icmp: bool,
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

///// OCI defaults specifies the default values that will be used in cri-base-json.
#[model]
struct OciDefaults {
    capabilities: HashMap<OciDefaultsCapability, bool>,
    resource_limits: HashMap<OciDefaultsResourceLimitType, OciDefaultsResourceLimit>,
}

///// The hard and soft limit values for an OCI defaults resource limit.
#[model(add_option = false)]
#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, Ord, PartialOrd, PartialEq)]
struct OciDefaultsResourceLimit {
    #[serde(deserialize_with = "deserialize_limit")]
    hard_limit: i64,
    #[serde(deserialize_with = "deserialize_limit")]
    soft_limit: i64,
}

#[model(add_option = false)]
struct Report {
    name: String,
    description: String,
}
