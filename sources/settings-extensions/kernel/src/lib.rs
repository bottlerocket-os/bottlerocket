/// The kernel settings can be used to configure settings related to the kernel, e.g.  
/// kernel modules
use bottlerocket_settings_sdk::{GenerateResult, SettingsModel};
use model_derive::model;
use modeled_types::{KmodKey, Lockdown, SysctlKey};
use std::collections::HashMap;
use std::convert::Infallible;

#[model(impl_default = true)]
struct KernelSettingsV1 {
    lockdown: Lockdown,
    modules: HashMap<KmodKey, KmodSetting>,
    // Values are almost always a single line and often just an integer... but not always.
    sysctl: HashMap<SysctlKey, String>,
}

#[model]
struct KmodSetting {
    allowed: bool,
    autoload: bool,
}

type Result<T> = std::result::Result<T, Infallible>;

impl SettingsModel for KernelSettingsV1 {
    type PartialKind = Self;
    type ErrorKind = Infallible;

    fn get_version() -> &'static str {
        "v1"
    }

    fn set(_current_value: Option<Self>, _target: Self) -> Result<()> {
        // allow anything that parses as KernelSettingsV1
        Ok(())
    }

    fn generate(
        existing_partial: Option<Self::PartialKind>,
        _dependent_settings: Option<serde_json::Value>,
    ) -> Result<GenerateResult<Self::PartialKind, Self>> {
        Ok(GenerateResult::Complete(
            existing_partial.unwrap_or_default(),
        ))
    }

    fn validate(_value: Self, _validated_settings: Option<serde_json::Value>) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_generate_kernel() {
        let generated = KernelSettingsV1::generate(None, None).unwrap();

        assert_eq!(
            generated,
            GenerateResult::Complete(KernelSettingsV1 {
                lockdown: None,
                modules: None,
                sysctl: None,
            })
        )
    }

    #[test]
    fn test_serde_kernel() {
        let test_json = r#"{
            "lockdown": "integrity",
            "modules": {"foo": {"allowed": true, "autoload": true}},
            "sysctl": {"key": "value"}
        }"#;

        let kernel: KernelSettingsV1 = serde_json::from_str(test_json).unwrap();

        let mut modules = HashMap::new();
        modules.insert(
            KmodKey::try_from("foo").unwrap(),
            KmodSetting {
                allowed: Some(true),
                autoload: Some(true),
            },
        );
        let modules = Some(modules);

        let mut sysctl = HashMap::new();
        sysctl.insert(SysctlKey::try_from("key").unwrap(), String::from("value"));
        let sysctl = Some(sysctl);

        assert_eq!(
            kernel,
            KernelSettingsV1 {
                lockdown: Some(Lockdown::try_from("integrity").unwrap()),
                modules,
                sysctl,
            }
        );
    }
}
