#[cfg(test)]
mod tests {
    use platform_bootstrap::pki::{
        PKIConfig, CertificateAuthority, CertificateStore, PKIDistributor,
    };
    use platform_bootstrap::election::{ElectionState, NodeInfo, ElectionConfig};
    use platform_bootstrap::proto::pki::{CertificateRequest, CertificateType};
    use std::sync::Arc;
    use std::time::Duration;
    
    #[tokio::test]
    async fn test_pki_distribution_basic() {
        // Create election state (mock as leader)
        let node_info = NodeInfo {
            node_id: "test-node".to_string(),
            address: "127.0.0.1:50100".to_string(),
            uptime: Duration::from_secs(300),
            cpu_available_percent: 80.0,
            memory_available_gb: 8.0,
            packet_loss_percent: 0.0,
            election_priority: 100,
        };
        let config = ElectionConfig::default();
        let election_state = Arc::new(ElectionState::new("test-node".to_string(), node_info, config));
        
        // Create PKI components
        let store = Arc::new(CertificateStore::new());
        let distributor = Arc::new(PKIDistributor::new(store.clone(), election_state.clone()));
        
        // Create CA and generate hierarchy
        let pki_config = PKIConfig::default();
        let mut ca = CertificateAuthority::new(pki_config);
        ca.initialize().unwrap();
        
        // Store CAs
        store.store_root_ca(ca.get_root_ca().unwrap().clone()).await.unwrap();
        if let Some(k8s_ca) = ca.get_kubernetes_ca() {
            store.store_kubernetes_ca(k8s_ca.clone()).await.unwrap();
        }
        
        // Register auth token for test node
        distributor.register_auth_token("client-1".to_string(), "test-token".to_string()).await.unwrap();
        
        // Test certificate request (will fail because we're not leader)
        let request = CertificateRequest {
            common_name: "test.example.com".to_string(),
            r#type: CertificateType::Server as i32,
            dns_names: vec!["test.example.com".to_string()],
            ip_addresses: vec!["192.168.1.100".to_string()],
            email_addresses: vec![],
            validity_days: 365,
            organizations: vec![],
            organizational_units: vec![],
            node_id: "client-1".to_string(),
            csr: vec![],
            auth_token: "test-token".to_string(),
        };
        
        // This should fail as we're not the leader
        assert!(distributor.process_certificate_request(request).await.is_err());
    }
    
    #[tokio::test]
    async fn test_certificate_validation() {
        let store = Arc::new(CertificateStore::new());
        let node_info = NodeInfo {
            node_id: "test-node".to_string(),
            address: "127.0.0.1:50100".to_string(),
            uptime: Duration::from_secs(300),
            cpu_available_percent: 80.0,
            memory_available_gb: 8.0,
            packet_loss_percent: 0.0,
            election_priority: 100,
        };
        let config = ElectionConfig::default();
        let election_state = Arc::new(ElectionState::new("test-node".to_string(), node_info, config));
        let distributor = Arc::new(PKIDistributor::new(store.clone(), election_state));
        
        // Create and store a test CA
        let pki_config = PKIConfig::default();
        let mut ca = CertificateAuthority::new(pki_config);
        ca.initialize().unwrap();
        store.store_root_ca(ca.get_root_ca().unwrap().clone()).await.unwrap();
        
        // Get the CA certificate PEM
        let root_ca = ca.get_root_ca().unwrap();
        let (cert_pem, _) = root_ca.to_pem().unwrap();
        
        // Validate the certificate chain
        assert!(distributor.validate_certificate_chain(&cert_pem).await.is_ok());
    }
}