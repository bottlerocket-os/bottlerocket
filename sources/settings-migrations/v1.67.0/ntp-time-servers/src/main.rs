use migration_helpers::{migrate, Migration, MigrationData, Result};
use serde_json::Value;
use std::process;

mod options;

const SERVERS: &str = "settings.ntp.time-servers";
const OPTIONS: &str = "settings.ntp.options";
const LOGGING: &str = "settings.ntp.logging";

/// Existing lists and defaults remain unchanged on upgrade. On rollback, object
/// lists become legacy address lists with textually matching complete options
/// shared by all sources. Invalid source entries stop the rollback migration.
pub struct NtpTimeServersMigration;

impl Migration for NtpTimeServersMigration {
    fn forward(&mut self, input: MigrationData) -> Result<MigrationData> {
        Ok(input)
    }

    fn backward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        input.data.remove(LOGGING);
        input.metadata.remove(LOGGING);

        let Some(value) = input.data.get(SERVERS) else {
            return Ok(input);
        };
        let servers =
            value
                .as_array()
                .ok_or_else(|| migration_helpers::error::Error::Validation {
                    msg: format!("Cannot roll back {SERVERS}: expected a list"),
                })?;
        if servers.iter().all(Value::is_string) {
            return Ok(input);
        }

        let server_metadata = input.metadata.get(SERVERS).cloned();
        let mut addresses = Vec::new();
        let mut option_lists = Vec::new();
        for (index, server) in servers.iter().enumerate() {
            match server {
                Value::String(_) => {
                    addresses.push(server.clone());
                    // Mixed lists are rejected by the SDK. If one is already
                    // stored, do not apply object-only flags to string entries.
                    option_lists.push(None);
                }
                Value::Object(fields) => {
                    let address = fields
                        .get("address")
                        .filter(|address| address.is_string())
                        .ok_or_else(|| migration_helpers::error::Error::Validation {
                            msg: format!(
                                "Cannot roll back {SERVERS}[{index}]: object requires a string address"
                            ),
                        })?;
                    addresses.push(address.clone());
                    option_lists.push(fields.get("options"));
                }
                _ => {
                    return Err(migration_helpers::error::Error::Validation {
                        msg: format!(
                            "Cannot roll back {SERVERS}[{index}]: expected an address string or server object"
                        ),
                    });
                }
            }
        }

        let shared_options = options::common(option_lists);
        input.data.insert(SERVERS.into(), Value::Array(addresses));
        // [] is an explicit value. Removing this key would let Storewolf restore
        // the legacy default options and change the rollback projection.
        input
            .data
            .insert(OPTIONS.into(), Value::Array(shared_options));
        input.metadata.remove(OPTIONS);
        if let Some(metadata) = server_metadata {
            // The migrator removes weak settings before clearing all metadata.
            // Matching strength keeps addresses and projected options together:
            // weak values are both re-defaulted, while user-set values survive.
            input.metadata.insert(OPTIONS.into(), metadata);
        }
        Ok(input)
    }
}

fn main() {
    if let Err(error) = migrate(NtpTimeServersMigration) {
        eprintln!("{error}");
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::hashmap;
    use serde_json::json;
    use std::collections::HashMap;

    fn data(values: HashMap<String, Value>) -> MigrationData {
        MigrationData {
            data: values,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn forward_preserves_legacy_values_and_metadata() {
        for servers in [
            json!([]),
            json!(["a.example"]),
            json!([{"address":"a.example"}]),
        ] {
            let input = MigrationData {
                data: hashmap! { SERVERS.into() => servers, OPTIONS.into() => json!(["minpoll", "6"]) },
                metadata: hashmap! {
                    SERVERS.into() => hashmap! {"strength".into() => json!("strong")},
                },
            };
            assert_eq!(
                NtpTimeServersMigration.forward(input.clone()).unwrap(),
                input
            );
        }
    }

    #[test]
    fn rollback_preserves_addresses_and_complete_common_options() {
        let metadata = hashmap! {"strength".into() => json!("strong")};
        let input = MigrationData {
            data: hashmap! {
                SERVERS.into() => json!([
                    {"address":"169.254.169.123", "directive":"server",
                     "options":["prefer", "iburst", "minpoll 4", "maxpoll 4"]},
                    {"address":"time.aws.com", "directive":"pool", "options":["iburst"]}
                ]),
                LOGGING.into() => json!(["tracking"]),
            },
            metadata: hashmap! {
                SERVERS.into() => metadata.clone(),
                LOGGING.into() => hashmap! {"strength".into() => json!("strong")},
            },
        };
        let output = NtpTimeServersMigration.backward(input).unwrap();
        assert_eq!(
            output.data,
            hashmap! {
                SERVERS.into() => json!(["169.254.169.123", "time.aws.com"]),
                OPTIONS.into() => json!(["iburst"]),
            }
        );
        assert_eq!(
            output.metadata,
            hashmap! {
                SERVERS.into() => metadata.clone(),
                OPTIONS.into() => metadata,
            }
        );
    }

    #[test]
    fn rollback_without_common_options_writes_an_explicit_empty_value() {
        let input = data(hashmap! {
            SERVERS.into() => json!([
                {"address":"a.example","options":["minpoll", "4"]},
                {"address":"b.example","options":["minpoll", "6"]}
            ]),
            OPTIONS.into() => json!(["stale"]),
        });
        let output = NtpTimeServersMigration.backward(input).unwrap();
        assert_eq!(output.data.get(OPTIONS), Some(&json!([])));
        assert_eq!(
            NtpTimeServersMigration.backward(output.clone()).unwrap(),
            output
        );
    }

    #[test]
    fn rollback_leaves_legacy_and_empty_lists_unchanged() {
        for servers in [json!([]), json!(["a.example"])] {
            let input = data(hashmap! {
                SERVERS.into() => servers,
                OPTIONS.into() => json!(["iburst", "minpoll", "6"]),
            });
            assert_eq!(
                NtpTimeServersMigration.backward(input.clone()).unwrap(),
                input
            );
        }
    }

    #[test]
    fn rollback_rejects_all_invalid_objects_instead_of_disabling_sources() {
        let input = data(hashmap! {
            SERVERS.into() => json!([
                {"directive":"server","options":["iburst"]},
                {"address":null,"directive":"pool"}
            ]),
        });
        let error = NtpTimeServersMigration.backward(input).unwrap_err();
        assert!(matches!(
            error,
            migration_helpers::error::Error::Validation { ref msg }
                if msg == "Cannot roll back settings.ntp.time-servers[0]: object requires a string address"
        ));
    }

    #[test]
    fn rollback_rejects_invalid_entries_even_when_other_sources_are_valid() {
        for entry in [json!({}), json!({"address":4}), json!(null), json!(4)] {
            let input = data(hashmap! {
                SERVERS.into() => json!([{"address":"a.example"}, entry]),
            });
            let error = NtpTimeServersMigration.backward(input).unwrap_err();
            assert!(matches!(
                error,
                migration_helpers::error::Error::Validation { ref msg }
                    if msg.starts_with("Cannot roll back settings.ntp.time-servers[1]:")
            ));
        }
    }

    #[test]
    fn rollback_rejects_invalid_scalars_and_non_list_values() {
        for servers in [json!([null]), json!([4]), json!({}), json!(false)] {
            let input = data(hashmap! { SERVERS.into() => servers });
            assert!(matches!(
                NtpTimeServersMigration.backward(input).unwrap_err(),
                migration_helpers::error::Error::Validation { .. }
            ));
        }
    }

    #[test]
    fn sparse_pending_logging_removal_preserves_other_data_and_metadata() {
        let input = MigrationData {
            data: hashmap! {
                LOGGING.into() => json!(["tracking"]),
                "settings.motd".into() => json!("hello"),
            },
            metadata: hashmap! {
                LOGGING.into() => hashmap! {"strength".into() => json!("strong")},
                "settings.motd".into() => hashmap! {"strength".into() => json!("strong")},
            },
        };
        let output = NtpTimeServersMigration.backward(input).unwrap();
        assert_eq!(
            output,
            MigrationData {
                data: hashmap! {"settings.motd".into() => json!("hello")},
                metadata: hashmap! {"settings.motd".into() => hashmap! {"strength".into() => json!("strong")}},
            }
        );
    }

    #[test]
    fn mixed_stored_entries_do_not_share_object_only_options() {
        let input = data(hashmap! {
            SERVERS.into() => json!(["a.example", {"address":"b.example","options":["prefer"]}]),
        });
        let output = NtpTimeServersMigration.backward(input).unwrap();
        assert_eq!(
            output.data.get(SERVERS),
            Some(&json!(["a.example", "b.example"]))
        );
        assert_eq!(output.data.get(OPTIONS), Some(&json!([])));
    }

    #[test]
    fn sparse_options_transaction_stays_unchanged() {
        let input = MigrationData {
            data: hashmap! { OPTIONS.into() => json!(["minpoll", "6"]) },
            metadata: hashmap! {
                OPTIONS.into() => hashmap! {"strength".into() => json!("weak")},
            },
        };
        assert_eq!(
            NtpTimeServersMigration.backward(input.clone()).unwrap(),
            input
        );
    }

    #[test]
    fn object_transaction_gets_explicit_options_and_matching_strength() {
        let input = MigrationData {
            data: hashmap! {
                SERVERS.into() => json!([{"address":"a.example"}, {"address":"b.example"}]),
            },
            metadata: hashmap! {
                SERVERS.into() => hashmap! {"strength".into() => json!("weak")},
            },
        };
        let output = NtpTimeServersMigration.backward(input).unwrap();
        assert_eq!(output.data.get(OPTIONS), Some(&json!([])));
        assert_eq!(output.metadata.get(OPTIONS), output.metadata.get(SERVERS));
    }
}
