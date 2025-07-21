use crate::bottlerocket::client::{
    HostContainer, KernelSettings, KubernetesSettings, NetworkSettings, NtpSettings, Settings,
};
use serde_json::Value;
use std::collections::HashMap;

/// Result of comparing two configurations
#[derive(Debug, Clone)]
pub struct ConfigDiff {
    /// List of detected drifts
    pub drifts: Vec<DriftItem>,
    /// Overall severity of the drift
    pub severity: DriftSeverity,
}

/// Individual drift item
#[derive(Debug, Clone)]
pub struct DriftItem {
    /// Field path (e.g., "kubernetes.api_server")
    pub field_path: String,
    /// Expected value from desired config
    pub expected: String,
    /// Actual value from current settings
    pub actual: String,
    /// Severity of this drift
    pub severity: DriftSeverity,
}

/// Severity levels for configuration drift
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DriftSeverity {
    /// Informational drift (e.g., comments, descriptions)
    Info,
    /// Warning drift (e.g., non-critical settings)
    Warning,
    /// Critical drift (e.g., API endpoints, certificates)
    Critical,
}

impl std::fmt::Display for DriftSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DriftSeverity::Info => write!(f, "info"),
            DriftSeverity::Warning => write!(f, "warning"),
            DriftSeverity::Critical => write!(f, "critical"),
        }
    }
}

impl ConfigDiff {
    /// Check if any drift was detected
    pub fn has_drift(&self) -> bool {
        !self.drifts.is_empty()
    }

    /// Get a summary of the drift
    pub fn summary(&self) -> String {
        if self.drifts.is_empty() {
            return "No drift detected".to_string();
        }

        let critical_count = self.drifts.iter().filter(|d| d.severity == DriftSeverity::Critical).count();
        let warning_count = self.drifts.iter().filter(|d| d.severity == DriftSeverity::Warning).count();
        let info_count = self.drifts.iter().filter(|d| d.severity == DriftSeverity::Info).count();

        format!(
            "Detected {} drift(s): {} critical, {} warning, {} info",
            self.drifts.len(),
            critical_count,
            warning_count,
            info_count
        )
    }
}

/// Compare two Settings structures and return drift
pub fn compare_settings(desired: &Settings, actual: &Settings) -> ConfigDiff {
    let mut drifts = Vec::new();

    // Compare MOTD
    compare_option_field(&mut drifts, "motd", &desired.motd, &actual.motd, DriftSeverity::Info);

    // Compare Kubernetes settings
    if desired.kubernetes.is_some() || actual.kubernetes.is_some() {
        compare_kubernetes_settings(
            &mut drifts,
            desired.kubernetes.as_ref(),
            actual.kubernetes.as_ref(),
        );
    }

    // Compare Network settings
    if desired.network.is_some() || actual.network.is_some() {
        compare_network_settings(
            &mut drifts,
            desired.network.as_ref(),
            actual.network.as_ref(),
        );
    }

    // Compare Kernel settings
    if desired.kernel.is_some() || actual.kernel.is_some() {
        compare_kernel_settings(
            &mut drifts,
            desired.kernel.as_ref(),
            actual.kernel.as_ref(),
        );
    }

    // Compare Host containers
    if desired.host_containers.is_some() || actual.host_containers.is_some() {
        compare_host_containers(
            &mut drifts,
            desired.host_containers.as_ref(),
            actual.host_containers.as_ref(),
        );
    }

    // Compare NTP settings
    if desired.ntp.is_some() || actual.ntp.is_some() {
        compare_ntp_settings(&mut drifts, desired.ntp.as_ref(), actual.ntp.as_ref());
    }

    // Determine overall severity
    let severity = drifts
        .iter()
        .map(|d| d.severity)
        .max()
        .unwrap_or(DriftSeverity::Info);

    ConfigDiff { drifts, severity }
}

/// Compare Kubernetes settings
fn compare_kubernetes_settings(
    drifts: &mut Vec<DriftItem>,
    desired: Option<&KubernetesSettings>,
    actual: Option<&KubernetesSettings>,
) {
    match (desired, actual) {
        (Some(d), Some(a)) => {
            compare_option_field(
                drifts,
                "kubernetes.api_server",
                &d.api_server,
                &a.api_server,
                DriftSeverity::Critical,
            );
            compare_option_field(
                drifts,
                "kubernetes.cluster_certificate",
                &d.cluster_certificate,
                &a.cluster_certificate,
                DriftSeverity::Critical,
            );
            compare_option_field(
                drifts,
                "kubernetes.cluster_dns_ip",
                &d.cluster_dns_ip,
                &a.cluster_dns_ip,
                DriftSeverity::Warning,
            );
            compare_option_field(
                drifts,
                "kubernetes.cluster_domain",
                &d.cluster_domain,
                &a.cluster_domain,
                DriftSeverity::Warning,
            );
        }
        (Some(_), None) => {
            drifts.push(DriftItem {
                field_path: "kubernetes".to_string(),
                expected: "configured".to_string(),
                actual: "not configured".to_string(),
                severity: DriftSeverity::Critical,
            });
        }
        (None, Some(_)) => {
            drifts.push(DriftItem {
                field_path: "kubernetes".to_string(),
                expected: "not configured".to_string(),
                actual: "configured".to_string(),
                severity: DriftSeverity::Warning,
            });
        }
        (None, None) => {}
    }
}

/// Compare Network settings
fn compare_network_settings(
    drifts: &mut Vec<DriftItem>,
    desired: Option<&NetworkSettings>,
    actual: Option<&NetworkSettings>,
) {
    match (desired, actual) {
        (Some(d), Some(a)) => {
            compare_option_field(
                drifts,
                "network.hostname",
                &d.hostname,
                &a.hostname,
                DriftSeverity::Critical,
            );
        }
        (Some(_), None) => {
            drifts.push(DriftItem {
                field_path: "network".to_string(),
                expected: "configured".to_string(),
                actual: "not configured".to_string(),
                severity: DriftSeverity::Warning,
            });
        }
        (None, Some(_)) => {
            drifts.push(DriftItem {
                field_path: "network".to_string(),
                expected: "not configured".to_string(),
                actual: "configured".to_string(),
                severity: DriftSeverity::Info,
            });
        }
        (None, None) => {}
    }
}

/// Compare Kernel settings
fn compare_kernel_settings(
    drifts: &mut Vec<DriftItem>,
    desired: Option<&KernelSettings>,
    actual: Option<&KernelSettings>,
) {
    match (desired, actual) {
        (Some(d), Some(a)) => {
            compare_option_field(
                drifts,
                "kernel.lockdown",
                &d.lockdown,
                &a.lockdown,
                DriftSeverity::Critical,
            );
            
            // Compare sysctl settings
            if let (Some(d_sysctl), Some(a_sysctl)) = (&d.sysctl, &a.sysctl) {
                compare_hashmaps(drifts, "kernel.sysctl", d_sysctl, a_sysctl, DriftSeverity::Warning);
            }
        }
        (Some(_), None) => {
            drifts.push(DriftItem {
                field_path: "kernel".to_string(),
                expected: "configured".to_string(),
                actual: "not configured".to_string(),
                severity: DriftSeverity::Warning,
            });
        }
        (None, Some(_)) => {
            // Kernel settings exist but weren't expected - could be defaults
        }
        (None, None) => {}
    }
}

/// Compare Host containers
fn compare_host_containers(
    drifts: &mut Vec<DriftItem>,
    desired: Option<&HashMap<String, HostContainer>>,
    actual: Option<&HashMap<String, HostContainer>>,
) {
    match (desired, actual) {
        (Some(d), Some(a)) => {
            // Check for containers in desired but not in actual
            for (name, d_container) in d {
                if let Some(a_container) = a.get(name) {
                    compare_host_container(drifts, name, d_container, a_container);
                } else {
                    drifts.push(DriftItem {
                        field_path: format!("host_containers.{}", name),
                        expected: "configured".to_string(),
                        actual: "not configured".to_string(),
                        severity: DriftSeverity::Warning,
                    });
                }
            }
            
            // Check for containers in actual but not in desired
            for name in a.keys() {
                if !d.contains_key(name) {
                    drifts.push(DriftItem {
                        field_path: format!("host_containers.{}", name),
                        expected: "not configured".to_string(),
                        actual: "configured".to_string(),
                        severity: DriftSeverity::Info,
                    });
                }
            }
        }
        (Some(_), None) => {
            drifts.push(DriftItem {
                field_path: "host_containers".to_string(),
                expected: "configured".to_string(),
                actual: "not configured".to_string(),
                severity: DriftSeverity::Warning,
            });
        }
        (None, Some(_)) => {
            // Host containers exist but weren't expected
        }
        (None, None) => {}
    }
}

/// Compare individual host container
fn compare_host_container(
    drifts: &mut Vec<DriftItem>,
    name: &str,
    desired: &HostContainer,
    actual: &HostContainer,
) {
    let prefix = format!("host_containers.{}", name);
    
    if desired.enabled != actual.enabled {
        drifts.push(DriftItem {
            field_path: format!("{}.enabled", prefix),
            expected: desired.enabled.to_string(),
            actual: actual.enabled.to_string(),
            severity: DriftSeverity::Warning,
        });
    }
    
    compare_option_field(
        drifts,
        &format!("{}.source", prefix),
        &desired.source,
        &actual.source,
        DriftSeverity::Warning,
    );
    
    compare_option_field(
        drifts,
        &format!("{}.superpowered", prefix),
        &desired.superpowered,
        &actual.superpowered,
        DriftSeverity::Warning,
    );
}

/// Compare NTP settings
fn compare_ntp_settings(
    drifts: &mut Vec<DriftItem>,
    desired: Option<&NtpSettings>,
    actual: Option<&NtpSettings>,
) {
    match (desired, actual) {
        (Some(d), Some(a)) => {
            if let (Some(d_servers), Some(a_servers)) = (&d.time_servers, &a.time_servers) {
                if d_servers != a_servers {
                    drifts.push(DriftItem {
                        field_path: "ntp.time_servers".to_string(),
                        expected: format!("{:?}", d_servers),
                        actual: format!("{:?}", a_servers),
                        severity: DriftSeverity::Warning,
                    });
                }
            }
        }
        _ => {}
    }
}

/// Helper to compare optional fields
fn compare_option_field<T: std::fmt::Display + PartialEq>(
    drifts: &mut Vec<DriftItem>,
    field_path: &str,
    desired: &Option<T>,
    actual: &Option<T>,
    severity: DriftSeverity,
) {
    match (desired, actual) {
        (Some(d), Some(a)) if d != a => {
            drifts.push(DriftItem {
                field_path: field_path.to_string(),
                expected: d.to_string(),
                actual: a.to_string(),
                severity,
            });
        }
        (Some(d), None) => {
            drifts.push(DriftItem {
                field_path: field_path.to_string(),
                expected: d.to_string(),
                actual: "not set".to_string(),
                severity,
            });
        }
        (None, Some(a)) => {
            drifts.push(DriftItem {
                field_path: field_path.to_string(),
                expected: "not set".to_string(),
                actual: a.to_string(),
                severity: DriftSeverity::Info, // Lower severity for unexpected fields
            });
        }
        _ => {}
    }
}

/// Helper to compare HashMaps
fn compare_hashmaps(
    drifts: &mut Vec<DriftItem>,
    prefix: &str,
    desired: &HashMap<String, String>,
    actual: &HashMap<String, String>,
    severity: DriftSeverity,
) {
    // Check for keys in desired but not in actual
    for (key, d_value) in desired {
        let field_path = format!("{}.{}", prefix, key);
        if let Some(a_value) = actual.get(key) {
            if d_value != a_value {
                drifts.push(DriftItem {
                    field_path,
                    expected: d_value.clone(),
                    actual: a_value.clone(),
                    severity,
                });
            }
        } else {
            drifts.push(DriftItem {
                field_path,
                expected: d_value.clone(),
                actual: "not set".to_string(),
                severity,
            });
        }
    }
    
    // Check for keys in actual but not in desired
    for (key, a_value) in actual {
        if !desired.contains_key(key) {
            drifts.push(DriftItem {
                field_path: format!("{}.{}", prefix, key),
                expected: "not set".to_string(),
                actual: a_value.clone(),
                severity: DriftSeverity::Info,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_drift() {
        let settings = Settings::default();
        let diff = compare_settings(&settings, &settings);
        assert!(!diff.has_drift());
        assert_eq!(diff.summary(), "No drift detected");
    }

    #[test]
    fn test_kubernetes_drift() {
        let mut desired = Settings::default();
        let mut actual = Settings::default();
        
        desired.kubernetes = Some(KubernetesSettings {
            api_server: Some("https://k8s.example.com:6443".to_string()),
            cluster_certificate: Some("cert-data".to_string()),
            cluster_dns_ip: Some("10.96.0.10".to_string()),
            cluster_domain: Some("cluster.local".to_string()),
            node_labels: None,
            node_taints: None,
        });
        
        actual.kubernetes = Some(KubernetesSettings {
            api_server: Some("https://different.example.com:6443".to_string()),
            cluster_certificate: Some("cert-data".to_string()),
            cluster_dns_ip: Some("10.96.0.10".to_string()),
            cluster_domain: Some("cluster.local".to_string()),
            node_labels: None,
            node_taints: None,
        });
        
        let diff = compare_settings(&desired, &actual);
        assert!(diff.has_drift());
        assert_eq!(diff.severity, DriftSeverity::Critical);
        assert_eq!(diff.drifts.len(), 1);
        assert_eq!(diff.drifts[0].field_path, "kubernetes.api_server");
    }
}