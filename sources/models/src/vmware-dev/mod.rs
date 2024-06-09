use model_derive::model;
use std::collections::HashMap;

use crate::{HostContainer, NetworkSettings};
use modeled_types::Identifier;

// Note: we have to use 'rename' here because the top-level Settings structure is the only one
// that uses its name in serialization; internal structures use the field name that points to it
#[model(rename = "settings", impl_default = true)]
struct Settings {
    motd: settings_extension_motd::MotdV1,
    updates: settings_extension_updates::UpdatesSettingsV1,
    host_containers: HashMap<Identifier, HostContainer>,
    bootstrap_containers: settings_extension_bootstrap_container::BootstrapContainerSettingsV1,
    ntp: settings_extension_ntp::NtpSettingsV1,
    network: NetworkSettings,
    kernel: settings_extension_kernel::KernelSettingsV1,
    boot: BootSettings,
    metrics: settings_extension_metrics::MetricsSettingsV1,
    pki: settings_extension_pki::PkiSettingsV1,
    container_registry: settings_extension_container_registry::RegistrySettingsV1,
    oci_hooks: settings_extension_oci_hooks::OciHooksSettingsV1,
    dns: settings_extension_dns::DnsSettingsV1,
}
