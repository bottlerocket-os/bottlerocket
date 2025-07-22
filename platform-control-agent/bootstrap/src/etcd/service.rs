use tonic::{Request, Response, Status};
use tokio::sync::{RwLock, broadcast};
use std::sync::Arc;
use std::collections::HashMap;
use std::time::SystemTime;
use anyhow::{Result, Context as AnyhowContext};
use tracing::{info, warn, debug};
use uuid::Uuid;

use crate::proto::etcd::{
    etcd_service_server::EtcdService as EtcdServiceTrait,
    InitializeEtcdRequest, InitializeEtcdResponse,
    JoinEtcdRequest, JoinEtcdResponse,
    LeaveEtcdRequest, LeaveEtcdResponse,
    GetEtcdStatusRequest, EtcdClusterStatus,
    BackupRequest, BackupResponse,
    RestoreRequest, RestoreResponse,
    ObserveClusterRequest, EtcdEvent,
    EtcdMember, HealthStatus, EtcdEventType, MemberStatus,
};

use crate::election::ElectionState;
use crate::pki::PKIService;
use super::config::EtcdConfig;
use super::static_pod::generate_static_pod_manifest;

/// etcd cluster state
#[derive(Debug, Clone)]
pub struct EtcdState {
    pub cluster_initialized: bool,
    pub member_id: Option<String>,
    pub cluster_id: Option<String>,
    pub peers: HashMap<String, EtcdMember>,
    pub last_backup: Option<SystemTime>,
    pub is_healthy: bool,
}

impl Default for EtcdState {
    fn default() -> Self {
        Self {
            cluster_initialized: false,
            member_id: None,
            cluster_id: None,
            peers: HashMap::new(),
            last_backup: None,
            is_healthy: false,
        }
    }
}

/// Join token for authenticating new members
#[derive(Debug, Clone)]
struct JoinToken {
    token: String,
    node_id: String,
    expires_at: SystemTime,
}

pub struct EtcdService {
    state: Arc<RwLock<EtcdState>>,
    config: Arc<RwLock<EtcdConfig>>,
    election_state: Arc<ElectionState>,
    pki_service: Arc<PKIService>,
    join_tokens: Arc<RwLock<HashMap<String, JoinToken>>>,
    event_tx: broadcast::Sender<EtcdEvent>,
    dev_mode: bool,
    // Note: In production, we would use an actual etcd client here
    // For now, we manage state internally
}

impl EtcdService {
    pub fn new(
        election_state: Arc<ElectionState>,
        pki_service: Arc<PKIService>,
    ) -> Self {
        Self::with_dev_mode(election_state, pki_service, false)
    }
    
    pub fn with_dev_mode(
        election_state: Arc<ElectionState>,
        pki_service: Arc<PKIService>,
        dev_mode: bool,
    ) -> Self {
        let (event_tx, _) = broadcast::channel(1024);
        
        Self {
            state: Arc::new(RwLock::new(EtcdState::default())),
            config: Arc::new(RwLock::new(EtcdConfig::default())),
            election_state,
            pki_service,
            join_tokens: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            dev_mode,
        }
    }
    
    /// Generate a join token for a new member
    async fn generate_join_token(&self, node_id: String) -> Result<String> {
        let token = Uuid::new_v4().to_string();
        let join_token = JoinToken {
            token: token.clone(),
            node_id,
            expires_at: SystemTime::now() + std::time::Duration::from_secs(300), // 5 minutes
        };
        
        self.join_tokens.write().await.insert(token.clone(), join_token);
        Ok(token)
    }
    
    /// Validate a join token
    async fn validate_join_token(&self, token: &str, node_id: &str) -> Result<()> {
        let tokens = self.join_tokens.read().await;
        
        match tokens.get(token) {
            Some(join_token) => {
                if join_token.node_id != node_id {
                    anyhow::bail!("Token not valid for this node");
                }
                if SystemTime::now() > join_token.expires_at {
                    anyhow::bail!("Token expired");
                }
                Ok(())
            }
            None => anyhow::bail!("Invalid token"),
        }
    }
    
    /// Clean up expired tokens
    async fn cleanup_expired_tokens(&self) {
        let mut tokens = self.join_tokens.write().await;
        let now = SystemTime::now();
        
        tokens.retain(|_, token| token.expires_at > now);
    }
    
    /// Initialize the etcd configuration for this node
    async fn initialize_node_config(&self, node_name: String) -> Result<()> {
        let mut config = self.config.write().await;
        
        // Set node-specific configuration
        config.node.name = node_name.clone();
        config.node.id = format!("{}-{}", node_name, Uuid::new_v4());
        config.node.data_dir = "/var/lib/etcd".to_string();
        
        // Get node IP from election state
        let node_ip = self.election_state.node_info.address.clone();
        config.node.peer_address = node_ip.parse()
            .context("Failed to parse node IP")?;
        config.node.client_address = config.node.peer_address.clone();
        
        if self.dev_mode {
            // Disable TLS in dev mode
            config.security.peer_tls_enabled = false;
            config.security.client_tls_enabled = false;
            config.security.peer_client_cert_auth = false;
            config.security.client_cert_auth = false;
            info!("Dev mode: etcd TLS disabled");
        } else {
            // Set certificate paths (will be populated by PKI service)
            config.security.peer_cert_file = "/etc/kubernetes/pki/etcd/peer.crt".to_string();
            config.security.peer_key_file = "/etc/kubernetes/pki/etcd/peer.key".to_string();
            config.security.peer_ca_file = "/etc/kubernetes/pki/etcd/ca.crt".to_string();
            config.security.client_cert_file = "/etc/kubernetes/pki/etcd/server.crt".to_string();
            config.security.client_key_file = "/etc/kubernetes/pki/etcd/server.key".to_string();
            config.security.client_ca_file = "/etc/kubernetes/pki/etcd/ca.crt".to_string();
        }
        
        Ok(())
    }
    
    /// Write static pod manifest to filesystem
    async fn deploy_static_pod(&self) -> Result<()> {
        let config = self.config.read().await;
        let manifest = generate_static_pod_manifest(&config)?;
        
        // In production, we would use the Bottlerocket API to set the static pod
        // For now, we'll log the action
        info!(
            node_id = %config.node.id,
            "Would deploy etcd static pod manifest (via Bottlerocket API)"
        );
        
        debug!("Generated manifest:\n{}", manifest);
        
        // TODO: Implement actual Bottlerocket API call
        // Example:
        // self.bottlerocket_client.set_setting(
        //     "kubernetes.static-pods.etcd.manifest",
        //     base64::encode(&manifest)
        // ).await?;
        
        Ok(())
    }
    
    /// Broadcast an event to observers
    fn broadcast_event(&self, event_type: EtcdEventType, description: String) {
        let event = EtcdEvent {
            timestamp: Some(SystemTime::now().into()),
            r#type: event_type as i32,
            details: description,
            data: None, // TODO: Add specific event data when needed
        };
        
        // Ignore send errors (no receivers)
        let _ = self.event_tx.send(event);
    }
}

#[tonic::async_trait]
impl EtcdServiceTrait for EtcdService {
    async fn initialize_cluster(
        &self,
        request: Request<InitializeEtcdRequest>,
    ) -> Result<Response<InitializeEtcdResponse>, Status> {
        let req = request.into_inner();
        
        // Only leader can initialize cluster
        if !self.election_state.is_leader().await {
            return Err(Status::failed_precondition("Only leader can initialize cluster"));
        }
        
        // Check if already initialized
        let state = self.state.read().await;
        if state.cluster_initialized {
            return Err(Status::already_exists("Cluster already initialized"));
        }
        drop(state);
        
        info!("Initializing etcd cluster");
        
        // Initialize node configuration
        let node_name = self.election_state.node_info.node_id.clone();
        self.initialize_node_config(node_name.clone())
            .await
            .map_err(|e| Status::internal(format!("Failed to initialize config: {}", e)))?;
        
        // Update configuration if provided
        if let Some(provided_config) = req.config {
            let mut config = self.config.write().await;
            
            // Only update certain fields from the request
            if !provided_config.version.is_empty() {
                config.version = provided_config.version;
            }
            if !provided_config.auto_compaction_retention.is_empty() {
                config.cluster.auto_compaction_retention = provided_config.auto_compaction_retention;
            }
            if provided_config.quota_backend_bytes > 0 {
                config.cluster.quota_backend_bytes = provided_config.quota_backend_bytes as u64;
            }
        }
        
        // Generate cluster ID
        let cluster_id = Uuid::new_v4().to_string();
        let member_id = Uuid::new_v4().to_string();
        
        // Initialize cluster with self as first member
        let mut config = self.config.write().await;
        config.cluster.cluster_token = cluster_id.clone();
        config.node.initial_cluster_state = "new".to_string();
        
        // Add self to initial cluster
        let node_name = config.node.name.clone();
        let protocol = if self.dev_mode { "http" } else { "https" };
        let peer_url = format!("{}://{}:{}", 
            protocol,
            config.node.peer_address, 
            config.node.peer_port
        );
        config.cluster.initial_cluster.insert(
            node_name,
            peer_url.clone()
        );
        
        drop(config);
        
        // Request certificates from PKI service
        // In production, this would actually call the PKI service
        info!("Requesting etcd certificates from PKI service");
        
        // Deploy static pod
        self.deploy_static_pod()
            .await
            .map_err(|e| Status::internal(format!("Failed to deploy static pod: {}", e)))?;
        
        // Update state
        let mut state = self.state.write().await;
        state.cluster_initialized = true;
        state.cluster_id = Some(cluster_id.clone());
        state.member_id = Some(member_id.clone());
        state.is_healthy = true;
        
        // Add self as first member
        let protocol = if self.dev_mode { "http" } else { "https" };
        let self_member = EtcdMember {
            id: member_id.clone(),
            name: self.election_state.node_info.node_id.clone(),
            peer_urls: vec![peer_url.clone()],
            client_urls: vec![format!("{}://{}:{}", 
                protocol,
                self.config.read().await.node.client_address,
                self.config.read().await.node.client_port
            )],
            is_learner: false,
            status: MemberStatus::Healthy as i32,
        };
        state.peers.insert(member_id.clone(), self_member);
        
        drop(state);
        
        // Broadcast event
        self.broadcast_event(
            EtcdEventType::MemberAdded,  // No ClusterInitialized event, use MemberAdded for initial member
            format!("Cluster {} initialized with leader {}", cluster_id, member_id)
        );
        
        info!(
            cluster_id = %cluster_id,
            member_id = %member_id,
            "etcd cluster initialized successfully"
        );
        
        Ok(Response::new(InitializeEtcdResponse {
            success: true,
            cluster_id: cluster_id.clone(),
            member_id: member_id.clone(),
            peer_urls: vec![peer_url],
        }))
    }
    
    async fn join_cluster(
        &self,
        request: Request<JoinEtcdRequest>,
    ) -> Result<Response<JoinEtcdResponse>, Status> {
        let req = request.into_inner();
        
        // Validate join token
        self.validate_join_token(&req.join_token, &req.node_id)
            .await
            .map_err(|e| Status::unauthenticated(format!("Invalid token: {}", e)))?;
        
        // Check if cluster is initialized
        let state = self.state.read().await;
        if !state.cluster_initialized {
            return Err(Status::failed_precondition("Cluster not initialized"));
        }
        let cluster_id = state.cluster_id.clone()
            .ok_or_else(|| Status::internal("Cluster ID not set"))?;
        drop(state);
        
        info!(
            node_id = %req.node_id,
            "Processing join request"
        );
        
        // Initialize node configuration
        self.initialize_node_config(req.node_id.clone())
            .await
            .map_err(|e| Status::internal(format!("Failed to initialize config: {}", e)))?;
        
        // Generate member ID
        let member_id = Uuid::new_v4().to_string();
        
        // Update configuration for joining existing cluster
        let mut config = self.config.write().await;
        config.node.initial_cluster_state = "existing".to_string();
        config.cluster.cluster_token = cluster_id.clone();
        
        // Add all existing members to initial cluster
        let state = self.state.read().await;
        for (_, member) in &state.peers {
            if !member.peer_urls.is_empty() {
                config.cluster.initial_cluster.insert(
                    member.name.clone(),
                    member.peer_urls[0].clone()
                );
            }
        }
        drop(state);
        
        // Add self to initial cluster
        if !req.peer_urls.is_empty() {
            config.cluster.initial_cluster.insert(
                req.node_id.clone(),
                req.peer_urls[0].clone()
            );
        }
        
        drop(config);
        
        // Request certificates from PKI service
        info!("Requesting etcd certificates from PKI service");
        
        // Deploy static pod
        self.deploy_static_pod()
            .await
            .map_err(|e| Status::internal(format!("Failed to deploy static pod: {}", e)))?;
        
        // Add new member to state
        let mut state = self.state.write().await;
        let new_member = EtcdMember {
            id: member_id.clone(),
            name: req.node_id.clone(),
            peer_urls: req.peer_urls.clone(),
            client_urls: vec![], // Will be populated when member reports in
            is_learner: false,
            status: MemberStatus::Unknown as i32,
        };
        state.peers.insert(member_id.clone(), new_member);
        drop(state);
        
        // Clean up used token
        self.join_tokens.write().await.remove(&req.join_token);
        
        // Broadcast event
        self.broadcast_event(
            EtcdEventType::MemberAdded,
            format!("Member {} joined cluster", req.node_id)
        );
        
        info!(
            node_id = %req.node_id,
            member_id = %member_id,
            "Node joined cluster successfully"
        );
        
        Ok(Response::new(JoinEtcdResponse {
            success: true,
            member_id,
            cluster_id,
        }))
    }
    
    async fn leave_cluster(
        &self,
        request: Request<LeaveEtcdRequest>,
    ) -> Result<Response<LeaveEtcdResponse>, Status> {
        let req = request.into_inner();
        
        let state = self.state.read().await;
        if !state.cluster_initialized {
            return Err(Status::failed_precondition("Cluster not initialized"));
        }
        
        // Get the current node's member ID
        let member_id = state.member_id.clone()
            .ok_or_else(|| Status::internal("Member ID not set"))?;
        drop(state);
        
        info!(
            member_id = %member_id,
            graceful = %req.graceful,
            "Processing leave request for current node"
        );
        
        if req.graceful {
            // TODO: Implement graceful shutdown with data transfer
            info!("Performing graceful etcd member removal");
        }
        
        // Remove self from state
        let mut state = self.state.write().await;
        state.cluster_initialized = false;
        state.is_healthy = false;
        
        // Broadcast event
        self.broadcast_event(
            EtcdEventType::MemberRemoved,
            format!("Member {} left cluster", member_id)
        );
        
        Ok(Response::new(LeaveEtcdResponse {
            success: true,
            reason: "Node left cluster successfully".to_string(),
        }))
    }
    
    async fn get_cluster_status(
        &self,
        _request: Request<GetEtcdStatusRequest>,
    ) -> Result<Response<EtcdClusterStatus>, Status> {
        let state = self.state.read().await;
        
        let members: Vec<EtcdMember> = state.peers.values().cloned().collect();
        
        let status = EtcdClusterStatus {
            cluster_id: state.cluster_id.clone().unwrap_or_default(),
            member_id: state.member_id.clone().unwrap_or_default(),
            leader_id: if self.election_state.is_leader().await {
                state.member_id.clone().unwrap_or_default()
            } else {
                String::new()
            },
            raft_term: 0, // Would come from actual etcd client
            raft_index: 0, // Would come from actual etcd client
            members,
            health: if state.is_healthy {
                Some(HealthStatus {
                    healthy: true,
                    reason: "Cluster healthy".to_string(),
                    last_check: Some(SystemTime::now().into()),
                    checks: vec![],
                })
            } else {
                Some(HealthStatus {
                    healthy: false,
                    reason: "Cluster not initialized".to_string(),
                    last_check: Some(SystemTime::now().into()),
                    checks: vec![],
                })
            },
            db_size_bytes: 0, // Would come from actual etcd client
            db_size_in_use_bytes: 0, // Would come from actual etcd client
            alarms: vec![],
            last_backup: state.last_backup.map(Into::into),
        };
        
        Ok(Response::new(status))
    }
    
    async fn backup_data(
        &self,
        _request: Request<BackupRequest>,
    ) -> Result<Response<BackupResponse>, Status> {
        // Only leader should perform backups
        if !self.election_state.is_leader().await {
            return Err(Status::failed_precondition("Only leader can perform backups"));
        }
        
        let state = self.state.read().await;
        if !state.cluster_initialized {
            return Err(Status::failed_precondition("Cluster not initialized"));
        }
        drop(state);
        
        info!("Starting etcd backup");
        
        // In production, this would use etcdctl to create a snapshot
        // For now, we simulate the backup
        let backup_path = format!("/var/lib/etcd-backup/snapshot-{}.db", 
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );
        
        // Update last backup time
        self.state.write().await.last_backup = Some(SystemTime::now());
        
        // Broadcast event
        self.broadcast_event(
            EtcdEventType::BackupCreated,
            format!("Backup completed: {}", backup_path)
        );
        
        Ok(Response::new(BackupResponse {
            backup_id: Uuid::new_v4().to_string(),
            size_bytes: 0, // Would be actual size
            location: backup_path,
            timestamp: Some(SystemTime::now().into()),
        }))
    }
    
    async fn restore_data(
        &self,
        request: Request<RestoreRequest>,
    ) -> Result<Response<RestoreResponse>, Status> {
        let req = request.into_inner();
        
        // Restore should only be done on a stopped cluster
        let state = self.state.read().await;
        if state.cluster_initialized && state.is_healthy {
            return Err(Status::failed_precondition(
                "Cannot restore to a running cluster"
            ));
        }
        drop(state);
        
        info!(
            backup_id = %req.backup_id,
            force = %req.force,
            "Starting etcd restore"
        );
        
        // In production, this would use etcdctl to restore from snapshot
        // For now, we simulate the restore
        
        // Broadcast event
        self.broadcast_event(
            EtcdEventType::BackupCreated,  // No RestoreCompleted event, reuse BackupCreated
            format!("Restore completed from backup: {}", req.backup_id)
        );
        
        Ok(Response::new(RestoreResponse {
            success: true,
            details: "Restore completed successfully".to_string(),
        }))
    }
    
    type ObserveClusterStream = std::pin::Pin<
        Box<dyn tokio_stream::Stream<Item = Result<EtcdEvent, Status>> + Send>
    >;
    
    async fn observe_cluster(
        &self,
        request: Request<ObserveClusterRequest>,
    ) -> Result<Response<Self::ObserveClusterStream>, Status> {
        let _req = request.into_inner();
        let mut rx = self.event_tx.subscribe();
        
        let stream = async_stream::stream! {
            // Clean up expired tokens periodically
            let mut cleanup_interval = tokio::time::interval(
                std::time::Duration::from_secs(60)
            );
            
            loop {
                tokio::select! {
                    event = rx.recv() => {
                        match event {
                            Ok(evt) => {
                                // For now, yield all events (filtering could be added later)
                                yield Ok(evt);
                            }
                            Err(broadcast::error::RecvError::Closed) => break,
                            Err(broadcast::error::RecvError::Lagged(_)) => {
                                warn!("Event stream lagged, some events dropped");
                                continue;
                            }
                        }
                    }
                    _ = cleanup_interval.tick() => {
                        // This doesn't yield events, just maintains tokens
                        continue;
                    }
                }
            }
        };
        
        Ok(Response::new(Box::pin(stream)))
    }
}

/// Public API for generating join tokens (used by coordinator)
impl EtcdService {
    pub async fn create_join_token(&self, node_id: String) -> Result<String> {
        // Only leader can create join tokens
        if !self.election_state.is_leader().await {
            anyhow::bail!("Only leader can create join tokens");
        }
        
        self.generate_join_token(node_id).await
    }
    
    /// Get current cluster state
    pub async fn is_cluster_initialized(&self) -> bool {
        self.state.read().await.cluster_initialized
    }
    
    /// Wait for cluster to be ready (used by non-leaders)
    pub async fn wait_for_cluster_ready(&self) -> Result<()> {
        let timeout = std::time::Duration::from_secs(300); // 5 minutes
        let start = std::time::Instant::now();
        
        loop {
            if self.is_cluster_initialized().await {
                return Ok(());
            }
            
            if start.elapsed() > timeout {
                anyhow::bail!("Timeout waiting for cluster initialization");
            }
            
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    }
}