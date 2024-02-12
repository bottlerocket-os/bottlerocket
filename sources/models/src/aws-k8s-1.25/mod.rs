use crate::{
    AutoScalingSettings, BootSettings, BootstrapContainer, CloudFormationSettings,
    ContainerRuntimeSettings, DnsSettings, KubernetesSettings, MetricsSettings, NetworkSettings,
    OciDefaults, OciHooks, PemCertificate,
};
use modeled_types::Identifier;

use model_derive::model;
use std::collections::HashMap;

// Note: we have to use 'rename' here because the top-level Settings structure is the only one
// that uses its name in serialization; internal structures use the field name that points to it
#[model(rename = "settings", impl_default = true)]
struct Settings {
    motd: settings_extension_motd::MotdV1,
    kubernetes: KubernetesSettings,
    updates: settings_extension_updates::UpdatesSettingsV1,
    host_containers: settings_extension_host_containers::HostContainersSettingsV1,
    bootstrap_containers: HashMap<Identifier, BootstrapContainer>,
    ntp: settings_extension_ntp::NtpSettingsV1,
    network: NetworkSettings,
    kernel: settings_extension_kernel::KernelSettingsV1,
    boot: BootSettings,
    aws: settings_extension_aws::AwsSettingsV1,
    metrics: MetricsSettings,
    pki: HashMap<Identifier, PemCertificate>,
    container_registry: settings_extension_container_registry::RegistrySettingsV1,
    oci_defaults: OciDefaults,
    oci_hooks: OciHooks,
    cloudformation: CloudFormationSettings,
    dns: DnsSettings,
    container_runtime: ContainerRuntimeSettings,
    autoscaling: AutoScalingSettings,
}
