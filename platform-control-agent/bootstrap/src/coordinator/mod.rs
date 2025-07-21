// Bootstrap coordinator - orchestrates the entire bootstrap process
use std::sync::Arc;
use std::time::Duration;
use anyhow::{Result, Context};
use tracing::{info, warn, error, debug};
use tokio::time::sleep;

use crate::election::{ElectionService, ElectionState};
use crate::pki::PKIService;
use crate::etcd::EtcdService;
use crate::proto::etcd::{
    InitializeEtcdRequest, JoinEtcdRequest, EtcdConfig as ProtoEtcdConfig,
    etcd_service_server::EtcdService as EtcdServiceTrait,
};
use crate::proto::pki::{
    InitializePkiRequest,
    pki_service_server::PkiService as PkiServiceTrait,
};

pub struct BootstrapCoordinator {
    election_service: Arc<ElectionService>,
    election_state: Arc<ElectionState>,
    pki_service: Arc<PKIService>,
    etcd_service: Arc<EtcdService>,
}

impl BootstrapCoordinator {
    pub fn new(
        election_service: Arc<ElectionService>,
        election_state: Arc<ElectionState>,
        pki_service: Arc<PKIService>,
        etcd_service: Arc<EtcdService>,
    ) -> Self {
        Self {
            election_service,
            election_state,
            pki_service,
            etcd_service,
        }
    }
    
    /// Start the bootstrap process
    pub async fn start_bootstrap(&self) -> Result<()> {
        info!("Starting cluster bootstrap process");
        
        // Phase 1: Leader Election
        info!("Phase 1: Leader Election");
        self.wait_for_election_stability().await?;
        
        // Phase 2: PKI Generation
        info!("Phase 2: PKI Generation");
        self.handle_pki_phase().await?;
        
        // Phase 3: etcd Formation
        info!("Phase 3: etcd Formation");
        self.handle_etcd_phase().await?;
        
        info!("Bootstrap process complete");
        Ok(())
    }
    
    /// Wait for election to stabilize with a leader
    async fn wait_for_election_stability(&self) -> Result<()> {
        let timeout = Duration::from_secs(300); // 5 minutes
        let start = std::time::Instant::now();
        
        loop {
            // Check if we have a stable leader
            let leader = self.election_state.current_leader.read().await;
            if leader.is_some() {
                let is_leader = self.election_state.is_leader().await;
                info!(
                    node_id = %self.election_state.node_id,
                    leader = ?leader,
                    is_leader = %is_leader,
                    "Election complete"
                );
                return Ok(());
            }
            drop(leader);
            
            if start.elapsed() > timeout {
                return Err(anyhow::anyhow!("Timeout waiting for leader election"));
            }
            
            debug!("Waiting for leader election to complete...");
            sleep(Duration::from_secs(5)).await;
        }
    }
    
    /// Handle PKI phase - initialize if leader, wait if follower
    async fn handle_pki_phase(&self) -> Result<()> {
        if self.election_state.is_leader().await {
            info!("This node is the leader, initializing PKI");
            
            // Initialize PKI on the leader
            let request = tonic::Request::new(InitializePkiRequest {
                config: None, // Use default config
                force: false, // Don't force reinitialization
            });
            
            PkiServiceTrait::initialize_pki(&*self.pki_service, request)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to initialize PKI: {}", e))?;
            
            info!("PKI CA initialized successfully");
            
            // Give some time for the PKI service to be ready
            sleep(Duration::from_secs(2)).await;
        } else {
            info!("This node is a follower, waiting for PKI to be ready");
            
            // Wait for PKI to be available from leader
            self.wait_for_pki_ready().await?;
        }
        
        Ok(())
    }
    
    /// Wait for PKI service to be ready
    async fn wait_for_pki_ready(&self) -> Result<()> {
        let timeout = Duration::from_secs(300); // 5 minutes
        let start = std::time::Instant::now();
        
        loop {
            // Check if we can request a certificate
            // In production, this would actually try to get a certificate
            let leader = self.election_state.current_leader.read().await;
            if leader.is_some() {
                // Assume PKI is ready if we have a leader
                // In production, we'd check if the PKI service is actually serving certificates
                info!("PKI service is ready");
                return Ok(());
            }
            
            if start.elapsed() > timeout {
                return Err(anyhow::anyhow!("Timeout waiting for PKI service"));
            }
            
            debug!("Waiting for PKI service to be ready...");
            sleep(Duration::from_secs(5)).await;
        }
    }
    
    /// Handle etcd phase - initialize if leader, join if follower
    async fn handle_etcd_phase(&self) -> Result<()> {
        if self.election_state.is_leader().await {
            info!("This node is the leader, initializing etcd cluster");
            
            // Initialize etcd cluster
            let request = tonic::Request::new(InitializeEtcdRequest {
                config: Some(ProtoEtcdConfig {
                    version: "3.5.10".to_string(),
                    data_dir: "/var/lib/etcd".to_string(),
                    listen_peer_urls: vec![],  // Will be set by service
                    listen_client_urls: vec![], // Will be set by service
                    advertise_peer_urls: vec![], // Will be set by service
                    advertise_client_urls: vec![], // Will be set by service
                    cluster_token: String::new(), // Will be set by service
                    quota_backend_bytes: 8 * 1024 * 1024 * 1024, // 8GB
                    auto_compaction_mode: "periodic".to_string(),
                    auto_compaction_retention: "24h".to_string(),
                    snapshot_count: 100000,
                    heartbeat_interval_ms: 100,
                    election_timeout_ms: 1000,
                    tls: None, // Will be configured by the service
                    extra_args: std::collections::HashMap::new(),
                }),
                initial_members: vec![], // Start with just the leader
            });
            
            match EtcdServiceTrait::initialize_cluster(&*self.etcd_service, request).await {
                Ok(response) => {
                    let resp = response.into_inner();
                    info!(
                        cluster_id = %resp.cluster_id,
                        member_id = %resp.member_id,
                        "etcd cluster initialized successfully"
                    );
                }
                Err(e) => {
                    // Check if already initialized
                    if e.code() == tonic::Code::AlreadyExists {
                        info!("etcd cluster already initialized");
                    } else {
                        return Err(anyhow::anyhow!("Failed to initialize etcd cluster: {}", e));
                    }
                }
            }
        } else {
            info!("This node is a follower, joining etcd cluster");
            
            // Wait for cluster to be initialized
            self.etcd_service.wait_for_cluster_ready()
                .await
                .context("Failed waiting for etcd cluster")?;
            
            // Get join token from leader
            let join_token = self.request_join_token().await?;
            
            // Get our node information
            let node_id = self.election_state.node_id.clone();
            let node_ip = self.election_state.node_info.address.clone();
            let peer_urls = vec![format!("https://{}:2380", node_ip)];
            
            // Join the cluster
            let request = tonic::Request::new(JoinEtcdRequest {
                node_id: node_id.clone(),
                peer_urls,
                join_token,
            });
            
            match EtcdServiceTrait::join_cluster(&*self.etcd_service, request).await {
                Ok(response) => {
                    let resp = response.into_inner();
                    info!(
                        cluster_id = %resp.cluster_id,
                        member_id = %resp.member_id,
                        "Successfully joined etcd cluster"
                    );
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("Failed to join etcd cluster: {}", e));
                }
            }
        }
        
        Ok(())
    }
    
    /// Request a join token from the leader
    async fn request_join_token(&self) -> Result<String> {
        // In production, this would make a gRPC call to the leader
        // For now, we'll simulate getting a token
        let leader = self.election_state.current_leader.read().await
            .clone()
            .ok_or_else(|| anyhow::anyhow!("No leader available"))?;
        
        info!(
            leader = %leader,
            "Requesting join token from leader"
        );
        
        // TODO: Implement actual gRPC call to leader's etcd service
        // For now, return a placeholder
        warn!("Join token request not fully implemented - using placeholder");
        Ok("placeholder-join-token".to_string())
    }
    
    /// Run continuous health checks and maintenance
    pub async fn run_maintenance_loop(&self) -> Result<()> {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            
            // Check if we're still the leader
            if self.election_state.is_leader().await {
                // Perform leader-specific maintenance
                debug!("Running leader maintenance tasks");
                
                // TODO: Add health checks for etcd
                // TODO: Monitor PKI certificate expiration
                // TODO: Check cluster health
            } else {
                // Perform follower-specific maintenance
                debug!("Running follower maintenance tasks");
                
                // TODO: Check connection to leader
                // TODO: Verify certificate validity
            }
        }
    }
}

/// Public API for external coordination
impl BootstrapCoordinator {
    /// Check if bootstrap is complete
    pub async fn is_bootstrap_complete(&self) -> bool {
        // Bootstrap is complete when:
        // 1. We have a stable leader
        // 2. PKI is initialized
        // 3. etcd cluster is formed
        
        let has_leader = self.election_state.current_leader.read().await.is_some();
        let etcd_ready = self.etcd_service.is_cluster_initialized().await;
        
        has_leader && etcd_ready
    }
    
    /// Get bootstrap status
    pub async fn get_bootstrap_status(&self) -> BootstrapStatus {
        BootstrapStatus {
            election_complete: self.election_state.current_leader.read().await.is_some(),
            is_leader: self.election_state.is_leader().await,
            pki_initialized: true, // TODO: Get actual PKI status
            etcd_initialized: self.etcd_service.is_cluster_initialized().await,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BootstrapStatus {
    pub election_complete: bool,
    pub is_leader: bool,
    pub pki_initialized: bool,
    pub etcd_initialized: bool,
}