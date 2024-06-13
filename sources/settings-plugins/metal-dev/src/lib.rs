use bottlerocket_settings_plugin::SettingsPlugin;
use model::BootSettings;
use model_derive::model;

#[derive(SettingsPlugin)]
#[model(rename = "settings", impl_default = true)]
struct MetalDevSettings {
    motd: settings_extension_motd::MotdV1,
    updates: settings_extension_updates::UpdatesSettingsV1,
    host_containers: settings_extension_host_containers::HostContainersSettingsV1,
    bootstrap_containers: settings_extension_bootstrap_containers::BootstrapContainersSettingsV1,
    ntp: settings_extension_ntp::NtpSettingsV1,
    network: settings_extension_network::NetworkSettingsV1,
    kernel: settings_extension_kernel::KernelSettingsV1,
    boot: BootSettings,
    metrics: settings_extension_metrics::MetricsSettingsV1,
    pki: settings_extension_pki::PkiSettingsV1,
    container_registry: settings_extension_container_registry::RegistrySettingsV1,
    oci_hooks: settings_extension_oci_hooks::OciHooksSettingsV1,
    dns: settings_extension_dns::DnsSettingsV1,
}
