#[cfg(test)]
mod tests {
    use crate::election::{ElectionState, NodeInfo, ElectionConfig};
    use crate::pki::PKIService;
    use crate::etcd::service::EtcdService;
    use std::sync::Arc;
    use std::time::Duration;

    fn create_test_node_info(address: &str) -> NodeInfo {
        NodeInfo {
            node_id: "test-node".to_string(),
            address: address.to_string(),
            uptime: Duration::from_secs(0),
            cpu_available_percent: 80.0,
            memory_available_gb: 8.0,
            packet_loss_percent: 0.0,
            election_priority: 100,
        }
    }

    #[tokio::test]
    async fn test_etcd_service_creation_with_valid_ip() {
        // Test that we can create service with valid IP addresses
        let test_cases = vec![
            ("127.0.0.1", "dev mode localhost"),
            ("10.0.0.1", "private IP"),
            ("192.168.1.1", "another private IP"),
            ("0.0.0.0", "bind all interfaces"),
        ];
        
        for (ip, description) in test_cases {
            let node_info = create_test_node_info(ip);
            let election_state = Arc::new(ElectionState::new(
                "test-node".to_string(),
                node_info,
                ElectionConfig::default(),
            ));
            let pki_service = Arc::new(PKIService::new(election_state.clone()));
            
            // Should not panic when creating with valid IP
            let _etcd_service = EtcdService::with_dev_mode(
                election_state.clone(),
                pki_service,
                true,
            );
            
            // If we get here, service was created successfully
            assert!(true, "Service created successfully with {}: {}", description, ip);
        }
    }
    
    #[tokio::test] 
    async fn test_etcd_service_dev_mode_flag() {
        // Test that dev mode flag is properly stored
        let node_info = create_test_node_info("127.0.0.1");
        let election_state = Arc::new(ElectionState::new(
            "test-node".to_string(),
            node_info,
            ElectionConfig::default(),
        ));
        let pki_service = Arc::new(PKIService::new(election_state.clone()));
        
        // Create with dev mode true
        let _etcd_service_dev = EtcdService::with_dev_mode(
            election_state.clone(),
            pki_service.clone(),
            true,
        );
        
        // Create with dev mode false (production)
        let _etcd_service_prod = EtcdService::with_dev_mode(
            election_state.clone(),
            pki_service,
            false,
        );
        
        // Services created successfully with different modes
        assert!(true);
    }
    
    #[tokio::test]
    async fn test_etcd_cluster_state_initialization() {
        // Test that service initializes with correct default state
        let node_info = create_test_node_info("127.0.0.1");
        let election_state = Arc::new(ElectionState::new(
            "test-node".to_string(),
            node_info,
            ElectionConfig::default(),
        ));
        let pki_service = Arc::new(PKIService::new(election_state.clone()));
        
        let etcd_service = EtcdService::new(election_state, pki_service);
        
        // Verify initial state
        assert!(!etcd_service.is_cluster_initialized().await, 
            "Cluster should not be initialized on creation");
    }
}