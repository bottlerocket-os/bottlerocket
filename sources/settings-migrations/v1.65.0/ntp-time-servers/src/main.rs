use migration_helpers::{migrate, Migration, MigrationData, Result};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::process;

const LEGACY_SERVERS: &str = "settings.ntp.time-servers";
const LEGACY_OPTIONS: &str = "settings.ntp.options";
const LOGGING: &str = "settings.ntp.logging";
const NAMED_SERVER_PREFIX: &str = "settings.ntp.time-servers.";

const LINK_LOCAL_ADDRESS: &str = "169.254.169.123";
const OLD_AMAZON_POOL_ADDRESS: &str = "2.amazon.pool.ntp.org";
const AMAZON_POOL_ADDRESS: &str = "time.aws.com";
const LINK_LOCAL_OPTIONS: &[&str] = &["prefer", "iburst", "minpoll 4", "maxpoll 4"];

/// Convert the legacy NTP server list to and from named per-server settings.
pub struct NtpTimeServersMigration;

impl Migration for NtpTimeServersMigration {
    fn forward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        let servers = match input.data.get(LEGACY_SERVERS) {
            Some(Value::Array(servers)) if servers.iter().all(Value::is_string) => servers.clone(),
            Some(value) => {
                println!(
                    "Found invalid '{LEGACY_SERVERS}' value ('{value}'); leaving it unchanged"
                );
                return Ok(input);
            }
            None => {
                println!("Found no '{LEGACY_SERVERS}' to migrate on upgrade");
                return Ok(input);
            }
        };

        let shared_options = match input.data.get(LEGACY_OPTIONS) {
            Some(Value::Array(options)) if options.iter().all(Value::is_string) => {
                Some(options.clone())
            }
            Some(value) => {
                println!(
                    "Found invalid '{LEGACY_OPTIONS}' value ('{value}'); leaving NTP unchanged"
                );
                return Ok(input);
            }
            None => None,
        };

        let server_metadata = input.metadata.remove(LEGACY_SERVERS);
        let options_metadata = input.metadata.remove(LEGACY_OPTIONS);
        input.data.remove(LEGACY_SERVERS);
        input.data.remove(LEGACY_OPTIONS);
        remove_named_servers(&mut input);

        // Existing nodes need the link-local recovery settings from the new defaults.
        let mut names = HashSet::new();
        for (index, value) in servers.into_iter().enumerate() {
            let address = value
                .as_str()
                .expect("NTP server entries were checked above");
            let (base_name, address, directive, options) = match address {
                LINK_LOCAL_ADDRESS => (
                    "link-local",
                    LINK_LOCAL_ADDRESS,
                    "server",
                    Some(strings_to_values(LINK_LOCAL_OPTIONS)),
                ),
                OLD_AMAZON_POOL_ADDRESS => (
                    "amazon-pool",
                    AMAZON_POOL_ADDRESS,
                    "pool",
                    shared_options.clone(),
                ),
                _ => ("time-server", address, "pool", shared_options.clone()),
            };
            let name = unique_name(base_name, index, &mut names);
            let address_key = named_key(&name, "address");
            let directive_key = named_key(&name, "directive");

            input
                .data
                .insert(address_key.clone(), Value::String(address.into()));
            input
                .data
                .insert(directive_key.clone(), Value::String(directive.into()));

            if let Some(options) = options {
                let options_key = named_key(&name, "options");
                input
                    .data
                    .insert(options_key.clone(), Value::Array(options));
                if let Some(metadata) = options_metadata.clone() {
                    input.metadata.insert(options_key, metadata);
                }
            }

            if let Some(metadata) = server_metadata.clone() {
                input.metadata.insert(address_key, metadata.clone());
                input.metadata.insert(directive_key, metadata);
            }
        }

        Ok(input)
    }

    fn backward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        input.data.remove(LOGGING);
        input.metadata.remove(LOGGING);

        if input.data.contains_key(LEGACY_SERVERS) {
            remove_named_servers(&mut input);
            return Ok(input);
        }

        let named_keys: Vec<String> = input
            .data
            .keys()
            .filter(|key| key.starts_with(NAMED_SERVER_PREFIX))
            .cloned()
            .collect();
        if named_keys.is_empty() {
            println!("Found no named NTP servers to migrate on downgrade");
            return Ok(input);
        }

        let mut servers: BTreeMap<String, ServerFields> = BTreeMap::new();
        for key in &named_keys {
            let Some((name, field)) = named_parts(key) else {
                continue;
            };
            let server = servers.entry(name.into()).or_default();
            match field {
                "address" => server.address = input.data.get(key).cloned(),
                "options" => {
                    server.options = input.data.get(key).and_then(Value::as_array).cloned()
                }
                _ => {}
            }
        }

        let mut servers: Vec<(String, ServerFields)> = servers.into_iter().collect();
        servers.sort_by(|left, right| server_sort_key(&left.0).cmp(&server_sort_key(&right.0)));

        let addresses: Vec<Value> = servers
            .iter()
            .filter_map(|(_, server)| server.address.clone())
            .collect();
        let options = common_options(&servers);
        let server_metadata = first_metadata(&input, &servers, "address");
        let options_metadata = first_metadata(&input, &servers, "options");

        remove_named_servers(&mut input);
        if !addresses.is_empty() {
            input
                .data
                .insert(LEGACY_SERVERS.into(), Value::Array(addresses));
            if let Some(metadata) = server_metadata {
                input.metadata.insert(LEGACY_SERVERS.into(), metadata);
            }
        }
        if let Some(options) = options {
            input
                .data
                .insert(LEGACY_OPTIONS.into(), Value::Array(options));
            if let Some(metadata) = options_metadata {
                input.metadata.insert(LEGACY_OPTIONS.into(), metadata);
            }
        }

        Ok(input)
    }
}

#[derive(Default)]
struct ServerFields {
    address: Option<Value>,
    options: Option<Vec<Value>>,
}

fn named_key(name: &str, field: &str) -> String {
    format!("{NAMED_SERVER_PREFIX}{name}.{field}")
}

fn named_parts(key: &str) -> Option<(&str, &str)> {
    key.strip_prefix(NAMED_SERVER_PREFIX)?.split_once('.')
}

fn unique_name(base: &str, index: usize, names: &mut HashSet<String>) -> String {
    let name = if names.contains(base) {
        format!("{base}-{index}")
    } else {
        base.into()
    };
    names.insert(name.clone());
    name
}

fn strings_to_values(values: &[&str]) -> Vec<Value> {
    values.iter().map(|value| (*value).into()).collect()
}

fn remove_named_servers(input: &mut MigrationData) {
    input
        .data
        .retain(|key, _| !key.starts_with(NAMED_SERVER_PREFIX));
    input
        .metadata
        .retain(|key, _| !key.starts_with(NAMED_SERVER_PREFIX));
}

fn server_sort_key(name: &str) -> (u8, &str) {
    match name {
        "link-local" => (0, name),
        "amazon-pool" => (1, name),
        _ => (2, name),
    }
}

fn first_metadata(
    input: &MigrationData,
    servers: &[(String, ServerFields)],
    field: &str,
) -> Option<HashMap<String, Value>> {
    servers
        .iter()
        .find_map(|(name, _)| input.metadata.get(&named_key(name, field)).cloned())
}

fn common_options(servers: &[(String, ServerFields)]) -> Option<Vec<Value>> {
    let servers: Vec<&ServerFields> = servers
        .iter()
        .filter_map(|(_, server)| server.address.as_ref().map(|_| server))
        .collect();
    let first = servers.first()?.options.clone()?;
    if servers.iter().any(|server| server.options.is_none()) {
        return None;
    }

    // The old model has one shared list, so only options valid for every server survive rollback.
    Some(
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
            .collect(),
    )
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
    fn forward_handles_sparse_invalid_and_mixed_data() {
        let sparse = data(HashMap::new());
        assert_eq!(
            NtpTimeServersMigration.forward(sparse.clone()).unwrap(),
            sparse
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
    fn migration_preserves_strength_metadata() {
        let strength = hashmap! {
            "strength".into() => Value::String("strong".into()),
        };
        let input = MigrationData {
            data: hashmap! {
                LEGACY_SERVERS.into() => vec![LINK_LOCAL_ADDRESS, OLD_AMAZON_POOL_ADDRESS].into(),
                LEGACY_OPTIONS.into() => vec!["iburst"].into(),
            },
            metadata: hashmap! {
                LEGACY_SERVERS.into() => strength.clone(),
                LEGACY_OPTIONS.into() => strength.clone(),
            },
        };

        let migrated = NtpTimeServersMigration.forward(input).unwrap();
        let result = NtpTimeServersMigration.backward(migrated).unwrap();
        assert_eq!(result.metadata.get(LEGACY_SERVERS), Some(&strength));
        assert_eq!(result.metadata.get(LEGACY_OPTIONS), Some(&strength));
    }
}
