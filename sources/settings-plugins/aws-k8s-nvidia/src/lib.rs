use bottlerocket_settings_models::model_derive::model;
use bottlerocket_settings_plugin::SettingsPlugin;
use bottlerocket_settings_models::kubernetes::NVIDIA_DEVICE_PLUGIN_FEATURE_ENABLED;

// check nvidia-device-plugin feature flag is enabled for this pacakge
const _: () = assert!(NVIDIA_DEVICE_PLUGIN_FEATURE_ENABLED, "nvidia-device-plugin feature flag should be enabled for this package.");

#[derive(SettingsPlugin)]
#[model(rename = "settings", impl_default = true)]
struct AwsK8sSettings {
    motd: bottlerocket_settings_models::MotdV1,
    kubernetes: bottlerocket_settings_models::KubernetesSettingsV1,
    updates: bottlerocket_settings_models::UpdatesSettingsV1,
    host_containers: bottlerocket_settings_models::HostContainersSettingsV1,
    bootstrap_containers: bottlerocket_settings_models::BootstrapContainersSettingsV1,
    ntp: bottlerocket_settings_models::NtpSettingsV1,
    network: bottlerocket_settings_models::NetworkSettingsV1,
    kernel: bottlerocket_settings_models::KernelSettingsV1,
    boot: bottlerocket_settings_models::BootSettingsV1,
    aws: bottlerocket_settings_models::AwsSettingsV1,
    metrics: bottlerocket_settings_models::MetricsSettingsV1,
    pki: bottlerocket_settings_models::PkiSettingsV1,
    container_registry: bottlerocket_settings_models::RegistrySettingsV1,
    oci_defaults: bottlerocket_settings_models::OciDefaultsV1,
    oci_hooks: bottlerocket_settings_models::OciHooksSettingsV1,
    cloudformation: bottlerocket_settings_models::CloudFormationSettingsV1,
    dns: bottlerocket_settings_models::DnsSettingsV1,
    container_runtime: bottlerocket_settings_models::ContainerRuntimeSettingsV1,
    autoscaling: bottlerocket_settings_models::AutoScalingSettingsV1,
    nvidia_container_runtime: bottlerocket_settings_models::NvidiaContainerRuntimeSettingsV1,
}
