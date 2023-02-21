use model_derive::model;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::modeled_types::Identifier;
use crate::{
    AwsSettings, BootSettings, BootstrapContainer, ContainerRuntimeSettings, DnsSettings,
    HostContainer, KernelSettings, KubernetesSettings, MetricsSettings, NetworkSettings,
    NtpSettings, OciDefaults, OciHooks, PemCertificate, RegistrySettings, UpdatesSettings,
};

// Note: we have to use 'rename' here because the top-level Settings structure is the only one
// that uses its name in serialization; internal structures use the field name that points to it
#[model(rename = "settings", impl_default = true)]
struct Settings {
    motd: String,
    kubernetes: KubernetesSettings,
    updates: UpdatesSettings,
    host_containers: HashMap<Identifier, HostContainer>,
    bootstrap_containers: HashMap<Identifier, BootstrapContainer>,
    ntp: NtpSettings,
    network: NetworkSettings,
    kernel: KernelSettings,
    boot: BootSettings,
    aws: AwsSettings,
    metrics: MetricsSettings,
    pki: HashMap<Identifier, PemCertificate>,
    container_registry: RegistrySettings,
    oci_defaults: OciDefaults,
    oci_hooks: OciHooks,
    dns: DnsSettings,
    container_runtime: ContainerRuntimeSettings,
}
