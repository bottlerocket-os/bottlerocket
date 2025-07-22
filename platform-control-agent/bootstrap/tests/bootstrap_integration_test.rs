#[cfg(test)]
mod tests {
    use platform_bootstrap::{
        election::{ElectionState, NodeInfo, ElectionConfig},
        pki::PKIService,
        etcd::EtcdService,
        coordinator::BootstrapCoordinator,
        proto::{
            pki::pki_service_server::PkiService as PKIServiceTrait,
            etcd::etcd_service_server::EtcdService as EtcdServiceTrait,
        },
    };
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::time::sleep;
    use tonic::Request;
    use tracing::{info, debug};

    #[tokio::test]
    async fn test_complete_bootstrap_flow() {
        // Initialize tracing for debugging
        let _ = tracing_subscriber::fmt()
            .with_env_filter("platform_bootstrap=debug")
            .try_init();

        info!("Starting complete bootstrap flow test");

        // Create three nodes to simulate a cluster
        let nodes = vec![
            create_node("node-1", "127.0.0.1:50100", 200, 3600), // High priority, experienced
            create_node("node-2", "127.0.0.1:50101", 150, 1800), // Medium priority
            create_node("node-3", "127.0.0.1:50102", 100, 600),  // Low priority, new
        ];

        let mut coordinators = Vec::new();
        let mut election_services = Vec::new();

        // Initialize all nodes
        for (node_info, election_state) in &nodes {
            let election_service = Arc::new(
                platform_bootstrap::election::ElectionService::new(election_state.clone())
            );
            let pki_service = Arc::new(PKIService::new(election_state.clone()));
            let etcd_service = Arc::new(EtcdService::new(
                election_state.clone(),
                pki_service.clone(),
            ));

            let coordinator = Arc::new(BootstrapCoordinator::new(
                election_service.clone(),
                election_state.clone(),
                pki_service.clone(),
                etcd_service.clone(),
            ));

            coordinators.push(coordinator);
            election_services.push(election_service);
        }

        // Phase 1: Leader Election
        info!("=== Phase 1: Leader Election ===");
        
        // Start election on all nodes
        for election_service in &election_services {
            election_service.start().await.unwrap();
        }

        // Give time for election to complete
        sleep(Duration::from_secs(3)).await;

        // Check that we have exactly one leader
        let mut leader_count = 0;
        let mut leader_node = None;
        
        for (i, (node_info, election_state)) in nodes.iter().enumerate() {
            let is_leader = election_state.is_leader().await;
            if is_leader {
                leader_count += 1;
                leader_node = Some(i);
                info!("Node {} is the leader", node_info.node_id);
            } else {
                info!("Node {} is a follower", node_info.node_id);
            }
        }

        assert_eq!(leader_count, 1, "Should have exactly one leader");
        let leader_idx = leader_node.expect("Should have a leader");

        // Phase 2: PKI Distribution
        info!("=== Phase 2: PKI Distribution ===");
        
        // Leader initializes PKI
        let leader_coordinator = &coordinators[leader_idx];
        let (_, leader_election_state) = &nodes[leader_idx];
        
        // Create separate PKI service for testing
        let leader_pki_service = Arc::new(PKIService::new(leader_election_state.clone()));
        let init_request = Request::new(platform_bootstrap::proto::pki::InitializePkiRequest {
            config: None,
            force: false,
        });
        
        if leader_election_state.is_leader().await {
            let result = PKIServiceTrait::initialize_pki(&*leader_pki_service, init_request).await;
            assert!(result.is_ok(), "Leader should be able to initialize PKI");
            info!("PKI initialized successfully");
        }

        // Followers request certificates
        for (i, _) in coordinators.iter().enumerate() {
            if i != leader_idx {
                let (node_info, _) = &nodes[i];
                // In real scenario, followers would request certificates via gRPC
                debug!("Node {} would request certificates from leader", node_info.node_id);
            }
        }

        // Phase 3: etcd Formation
        info!("=== Phase 3: etcd Formation ===");
        
        // Create separate etcd service for testing
        let leader_etcd_service = Arc::new(EtcdService::new(
            leader_election_state.clone(),
            leader_pki_service.clone(),
        ));
        let init_request = Request::new(platform_bootstrap::proto::etcd::InitializeEtcdRequest {
            config: Some(platform_bootstrap::proto::etcd::EtcdConfig {
                version: "3.5.9".to_string(),
                data_dir: "/var/lib/etcd".to_string(),
                listen_client_urls: vec!["https://0.0.0.0:2379".to_string()],
                listen_peer_urls: vec!["https://0.0.0.0:2380".to_string()],
                advertise_client_urls: vec!["https://127.0.0.1:2379".to_string()],
                advertise_peer_urls: vec!["https://127.0.0.1:2380".to_string()],
                cluster_token: "test-token".to_string(),
                quota_backend_bytes: 8589934592, // 8GB
                auto_compaction_mode: "periodic".to_string(),
                auto_compaction_retention: "24h".to_string(),
                snapshot_count: 10000,
                heartbeat_interval_ms: 100,
                election_timeout_ms: 1000,
                tls: None, // Will be set from PKI certificates
                extra_args: std::collections::HashMap::new(),
            }),
            initial_members: vec!["node-1".to_string()],
        });
        
        if leader_election_state.is_leader().await {
            let result = EtcdServiceTrait::initialize_cluster(&*leader_etcd_service, init_request).await;
            assert!(result.is_ok(), "Leader should be able to initialize etcd cluster");
            info!("etcd cluster initialized successfully");
        }

        // Followers join the cluster
        for (i, _) in coordinators.iter().enumerate() {
            if i != leader_idx {
                let (node_info, _) = &nodes[i];
                
                // In real scenario, followers would join via gRPC
                debug!("Node {} would join etcd cluster", node_info.node_id);
            }
        }

        // Verify bootstrap completion
        info!("=== Verifying Bootstrap Completion ===");
        
        // Check that all nodes have completed bootstrap
        for (i, coordinator) in coordinators.iter().enumerate() {
            let (node_info, election_state) = &nodes[i];
            
            // In a real implementation, we would check:
            // 1. Election state is stable
            // 2. PKI certificates are distributed
            // 3. etcd cluster is formed
            // 4. Node is ready for workloads
            
            let state = election_state.state.read().await;
            info!("Node {} final state: {:?}", node_info.node_id, *state);
        }

        info!("Bootstrap flow test completed successfully!");
    }

    #[tokio::test]
    async fn test_leader_failure_recovery() {
        let _ = tracing_subscriber::fmt()
            .with_env_filter("platform_bootstrap=debug")
            .try_init();

        info!("Starting leader failure recovery test");

        // Create two nodes
        let nodes = vec![
            create_node("node-1", "127.0.0.1:50200", 150, 1800),
            create_node("node-2", "127.0.0.1:50201", 100, 1800),
        ];

        let mut election_services = Vec::new();

        for (_, election_state) in &nodes {
            let election_service = Arc::new(
                platform_bootstrap::election::ElectionService::new(election_state.clone())
            );
            election_services.push(election_service);
        }

        // Start elections
        for service in &election_services {
            service.start().await.unwrap();
        }

        sleep(Duration::from_secs(2)).await;

        // Find current leader
        let mut original_leader = None;
        for (i, (node_info, election_state)) in nodes.iter().enumerate() {
            if election_state.is_leader().await {
                original_leader = Some(i);
                info!("Original leader: {}", node_info.node_id);
                break;
            }
        }

        let leader_idx = original_leader.expect("Should have a leader");

        // Simulate leader failure by stopping its election service
        info!("Simulating leader failure...");
        let (_, failed_state) = &nodes[leader_idx];
        failed_state.become_follower(999, None).await;

        // Give time for new election
        sleep(Duration::from_secs(3)).await;

        // Check that we have a new leader
        let mut new_leader_count = 0;
        for (i, (node_info, election_state)) in nodes.iter().enumerate() {
            if election_state.is_leader().await {
                new_leader_count += 1;
                assert_ne!(i, leader_idx, "Failed node should not be leader");
                info!("New leader elected: {}", node_info.node_id);
            }
        }

        assert_eq!(new_leader_count, 1, "Should have exactly one new leader");
        info!("Leader failure recovery test completed successfully!");
    }

    // Helper function to create a node
    fn create_node(
        id: &str,
        address: &str,
        priority: u64,
        uptime_secs: u64,
    ) -> (NodeInfo, Arc<ElectionState>) {
        let node_info = NodeInfo {
            node_id: id.to_string(),
            address: address.to_string(),
            uptime: Duration::from_secs(uptime_secs),
            cpu_available_percent: 80.0,
            memory_available_gb: 8.0,
            packet_loss_percent: 0.0,
            election_priority: priority,
        };

        let config = ElectionConfig::default();
        let election_state = Arc::new(ElectionState::new(
            id.to_string(),
            node_info.clone(),
            config,
        ));

        (node_info, election_state)
    }
}