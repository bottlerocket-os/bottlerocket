use migration_helpers::common_migrations::AddPrefixesMigration;
use migration_helpers::{migrate, Result};
use std::process;

/// We added settings for the network DRA driver (DRANET) host service:
/// - settings.kubelet-dra-drivers.net.enabled
/// - settings.kubelet-dra-drivers.net.bind-address
/// - settings.kubelet-dra-drivers.net.advertised-devices
fn run() -> Result<()> {
    migrate(AddPrefixesMigration(vec![
        "settings.kubelet-dra-drivers.net",
    ]))
}

// Returning a Result from main makes it print a Debug representation of the error, but with Snafu
// we have nice Display representations of the error, so we wrap "main" (run) and print any error.
// https://github.com/shepmaster/snafu/issues/110
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        process::exit(1);
    }
}

#[cfg(test)]
mod test_kubelet_dra_drivers_net_settings {
    use maplit::hashmap;
    use migration_helpers::common_migrations::AddPrefixesMigration;
    use migration_helpers::{Migration, MigrationData};
    use std::collections::HashMap;

    const PREFIX: &str = "settings.kubelet-dra-drivers.net";

    fn populated_data() -> MigrationData {
        MigrationData {
            data: hashmap! {
                "settings.kubelet-dra-drivers.net.enabled".into() => false.into(),
                "settings.kubelet-dra-drivers.net.bind-address".into() => "127.0.0.1:9177".into(),
                "settings.kubelet-dra-drivers.net.advertised-devices".into() => "efa".into(),
                "settings.kubelet-dra-drivers.nvidia.enabled".into() => false.into(),
                "settings.kubelet-device-plugins.nvidia.enabled".into() => true.into(),
            },
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn forward_keeps_every_setting() {
        let expected = populated_data().data;
        let result = AddPrefixesMigration(vec![PREFIX])
            .forward(populated_data())
            .unwrap();
        assert_eq!(result.data, expected);
    }

    #[test]
    fn backward_removes_only_the_net_subtree() {
        let result = AddPrefixesMigration(vec![PREFIX])
            .backward(populated_data())
            .unwrap();
        assert_eq!(
            result.data,
            hashmap! {
                "settings.kubelet-dra-drivers.nvidia.enabled".into() => false.into(),
                "settings.kubelet-device-plugins.nvidia.enabled".into() => true.into(),
            }
        );
    }

    #[test]
    fn backward_with_no_net_settings_is_a_no_op() {
        let data = MigrationData {
            data: hashmap! {
                "settings.kubelet-dra-drivers.nvidia.enabled".into() => true.into(),
            },
            metadata: HashMap::new(),
        };
        let expected = data.data.clone();
        let result = AddPrefixesMigration(vec![PREFIX]).backward(data).unwrap();
        assert_eq!(result.data, expected);
    }
}
