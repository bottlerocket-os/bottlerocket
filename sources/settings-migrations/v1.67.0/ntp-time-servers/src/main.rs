use migration_helpers::named_map::{
    migrate_list_to_named_map, migrate_named_map_to_list, read_flattened_named_map,
    replace_flattened_named_map, ListReplacement,
};
use migration_helpers::{migrate, Metadata, Migration, MigrationData, Result};
use serde_json::Value;
use std::collections::{BTreeMap, HashSet};
use std::process;

const LEGACY_SERVERS: &str = "settings.ntp.time-servers";
const LEGACY_OPTIONS: &str = "settings.ntp.options";
const LOGGING: &str = "settings.ntp.logging";
#[cfg(test)]
const NAMED_SERVER_PREFIX: &str = "settings.ntp.time-servers.";
const NAMED_SERVER_PREFIX_SEGMENTS: &[&str] = &["settings", "ntp", "time-servers"];

const LINK_LOCAL_ADDRESS: &str = "169.254.169.123";
const OLD_AMAZON_POOL_ADDRESS: &str = "2.amazon.pool.ntp.org";
const AMAZON_POOL_ADDRESS: &str = "time.aws.com";
const LEGACY_DEFAULT_OPTIONS: &[&str] = &["iburst"];
const LINK_LOCAL_OPTIONS: &[&str] = &["prefer", "iburst", "minpoll 4", "maxpoll 4"];

/// Converts the legacy NTP server list to and from named per-server settings.
pub struct NtpTimeServersMigration;

impl Migration for NtpTimeServersMigration {
    fn forward(&mut self, input: MigrationData) -> Result<MigrationData> {
        let had_legacy_servers = input.data.contains_key(LEGACY_SERVERS);
        let legacy_options = input.data.get(LEGACY_OPTIONS).cloned();
        let server_metadata = input.metadata.get(LEGACY_SERVERS).cloned();
        let options_metadata = input.metadata.get(LEGACY_OPTIONS).cloned();
        let mut output = migrate_list_to_named_map(
            input,
            LEGACY_SERVERS,
            NAMED_SERVER_PREFIX_SEGMENTS,
            |servers| {
                Ok(legacy_servers_to_named_map(
                    servers,
                    legacy_options.as_ref(),
                ))
            },
        )?;
        if had_legacy_servers && !output.data.contains_key(LEGACY_SERVERS) {
            output.data.remove(LEGACY_OPTIONS);
            output.metadata.remove(LEGACY_SERVERS);
            output.metadata.remove(LEGACY_OPTIONS);
            let named_servers =
                read_flattened_named_map(&output.data, NAMED_SERVER_PREFIX_SEGMENTS)?;
            let named_metadata = named_server_metadata(
                named_servers.as_ref(),
                server_metadata.as_ref(),
                options_metadata.as_ref(),
            );
            replace_flattened_named_map(
                &mut output.metadata,
                NAMED_SERVER_PREFIX_SEGMENTS,
                &named_metadata,
            )?;
        }
        Ok(output)
    }

    fn backward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        input.data.remove(LOGGING);
        input.metadata.remove(LOGGING);
        let named_metadata =
            read_flattened_named_map(&input.metadata, NAMED_SERVER_PREFIX_SEGMENTS)?;
        let server_metadata = first_field_metadata(named_metadata.as_ref(), "address");
        let options_metadata = first_field_metadata(named_metadata.as_ref(), "options");
        let mut converted_options = None;
        let mut output = migrate_named_map_to_list(
            input,
            NAMED_SERVER_PREFIX_SEGMENTS,
            LEGACY_SERVERS,
            |named_servers| {
                let Some(legacy) = named_map_to_legacy_servers(named_servers) else {
                    converted_options = Some(None);
                    return Ok(Some(ListReplacement::Remove));
                };
                converted_options = Some(Some(legacy.options));
                Ok(Some(ListReplacement::Replace(legacy.servers)))
            },
        )?;

        replace_flattened_named_map(
            &mut output.metadata,
            NAMED_SERVER_PREFIX_SEGMENTS,
            &BTreeMap::<String, BTreeMap<String, Metadata>>::new(),
        )?;
        if let Some(options) = converted_options {
            output.data.remove(LEGACY_OPTIONS);
            output.metadata.remove(LEGACY_SERVERS);
            output.metadata.remove(LEGACY_OPTIONS);
            if output.data.contains_key(LEGACY_SERVERS) {
                if let Some(metadata) = server_metadata {
                    output.metadata.insert(LEGACY_SERVERS.into(), metadata);
                }
            }
            if let Some(options) = options {
                output
                    .data
                    .insert(LEGACY_OPTIONS.into(), Value::Array(options));
                if let Some(metadata) = options_metadata {
                    output.metadata.insert(LEGACY_OPTIONS.into(), metadata);
                }
            }
        }
        Ok(output)
    }
}

#[derive(Default)]
struct ServerFields {
    address: Option<Value>,
    options: Option<Vec<Value>>,
}

#[derive(Default)]
struct LegacyNtpSettings {
    servers: Vec<Value>,
    options: Vec<Value>,
}

#[derive(Clone, Copy)]
enum LegacyDefaults {
    Aws,
    Public,
}

fn legacy_defaults(servers: &[Value], options: Option<&[Value]>) -> Option<LegacyDefaults> {
    if options != Some(strings_to_values(LEGACY_DEFAULT_OPTIONS).as_slice()) {
        return None;
    }

    if servers == strings_to_values(&[LINK_LOCAL_ADDRESS, OLD_AMAZON_POOL_ADDRESS]) {
        Some(LegacyDefaults::Aws)
    } else if servers == strings_to_values(&[OLD_AMAZON_POOL_ADDRESS]) {
        Some(LegacyDefaults::Public)
    } else {
        None
    }
}

fn legacy_servers_to_named_map(
    servers: Vec<Value>,
    legacy_options: Option<&Value>,
) -> Option<BTreeMap<String, BTreeMap<String, Value>>> {
    if servers.iter().any(|server| !server.is_string()) {
        println!("Found invalid '{LEGACY_SERVERS}' list; leaving NTP unchanged");
        return None;
    }

    let shared_options = match legacy_options {
        Some(Value::Array(options)) if options.iter().all(Value::is_string) => {
            Some(options.clone())
        }
        Some(value) => {
            println!("Found invalid '{LEGACY_OPTIONS}' value ('{value}'); leaving NTP unchanged");
            return None;
        }
        None => None,
    };
    let legacy_defaults = legacy_defaults(&servers, shared_options.as_deref());

    let mut named_servers = BTreeMap::new();
    let mut names = HashSet::new();
    for (index, value) in servers.into_iter().enumerate() {
        let address = value
            .as_str()
            .expect("NTP server entries were checked above");
        let (base_name, address, directive, options) = match (legacy_defaults, address) {
            (Some(LegacyDefaults::Aws), LINK_LOCAL_ADDRESS) => (
                "link-local",
                LINK_LOCAL_ADDRESS,
                "server",
                Some(strings_to_values(LINK_LOCAL_OPTIONS)),
            ),
            (Some(LegacyDefaults::Aws | LegacyDefaults::Public), OLD_AMAZON_POOL_ADDRESS) => (
                "amazon-pool",
                AMAZON_POOL_ADDRESS,
                "pool",
                shared_options.clone(),
            ),
            _ => ("time-server", address, "pool", shared_options.clone()),
        };
        let name = unique_name(base_name, index, &mut names);

        let mut fields = BTreeMap::from([
            ("address".into(), Value::String(address.into())),
            ("directive".into(), Value::String(directive.into())),
        ]);
        if let Some(options) = options {
            fields.insert("options".into(), Value::Array(options));
        }
        named_servers.insert(name, fields);
    }

    Some(named_servers)
}

fn named_map_to_legacy_servers(
    named_servers: BTreeMap<String, BTreeMap<String, Value>>,
) -> Option<LegacyNtpSettings> {
    let mut servers = Vec::new();
    for (name, fields) in named_servers {
        let Some(address) = fields.get("address") else {
            println!("Named NTP server '{name}' has no address; removing named NTP settings");
            return None;
        };
        if !address.is_string() {
            println!("Found invalid named NTP address ('{address}'); removing named NTP settings");
            return None;
        }

        let options = match fields.get("options") {
            Some(value) => {
                let Some(options) = value.as_array() else {
                    println!(
                        "Found invalid named NTP options ('{value}'); removing named NTP settings"
                    );
                    return None;
                };
                if options.iter().any(|option| !option.is_string()) {
                    println!(
                        "Found invalid named NTP options ('{value}'); removing named NTP settings"
                    );
                    return None;
                }
                Some(options.clone())
            }
            None => None,
        };

        servers.push((
            name,
            ServerFields {
                address: Some(address.clone()),
                options,
            },
        ));
    }

    servers.sort_by(|left, right| server_sort_key(&left.0).cmp(&server_sort_key(&right.0)));
    let addresses = servers
        .iter()
        .filter_map(|(_, server)| server.address.clone())
        .collect();
    let options = common_options(&servers);
    Some(LegacyNtpSettings {
        servers: addresses,
        options,
    })
}

#[cfg(test)]
fn named_key(name: &str, field: &str) -> String {
    format!("{NAMED_SERVER_PREFIX}{name}.{field}")
}

fn unique_name(base: &str, index: usize, names: &mut HashSet<String>) -> String {
    if names.insert(base.into()) {
        return base.into();
    }

    let mut suffix = index.max(1);
    loop {
        let candidate = format!("{base}-{suffix}");
        if names.insert(candidate.clone()) {
            return candidate;
        }
        suffix += 1;
    }
}

fn strings_to_values(values: &[&str]) -> Vec<Value> {
    values.iter().map(|value| (*value).into()).collect()
}

fn named_server_metadata(
    named_servers: Option<&BTreeMap<String, BTreeMap<String, Value>>>,
    server_metadata: Option<&Metadata>,
    options_metadata: Option<&Metadata>,
) -> BTreeMap<String, BTreeMap<String, Metadata>> {
    let mut result = BTreeMap::new();
    let Some(named_servers) = named_servers else {
        return result;
    };

    for (name, fields) in named_servers {
        let metadata: BTreeMap<String, Metadata> = fields
            .keys()
            .filter_map(|field| {
                let value = if field == "options" {
                    options_metadata
                } else {
                    server_metadata
                };
                value.cloned().map(|value| (field.clone(), value))
            })
            .collect();
        if !metadata.is_empty() {
            result.insert(name.clone(), metadata);
        }
    }
    result
}

fn first_field_metadata(
    named_metadata: Option<&BTreeMap<String, BTreeMap<String, Metadata>>>,
    field: &str,
) -> Option<Metadata> {
    named_metadata?
        .iter()
        .filter_map(|(name, fields)| fields.get(field).map(|metadata| (name, metadata)))
        .min_by(|left, right| server_sort_key(left.0).cmp(&server_sort_key(right.0)))
        .map(|(_, metadata)| metadata.clone())
}

fn server_sort_key(name: &str) -> (u8, &str) {
    match name {
        "link-local" => (0, name),
        "amazon-pool" => (1, name),
        _ => (2, name),
    }
}

fn common_options(servers: &[(String, ServerFields)]) -> Vec<Value> {
    let servers: Vec<&ServerFields> = servers
        .iter()
        .filter_map(|(_, server)| server.address.as_ref().map(|_| server))
        .collect();
    let Some(first) = servers.first().and_then(|server| server.options.clone()) else {
        return Vec::new();
    };
    if servers.iter().any(|server| server.options.is_none()) {
        return Vec::new();
    }

    // The old model has one shared list, so only options valid for every server survive rollback.
    first
        .into_iter()
        .filter(|option| {
            servers.iter().skip(1).all(|server| {
                server
                    .options
                    .as_ref()
                    .is_some_and(|options| options.contains(option))
            })
        })
        .collect()
}

fn run() -> Result<()> {
    migrate(NtpTimeServersMigration)
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use maplit::hashmap;
    use std::collections::HashMap;

    fn data(values: HashMap<String, Value>) -> MigrationData {
        MigrationData {
            data: values,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn forward_migrates_design_defaults() {
        let input = data(hashmap! {
            LEGACY_SERVERS.into() => vec![LINK_LOCAL_ADDRESS, OLD_AMAZON_POOL_ADDRESS].into(),
            LEGACY_OPTIONS.into() => vec!["iburst"].into(),
        });

        let result = NtpTimeServersMigration.forward(input).unwrap();
        assert_eq!(
            result.data,
            hashmap! {
                named_key("link-local", "address") => LINK_LOCAL_ADDRESS.into(),
                named_key("link-local", "directive") => "server".into(),
                named_key("link-local", "options") => LINK_LOCAL_OPTIONS.into(),
                named_key("amazon-pool", "address") => AMAZON_POOL_ADDRESS.into(),
                named_key("amazon-pool", "directive") => "pool".into(),
                named_key("amazon-pool", "options") => vec!["iburst"].into(),
            }
        );
    }

    #[test]
    fn forward_migrates_public_defaults() {
        let input = data(hashmap! {
            LEGACY_SERVERS.into() => vec![OLD_AMAZON_POOL_ADDRESS].into(),
            LEGACY_OPTIONS.into() => vec!["iburst"].into(),
        });

        let result = NtpTimeServersMigration.forward(input).unwrap();
        assert_eq!(
            result.data,
            hashmap! {
                named_key("amazon-pool", "address") => AMAZON_POOL_ADDRESS.into(),
                named_key("amazon-pool", "directive") => "pool".into(),
                named_key("amazon-pool", "options") => vec!["iburst"].into(),
            }
        );
    }

    #[test]
    fn forward_preserves_custom_servers_and_options() {
        let input = data(hashmap! {
            LEGACY_SERVERS.into() => vec!["ntp1.example.com", "ntp2.example.com"].into(),
            LEGACY_OPTIONS.into() => vec!["iburst", "maxpoll 8"].into(),
        });

        let result = NtpTimeServersMigration.forward(input).unwrap();
        assert_eq!(
            result.data,
            hashmap! {
                named_key("time-server", "address") => "ntp1.example.com".into(),
                named_key("time-server", "directive") => "pool".into(),
                named_key("time-server", "options") => vec!["iburst", "maxpoll 8"].into(),
                named_key("time-server-1", "address") => "ntp2.example.com".into(),
                named_key("time-server-1", "directive") => "pool".into(),
                named_key("time-server-1", "options") => vec!["iburst", "maxpoll 8"].into(),
            }
        );
    }

    #[test]
    fn forward_preserves_custom_config_using_default_addresses() {
        let input = data(hashmap! {
            LEGACY_SERVERS.into() => vec![LINK_LOCAL_ADDRESS, OLD_AMAZON_POOL_ADDRESS].into(),
            LEGACY_OPTIONS.into() => vec!["noselect"].into(),
        });

        let result = NtpTimeServersMigration.forward(input).unwrap();
        assert_eq!(
            result.data,
            hashmap! {
                named_key("time-server", "address") => LINK_LOCAL_ADDRESS.into(),
                named_key("time-server", "directive") => "pool".into(),
                named_key("time-server", "options") => vec!["noselect"].into(),
                named_key("time-server-1", "address") => OLD_AMAZON_POOL_ADDRESS.into(),
                named_key("time-server-1", "directive") => "pool".into(),
                named_key("time-server-1", "options") => vec!["noselect"].into(),
            }
        );
    }

    #[test]
    fn forward_preserves_custom_subset_of_default_servers() {
        let input = data(hashmap! {
            LEGACY_SERVERS.into() => vec![LINK_LOCAL_ADDRESS].into(),
            LEGACY_OPTIONS.into() => vec!["iburst"].into(),
        });

        let result = NtpTimeServersMigration.forward(input).unwrap();
        assert_eq!(
            result.data,
            hashmap! {
                named_key("time-server", "address") => LINK_LOCAL_ADDRESS.into(),
                named_key("time-server", "directive") => "pool".into(),
                named_key("time-server", "options") => vec!["iburst"].into(),
            }
        );
    }

    #[test]
    fn forward_handles_sparse_invalid_and_mixed_data() {
        let sparse = data(HashMap::new());
        assert_eq!(
            NtpTimeServersMigration.forward(sparse.clone()).unwrap(),
            sparse
        );

        let options_only = data(hashmap! {
            LEGACY_OPTIONS.into() => vec!["iburst"].into(),
        });
        assert_eq!(
            NtpTimeServersMigration
                .forward(options_only.clone())
                .unwrap(),
            options_only
        );

        let invalid = data(hashmap! {
            LEGACY_SERVERS.into() => "not-a-list".into(),
        });
        assert_eq!(
            NtpTimeServersMigration.forward(invalid.clone()).unwrap(),
            invalid
        );

        let mixed = data(hashmap! {
            LEGACY_SERVERS.into() => vec![LINK_LOCAL_ADDRESS].into(),
            named_key("stale", "address") => "stale.example.com".into(),
        });
        let result = NtpTimeServersMigration.forward(mixed).unwrap();
        assert!(!result.data.keys().any(|key| key.contains(".stale.")));
    }

    #[test]
    fn forward_handles_empty_list() {
        // An empty named map has no flattened keys, so storewolf can populate defaults later.
        let input = data(hashmap! {
            LEGACY_SERVERS.into() => Vec::<String>::new().into(),
            LEGACY_OPTIONS.into() => vec!["iburst"].into(),
        });

        let result = NtpTimeServersMigration.forward(input).unwrap();
        assert!(result.data.is_empty());
    }

    #[test]
    fn forward_migrates_metadata() {
        let server_metadata = hashmap! {
            "strength".into() => Value::String("strong".into()),
        };
        let options_metadata = hashmap! {
            "strength".into() => Value::String("weak".into()),
        };
        let input = MigrationData {
            data: hashmap! {
                LEGACY_SERVERS.into() => vec![LINK_LOCAL_ADDRESS, OLD_AMAZON_POOL_ADDRESS].into(),
                LEGACY_OPTIONS.into() => vec!["iburst"].into(),
            },
            metadata: hashmap! {
                LEGACY_SERVERS.into() => server_metadata.clone(),
                LEGACY_OPTIONS.into() => options_metadata.clone(),
            },
        };

        let result = NtpTimeServersMigration.forward(input).unwrap();
        assert_eq!(
            result.metadata,
            hashmap! {
                named_key("link-local", "address") => server_metadata.clone(),
                named_key("link-local", "directive") => server_metadata.clone(),
                named_key("link-local", "options") => options_metadata.clone(),
                named_key("amazon-pool", "address") => server_metadata.clone(),
                named_key("amazon-pool", "directive") => server_metadata,
                named_key("amazon-pool", "options") => options_metadata,
            }
        );
    }

    #[test]
    fn backward_restores_legacy_shape_and_removes_logging() {
        let input = data(hashmap! {
            named_key("link-local", "address") => LINK_LOCAL_ADDRESS.into(),
            named_key("link-local", "directive") => "server".into(),
            named_key("link-local", "options") => LINK_LOCAL_OPTIONS.into(),
            named_key("amazon-pool", "address") => AMAZON_POOL_ADDRESS.into(),
            named_key("amazon-pool", "directive") => "pool".into(),
            named_key("amazon-pool", "options") => vec!["iburst"].into(),
            LOGGING.into() => vec!["measurements", "statistics", "tracking"].into(),
        });

        let result = NtpTimeServersMigration.backward(input).unwrap();
        assert_eq!(
            result.data,
            hashmap! {
                LEGACY_SERVERS.into() => vec![LINK_LOCAL_ADDRESS, AMAZON_POOL_ADDRESS].into(),
                LEGACY_OPTIONS.into() => vec!["iburst"].into(),
            }
        );
    }

    #[test]
    fn backward_writes_empty_shared_options_when_servers_have_none_in_common() {
        let input = data(hashmap! {
            named_key("first", "address") => "ntp1.example.com".into(),
            named_key("first", "directive") => "pool".into(),
            named_key("first", "options") => vec!["iburst"].into(),
            named_key("second", "address") => "ntp2.example.com".into(),
            named_key("second", "directive") => "pool".into(),
        });

        let result = NtpTimeServersMigration.backward(input).unwrap();
        assert_eq!(
            result.data,
            hashmap! {
                LEGACY_SERVERS.into() => vec!["ntp1.example.com", "ntp2.example.com"].into(),
                LEGACY_OPTIONS.into() => Vec::<String>::new().into(),
            }
        );
    }

    #[test]
    fn backward_migrates_metadata_and_removes_logging_metadata() {
        let server_metadata = hashmap! {
            "strength".into() => Value::String("strong".into()),
        };
        let options_metadata = hashmap! {
            "strength".into() => Value::String("weak".into()),
        };
        let logging_metadata = hashmap! {
            "strength".into() => Value::String("strong".into()),
        };
        let input = MigrationData {
            data: hashmap! {
                named_key("link-local", "address") => LINK_LOCAL_ADDRESS.into(),
                named_key("link-local", "directive") => "server".into(),
                named_key("link-local", "options") => LINK_LOCAL_OPTIONS.into(),
                named_key("amazon-pool", "address") => AMAZON_POOL_ADDRESS.into(),
                named_key("amazon-pool", "directive") => "pool".into(),
                named_key("amazon-pool", "options") => vec!["iburst"].into(),
                LOGGING.into() => vec!["tracking"].into(),
            },
            metadata: hashmap! {
                named_key("link-local", "address") => server_metadata.clone(),
                named_key("link-local", "directive") => server_metadata.clone(),
                named_key("link-local", "options") => options_metadata.clone(),
                named_key("amazon-pool", "address") => server_metadata.clone(),
                named_key("amazon-pool", "directive") => server_metadata.clone(),
                named_key("amazon-pool", "options") => options_metadata.clone(),
                LOGGING.into() => logging_metadata,
            },
        };

        let result = NtpTimeServersMigration.backward(input).unwrap();
        assert_eq!(
            result.metadata,
            hashmap! {
                LEGACY_SERVERS.into() => server_metadata,
                LEGACY_OPTIONS.into() => options_metadata,
            }
        );
    }

    #[test]
    fn backward_removes_unrepresentable_named_data() {
        for input in [
            data(hashmap! {
                named_key("partial", "options") => vec!["iburst"].into(),
                "settings.other".into() => true.into(),
            }),
            data(hashmap! {
                named_key("invalid", "address") => vec!["not-a-string"].into(),
                "settings.other".into() => true.into(),
            }),
            data(hashmap! {
                named_key("invalid", "address") => "ntp.example.com".into(),
                named_key("invalid", "options") => "not-a-list".into(),
                "settings.other".into() => true.into(),
            }),
            data(hashmap! {
                named_key("invalid", "address") => "ntp.example.com".into(),
                named_key("invalid", "options") => vec![Value::Bool(true)].into(),
                "settings.other".into() => true.into(),
            }),
        ] {
            assert_eq!(
                NtpTimeServersMigration.backward(input).unwrap(),
                data(hashmap! {
                    "settings.other".into() => true.into(),
                })
            );
        }

        let input = MigrationData {
            data: hashmap! {
                named_key("partial", "options") => vec!["iburst"].into(),
                "settings.other".into() => true.into(),
            },
            metadata: hashmap! {
                named_key("partial", "options") => hashmap! {
                    "strength".into() => Value::String("strong".into()),
                },
            },
        };
        assert_eq!(
            NtpTimeServersMigration.backward(input).unwrap(),
            data(hashmap! {
                "settings.other".into() => true.into(),
            })
        );
    }
}
