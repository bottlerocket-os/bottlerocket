#[cfg(test)]
mod tests {
    use crate::bottlerocket::client::{Settings, KubernetesSettings};
    use crate::reconciler::diff::{compare_settings, DriftSeverity, ConfigDiff, DriftItem};
    use crate::reconciler::config::{ReconcilerConfig, CorrectionThreshold};
    
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
    
    #[test]
    fn test_config_validation() {
        let mut config = ReconcilerConfig::default();
        assert!(config.validate().is_ok());
        
        config.interval_seconds = 10;
        assert!(config.validate().is_err());
        
        config.interval_seconds = 5000;
        assert!(config.validate().is_err());
        
        config.interval_seconds = 60;
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_field_patterns() {
        let config = ReconcilerConfig::default();
        
        // Test ignored fields
        assert!(config.should_ignore_field("motd"));
        assert!(config.should_ignore_field("host_containers.admin.user_data"));
        assert!(!config.should_ignore_field("kubernetes.api_server"));
        
        // Test critical fields
        assert!(config.is_critical_field("kubernetes.api_server"));
        assert!(config.is_critical_field("network.hostname"));
        assert!(!config.is_critical_field("motd"));
    }
    
    #[test]
    fn test_drift_summary() {
        let diff = ConfigDiff {
            drifts: vec![
                DriftItem {
                    field_path: "kubernetes.api_server".to_string(),
                    expected: "https://k8s.example.com".to_string(),
                    actual: "https://different.com".to_string(),
                    severity: DriftSeverity::Critical,
                },
                DriftItem {
                    field_path: "network.hostname".to_string(),
                    expected: "node1".to_string(),
                    actual: "node2".to_string(),
                    severity: DriftSeverity::Warning,
                },
            ],
            severity: DriftSeverity::Critical,
        };
        
        assert!(diff.has_drift());
        assert_eq!(diff.summary(), "Detected 2 drift(s): 1 critical, 1 warning, 0 info");
    }
}