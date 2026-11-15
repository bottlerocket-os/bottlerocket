use migration_helpers::{migrate, Migration, MigrationData, Result};
use std::process;

const OLD_PREFIX: &str = "settings.kubelet-dra-drivers.net";
const NEW_PREFIX: &str = "settings.kubelet-dra-drivers-net";

/// Moves every data and metadata key under one prefix to the other, in whole.
/// The leaf names do not change, so only the prefix is rewritten.
fn move_prefix(input: &mut MigrationData, from: &str, to: &str) {
    let data_keys: Vec<String> = input
        .data
        .keys()
        .filter(|k| *k == from || k.starts_with(&format!("{from}.")))
        .cloned()
        .collect();
    for old_key in data_keys {
        let new_key = format!("{to}{}", &old_key[from.len()..]);
        if let Some(value) = input.data.remove(&old_key) {
            println!("Moved {old_key} to {new_key}, which was set to '{value}'");
            input.data.insert(new_key, value);
        }
    }

    let metadata_keys: Vec<String> = input
        .metadata
        .keys()
        .filter(|k| *k == from || k.starts_with(&format!("{from}.")))
        .cloned()
        .collect();
    for old_key in metadata_keys {
        let new_key = format!("{to}{}", &old_key[from.len()..]);
        if let Some(metadata) = input.metadata.remove(&old_key) {
            println!("Moved metadata for {old_key} to {new_key}");
            input.metadata.insert(new_key, metadata);
        }
    }
}

pub struct RenameNetDraDriverSettings;

impl Migration for RenameNetDraDriverSettings {
    /// New versions model the network DRA driver as its own settings extension,
    /// so the old subtree has to move; keys the new model does not know would
    /// fail deserialization.
    fn forward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        move_prefix(&mut input, OLD_PREFIX, NEW_PREFIX);
        Ok(input)
    }

    /// Older versions model the driver under kubelet-dra-drivers, so move the
    /// subtree back and keep whatever the node had configured.
    fn backward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        move_prefix(&mut input, NEW_PREFIX, OLD_PREFIX);
        Ok(input)
    }
}

/// The network DRA driver (DRANET) settings moved out of
/// settings.kubelet-dra-drivers.net into their own extension at
/// settings.kubelet-dra-drivers-net, so a variant can carry them without the
/// NVIDIA GPU driver's settings.
fn run() -> Result<()> {
    migrate(RenameNetDraDriverSettings)
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
mod test_kubelet_dra_drivers_net_rename {
    use super::RenameNetDraDriverSettings;
    use maplit::hashmap;
    use migration_helpers::{Migration, MigrationData};
    use std::collections::HashMap;

    fn old_data() -> MigrationData {
        MigrationData {
            data: hashmap! {
                "settings.kubelet-dra-drivers.net.enabled".into() => true.into(),
                "settings.kubelet-dra-drivers.net.bind-address".into() => "127.0.0.1:9177".into(),
                "settings.kubelet-dra-drivers.net.advertised-devices".into() => "efa".into(),
                "settings.kubelet-dra-drivers.nvidia.enabled".into() => false.into(),
                "settings.kubelet-device-plugins.nvidia.enabled".into() => true.into(),
            },
            metadata: hashmap! {
                "settings.kubelet-dra-drivers.net".into() => hashmap! {
                    "affected-services".into() => vec!["dranet"].into(),
                },
                "settings.kubelet-dra-drivers.nvidia".into() => hashmap! {
                    "affected-services".into() => vec!["nvidia-dra-driver-gpu"].into(),
                },
            },
        }
    }

    fn new_data() -> MigrationData {
        MigrationData {
            data: hashmap! {
                "settings.kubelet-dra-drivers-net.enabled".into() => true.into(),
                "settings.kubelet-dra-drivers-net.bind-address".into() => "127.0.0.1:9177".into(),
                "settings.kubelet-dra-drivers-net.advertised-devices".into() => "efa".into(),
                "settings.kubelet-dra-drivers.nvidia.enabled".into() => false.into(),
                "settings.kubelet-device-plugins.nvidia.enabled".into() => true.into(),
            },
            metadata: hashmap! {
                "settings.kubelet-dra-drivers-net".into() => hashmap! {
                    "affected-services".into() => vec!["dranet"].into(),
                },
                "settings.kubelet-dra-drivers.nvidia".into() => hashmap! {
                    "affected-services".into() => vec!["nvidia-dra-driver-gpu"].into(),
                },
            },
        }
    }

    #[test]
    fn forward_moves_data_and_metadata_to_the_new_prefix() {
        let result = RenameNetDraDriverSettings.forward(old_data()).unwrap();
        assert_eq!(result, new_data());
    }

    #[test]
    fn backward_moves_data_and_metadata_to_the_old_prefix() {
        let result = RenameNetDraDriverSettings.backward(new_data()).unwrap();
        assert_eq!(result, old_data());
    }

    #[test]
    fn forward_keeps_the_values_it_moves() {
        let result = RenameNetDraDriverSettings.forward(old_data()).unwrap();
        assert_eq!(
            result.data.get("settings.kubelet-dra-drivers-net.enabled"),
            Some(&true.into())
        );
        assert_eq!(
            result
                .data
                .get("settings.kubelet-dra-drivers-net.bind-address"),
            Some(&"127.0.0.1:9177".into())
        );
    }

    #[test]
    fn forward_leaves_no_key_under_the_old_prefix() {
        let result = RenameNetDraDriverSettings.forward(old_data()).unwrap();
        assert!(!result
            .data
            .keys()
            .any(|k| k.starts_with("settings.kubelet-dra-drivers.net")));
        assert!(!result
            .metadata
            .keys()
            .any(|k| k.starts_with("settings.kubelet-dra-drivers.net")));
    }

    #[test]
    fn the_nvidia_subtree_is_untouched_in_both_directions() {
        for result in [
            RenameNetDraDriverSettings.forward(old_data()).unwrap(),
            RenameNetDraDriverSettings.backward(new_data()).unwrap(),
        ] {
            assert_eq!(
                result
                    .data
                    .get("settings.kubelet-dra-drivers.nvidia.enabled"),
                Some(&false.into())
            );
            assert_eq!(
                result
                    .data
                    .get("settings.kubelet-device-plugins.nvidia.enabled"),
                Some(&true.into())
            );
            assert_eq!(
                result
                    .metadata
                    .get("settings.kubelet-dra-drivers.nvidia")
                    .and_then(|m| m.get("affected-services")),
                Some(&vec!["nvidia-dra-driver-gpu"].into())
            );
        }
    }

    #[test]
    fn a_round_trip_returns_the_input() {
        let forward = RenameNetDraDriverSettings.forward(old_data()).unwrap();
        let back = RenameNetDraDriverSettings.backward(forward).unwrap();
        assert_eq!(back, old_data());
    }

    #[test]
    fn both_directions_are_a_no_op_with_no_net_settings() {
        let data = MigrationData {
            data: hashmap! {
                "settings.kubelet-dra-drivers.nvidia.enabled".into() => true.into(),
            },
            metadata: HashMap::new(),
        };
        assert_eq!(
            RenameNetDraDriverSettings.forward(data.clone()).unwrap(),
            data
        );
        assert_eq!(
            RenameNetDraDriverSettings.backward(data.clone()).unwrap(),
            data
        );
    }

    #[test]
    fn a_partial_subtree_moves_without_inventing_keys() {
        let data = MigrationData {
            data: hashmap! {
                "settings.kubelet-dra-drivers.net.enabled".into() => false.into(),
            },
            metadata: HashMap::new(),
        };
        let result = RenameNetDraDriverSettings.forward(data).unwrap();
        assert_eq!(
            result.data,
            hashmap! {
                "settings.kubelet-dra-drivers-net.enabled".into() => false.into(),
            }
        );
    }

    #[test]
    fn a_similarly_named_sibling_is_not_moved() {
        // settings.kubelet-dra-drivers.network would match a naive prefix test
        // that forgot the dot boundary.
        let data = MigrationData {
            data: hashmap! {
                "settings.kubelet-dra-drivers.network.enabled".into() => true.into(),
            },
            metadata: HashMap::new(),
        };
        let expected = data.clone();
        assert_eq!(RenameNetDraDriverSettings.forward(data).unwrap(), expected);
    }
}
