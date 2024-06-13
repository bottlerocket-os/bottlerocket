use migration_helpers::{migrate, Migration, MigrationData, Result};
use std::process;

const CLUSTER_DNS_IP_KEY: &str = "settings.kubernetes.cluster-dns-ip";

/// We changed `settings.kubernetes.cluster-dns-ip` to support being either a string or a list of strings.
fn run() -> Result<()> {
    migrate(ClusterDNSIPListMigration)
}

struct ClusterDNSIPListMigration;

impl Migration for ClusterDNSIPListMigration {
    /// New versions allow the older string values to be present, so we don't need to do anything.
    fn forward(&mut self, input: MigrationData) -> Result<MigrationData> {
        println!("ClusterDNSIPListMigration has no work to do on upgrade.");
        Ok(input)
    }

    /// Older versions don't know about list-style settings, so we need to create a scalar setting using the first value.
    fn backward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        let maybe_prior_value = input.data.get(CLUSTER_DNS_IP_KEY);

        // If the current value is a string, don't touch it.
        if let Some(prior_value) = maybe_prior_value {
            if prior_value.is_string() {
                println!(
                    "{} is already a string value ('{}'), and does not require migration.",
                    CLUSTER_DNS_IP_KEY, prior_value
                );
                return Ok(input);
            }
        }

        // If the current value is an array and the first element is a string, that element becomes the new value.
        // Any other cases result in clearing the value.
        let new_value = maybe_prior_value
            .and_then(|dns_ip_value| {
                println!(
                    "Found existing value for '{}': '{}'",
                    CLUSTER_DNS_IP_KEY, dns_ip_value
                );
                dns_ip_value.as_array()
            })
            .and_then(|ip_array| ip_array.iter().next())
            .map(|ip_value| ip_value.clone());

        match new_value {
            Some(ip_value) if ip_value.is_string() => {
                input
                    .data
                    .insert(CLUSTER_DNS_IP_KEY.to_string(), ip_value.clone());
                println!(
                    "Replaced prior value for '{}' with '{}'",
                    CLUSTER_DNS_IP_KEY, ip_value
                );
            }
            _ => {
                println!(
                    "Prior value for '{}' was not recognized. Removing it.",
                    CLUSTER_DNS_IP_KEY
                );
                input.data.remove(CLUSTER_DNS_IP_KEY);
            }
        };

        Ok(input)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_downgrade_string() {
        let input = MigrationData {
            data: serde_json::from_str(r#"{"settings.kubernetes.cluster-dns-ip": "10.0.0.1"}"#)
                .unwrap(),
            metadata: HashMap::new(),
        };
        let expected = MigrationData {
            data: serde_json::from_str(r#"{"settings.kubernetes.cluster-dns-ip": "10.0.0.1"}"#)
                .unwrap(),
            metadata: HashMap::new(),
        };
        assert_eq!(ClusterDNSIPListMigration.backward(input).unwrap(), expected);
    }

    #[test]
    fn test_downgrade_list() {
        let test_cases = [
            (
                MigrationData {
                    data: serde_json::from_str(
                        r#"{"settings.kubernetes.cluster-dns-ip": ["10.0.0.1"]}"#,
                    )
                    .unwrap(),
                    metadata: HashMap::new(),
                },
                MigrationData {
                    data: serde_json::from_str(
                        r#"{"settings.kubernetes.cluster-dns-ip": "10.0.0.1"}"#,
                    )
                    .unwrap(),
                    metadata: HashMap::new(),
                },
            ),
            (
                MigrationData {
                    data: serde_json::from_str(r#"{"settings.kubernetes.cluster-dns-ip": []}"#)
                        .unwrap(),
                    metadata: HashMap::new(),
                },
                MigrationData {
                    data: HashMap::new(),
                    metadata: HashMap::new(),
                },
            ),
            (
                MigrationData {
                    data: serde_json::from_str(
                        r#"{"settings.kubernetes.cluster-dns-ip": ["10.0.0.2", "10.0.0.1"]}"#,
                    )
                    .unwrap(),
                    metadata: HashMap::new(),
                },
                MigrationData {
                    data: serde_json::from_str(
                        r#"{"settings.kubernetes.cluster-dns-ip": "10.0.0.2"}"#,
                    )
                    .unwrap(),
                    metadata: HashMap::new(),
                },
            ),
        ];
        for (input, expected) in test_cases.iter() {
            assert_eq!(
                ClusterDNSIPListMigration.backward(input.clone()).unwrap(),
                *expected
            );
        }
    }

    #[test]
    fn test_downgrade_other() {
        let test_cases = [
            (
                MigrationData {
                    data: serde_json::from_str(
                        r#"{"settings.kubernetes.cluster-dns-ip": {"1": 2}}"#,
                    )
                    .unwrap(),
                    metadata: HashMap::new(),
                },
                MigrationData {
                    data: HashMap::new(),
                    metadata: HashMap::new(),
                },
            ),
            (
                MigrationData {
                    data: serde_json::from_str(r#"{"settings.kubernetes.cluster-dns-ip": 56}"#)
                        .unwrap(),
                    metadata: HashMap::new(),
                },
                MigrationData {
                    data: HashMap::new(),
                    metadata: HashMap::new(),
                },
            ),
            (
                MigrationData {
                    data: serde_json::from_str(r#"{"settings.kubernetes.cluster-dns-ip": false}"#)
                        .unwrap(),
                    metadata: HashMap::new(),
                },
                MigrationData {
                    data: HashMap::new(),
                    metadata: HashMap::new(),
                },
            ),
        ];
        for (input, expected) in test_cases.iter() {
            assert_eq!(
                ClusterDNSIPListMigration.backward(input.clone()).unwrap(),
                *expected
            );
        }
    }
}

// Returning a Result from main makes it print a Debug representation of the error, but with Snafu
// we have nice Display representations of the error, so we wrap "main" (run) and print any error.
// https://github.com/shepmaster/snafu/issues/110
fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}
