use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use anyhow::{Result, anyhow};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use tonic::transport::Channel;
use prost_types::Timestamp;

use crate::proto::election::{
    election_service_client::ElectionServiceClient,
    VoteRequest, VoteResponse, HeartbeatRequest, HeartbeatResponse,
};
use super::state::{ElectionState, NodeState, PriorityScore};

/// Convert SystemTime to prost Timestamp
fn system_time_to_timestamp(time: SystemTime) -> Timestamp {
    let duration = time.duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    Timestamp {
        seconds: duration.as_secs() as i64,
        nanos: duration.subsec_nanos() as i32,
    }
}

/// Raft consensus implementation for leader election
pub struct RaftElection {
    state: Arc<ElectionState>,
    clients: Arc<RwLock<HashMap<String, ElectionServiceClient<Channel>>>>,
}

impl RaftElection {
    pub fn new(state: Arc<ElectionState>) -> Self {
        Self {
            state,
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Initialize connections to cluster members
    pub async fn initialize_connections(&self) -> Result<()> {
        let members = self.state.cluster_members.read().await;
        let mut clients = self.clients.write().await;
        
        for (node_id, node_info) in members.iter() {
            if node_id != &self.state.node_id {
                match self.create_client(&node_info.address).await {
                    Ok(client) => {
                        clients.insert(node_id.clone(), client);
                        debug!("Connected to cluster member: {}", node_id);
                    }
                    Err(e) => {
                        let msg = format!("Failed to connect to cluster member {}: {}", node_id, e);
                        warn!("{}", msg);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Create gRPC client for a node
    async fn create_client(&self, address: &str) -> Result<ElectionServiceClient<Channel>> {
        // Try to load TLS certificates - if they exist, use HTTPS; otherwise, use HTTP
        if let (Ok(ca_cert), Ok(client_cert), Ok(client_key)) = (
            std::fs::read("/etc/platform/certs/ca.crt"),
            std::fs::read("/etc/platform/certs/tls.crt"),
            std::fs::read("/etc/platform/certs/tls.key")
        ) {
            // Use HTTPS with mTLS
            let endpoint = format!("https://{}", address);
            
            let ca_cert = tonic::transport::Certificate::from_pem(ca_cert);
            let client_identity = tonic::transport::Identity::from_pem(client_cert, client_key);
            
            let tls_config = tonic::transport::ClientTlsConfig::new()
                .ca_certificate(ca_cert)
                .identity(client_identity);
                
            let channel = Channel::from_shared(endpoint)?
                .tls_config(tls_config)?
                .connect()
                .await?;
                
            Ok(ElectionServiceClient::new(channel))
        } else {
            // Use HTTP (plaintext) 
            let endpoint = format!("http://{}", address);
            let channel = Channel::from_shared(endpoint)?
                .connect()
                .await?;
            Ok(ElectionServiceClient::new(channel))
        }
    }
    
    /// Start an election campaign
    pub async fn start_election(&self) -> Result<bool> {
        // Check if we're already a leader or candidate
        let current_state = *self.state.state.read().await;
        if current_state != NodeState::Follower {
            return Ok(false);
        }
        
        let msg = format!("Starting election campaign for node {}", self.state.node_id);
        info!("{}", msg);
        
        // Transition to candidate
        self.state.become_candidate().await;
        
        // Perform pre-vote if enabled
        if self.state.config.pre_vote_enabled {
            if !self.pre_vote().await? {
                info!("Pre-vote failed, returning to follower");
                let term = *self.state.current_term.read().await;
                self.state.become_follower(term, None).await;
                return Ok(false);
            }
        }
        
        // Request votes from all nodes
        let won = self.request_votes().await?;
        
        if won {
            self.state.become_leader().await;
            let node_id = self.state.node_id.clone();
            let term = *self.state.current_term.read().await;
            let msg = format!("Won election for node {} at term {}, became leader", node_id, term);
            info!("{}", msg);
        } else {
            let term = *self.state.current_term.read().await;
            self.state.become_follower(term, None).await;
            info!("Lost election, returning to follower");
        }
        
        Ok(won)
    }
    
    /// Perform pre-vote phase
    async fn pre_vote(&self) -> Result<bool> {
        debug!("Starting pre-vote phase");
        
        let term = *self.state.current_term.read().await;
        let priority = PriorityScore::calculate(&self.state.node_info);
        
        let request = VoteRequest {
            candidate_id: self.state.node_id.clone(),
            term: term as i64,
            last_log_index: 0, // TODO: Implement log tracking
            last_log_term: 0,
            priority: Some(priority.into()),
            pre_vote_proof: vec![], // This is the pre-vote, so no proof yet
        };
        
        // Clear pre-vote tracking
        self.state.pre_votes_received.lock().await.clear();
        self.state.pre_votes_received.lock().await.insert(self.state.node_id.clone(), true);
        
        // Send pre-vote requests
        let clients = self.clients.read().await;
        let mut vote_futures = Vec::new();
        
        for (node_id, client) in clients.iter() {
            let mut client = client.clone();
            let request = request.clone();
            let node_id = node_id.clone();
            
            vote_futures.push(tokio::spawn(async move {
                match client.request_vote(request).await {
                    Ok(response) => Some((node_id, response.into_inner())),
                    Err(e) => {
                        let msg = format!("Pre-vote request failed for {}: {}", node_id, e);
                        warn!("{}", msg);
                        None
                    }
                }
            }));
        }
        
        // Collect pre-votes
        let mut pre_votes = self.state.pre_votes_received.lock().await;
        for future in vote_futures {
            if let Ok(Some((node_id, response))) = future.await {
                if response.vote_granted {
                    pre_votes.insert(node_id, true);
                }
            }
        }
        
        let vote_count = pre_votes.len();
        let has_majority = self.state.has_majority(vote_count).await;
        
        let cluster_size = self.state.cluster_size().await;
        debug!(
            votes = %vote_count,
            cluster_size = %cluster_size,
            has_majority = %has_majority,
            "Pre-vote phase complete"
        );
        
        Ok(has_majority)
    }
    
    /// Request votes from all nodes
    async fn request_votes(&self) -> Result<bool> {
        debug!("Requesting votes from cluster");
        
        let term = *self.state.current_term.read().await;
        let priority = PriorityScore::calculate(&self.state.node_info);
        
        // Collect pre-vote proof
        let pre_vote_proof = self.generate_pre_vote_proof().await?;
        
        let request = VoteRequest {
            candidate_id: self.state.node_id.clone(),
            term: term as i64,
            last_log_index: 0, // TODO: Implement log tracking
            last_log_term: 0,
            priority: Some(priority.into()),
            pre_vote_proof,
        };
        
        // Send vote requests
        let clients = self.clients.read().await;
        let mut vote_futures = Vec::new();
        
        for (node_id, client) in clients.iter() {
            let mut client = client.clone();
            let request = request.clone();
            let node_id = node_id.clone();
            
            vote_futures.push(tokio::spawn(async move {
                match client.request_vote(request).await {
                    Ok(response) => Some((node_id, response.into_inner())),
                    Err(e) => {
                        let msg = format!("Vote request failed for {}: {}", node_id, e);
                        warn!("{}", msg);
                        None
                    }
                }
            }));
        }
        
        // Collect votes
        let mut votes = self.state.votes_received.lock().await;
        for future in vote_futures {
            if let Ok(Some((node_id, response))) = future.await {
                if response.vote_granted {
                    votes.insert(node_id.clone(), true);
                    debug!("Received vote from: {}", node_id);
                } else {
                    debug!("Vote denied from {}: {}", node_id, response.reason);
                }
                
                // Check if we discovered a higher term
                if response.term > term as i64 {
                    warn!("Discovered higher term {}, stepping down", response.term);
                    self.state.become_follower(response.term as u64, None).await;
                    return Ok(false);
                }
            }
        }
        
        let vote_count = votes.len();
        let has_majority = self.state.has_majority(vote_count).await;
        
        let cluster_size = self.state.cluster_size().await;
        info!(
            votes = %vote_count,
            cluster_size = %cluster_size,
            has_majority = %has_majority,
            "Vote collection complete"
        );
        
        Ok(has_majority)
    }
    
    /// Generate pre-vote proof (signatures from pre-vote phase)
    async fn generate_pre_vote_proof(&self) -> Result<Vec<u8>> {
        // TODO: Implement cryptographic proof generation
        // For now, return a simple serialized list of voters
        let pre_votes = self.state.pre_votes_received.lock().await;
        let voters: Vec<String> = pre_votes.keys().cloned().collect();
        Ok(serde_json::to_vec(&voters)?)
    }
    
    /// Handle incoming vote request
    pub async fn handle_vote_request(&self, request: VoteRequest) -> Result<VoteResponse> {
        let current_term = *self.state.current_term.read().await;
        let voted_for = self.state.voted_for.read().await.clone();
        let current_state = *self.state.state.read().await;
        
        let mut vote_granted = false;
        let mut reason = String::new();
        let mut response_term = current_term;
        
        // Check term
        if request.term < current_term as i64 {
            reason = "Outdated term".to_string();
        }
        // Update term if we see a higher one
        else if request.term > current_term as i64 {
            response_term = request.term as u64;
            self.state.become_follower(request.term as u64, None).await;
            // After becoming follower, we can vote
            vote_granted = true;
            *self.state.voted_for.write().await = Some(request.candidate_id.clone());
        }
        // Same term - check if we already voted
        else if let Some(voted_id) = voted_for {
            if voted_id == request.candidate_id {
                vote_granted = true;
                reason = "Already voted for this candidate".to_string();
            } else {
                reason = "Already voted for another candidate".to_string();
            }
        }
        // Haven't voted yet in this term
        else {
            // Verify pre-vote proof if required
            if self.state.config.pre_vote_enabled && request.pre_vote_proof.is_empty() {
                reason = "Missing pre-vote proof".to_string();
            }
            // Compare priorities
            else if current_state == NodeState::Leader {
                reason = "Currently leader".to_string();
            }
            // Check candidate priority
            else if let Some(priority) = request.priority {
                let my_priority = PriorityScore::calculate(&self.state.node_info);
                if priority.total >= my_priority.total() {
                    vote_granted = true;
                    *self.state.voted_for.write().await = Some(request.candidate_id.clone());
                    reason = "Candidate has sufficient priority".to_string();
                } else {
                    reason = "Candidate priority too low".to_string();
                }
            } else {
                reason = "No priority information".to_string();
            }
        }
        
        debug!("Processed vote request from {} for term {}: granted={}, reason={}", 
            request.candidate_id, request.term, vote_granted, reason);
        
        Ok(VoteResponse {
            term: response_term as i64,
            vote_granted,
            reason,
            signature: vec![], // TODO: Implement signature
        })
    }
    
    /// Send heartbeats as leader
    pub async fn send_heartbeats(&self) -> Result<()> {
        let current_state = *self.state.state.read().await;
        if current_state != NodeState::Leader {
            return Err(anyhow!("Not a leader"));
        }
        
        let term = *self.state.current_term.read().await;
        let lease = self.state.leader_lease.read().await.clone();
        let lease_expiry = lease.map(|l| system_time_to_timestamp(l.expiry));
        
        let request = HeartbeatRequest {
            leader_id: self.state.node_id.clone(),
            term: term as i64,
            commit_index: 0, // TODO: Implement log tracking
            lease_expiry,
        };
        
        let clients = self.clients.read().await;
        let mut heartbeat_futures = Vec::new();
        
        for (node_id, client) in clients.iter() {
            let mut client = client.clone();
            let request = request.clone();
            let node_id = node_id.clone();
            
            heartbeat_futures.push(tokio::spawn(async move {
                match client.heartbeat(request).await {
                    Ok(response) => Some((node_id, response.into_inner())),
                    Err(e) => {
                        let msg = format!("Heartbeat failed for {}: {}", node_id, e);
                        warn!("{}", msg);
                        None
                    }
                }
            }));
        }
        
        // Check responses for higher terms
        for future in heartbeat_futures {
            if let Ok(Some((node_id, response))) = future.await {
                if response.term as u64 > term {
                    warn!("Discovered higher term from {}: {} > {}, stepping down", 
                        node_id, response.term, term);
                    self.state.become_follower(response.term as u64, None).await;
                    return Ok(());
                }
            }
        }
        
        // Renew lease if needed
        if let Some(lease) = self.state.leader_lease.write().await.as_mut() {
            if lease.needs_renewal() {
                lease.renew();
                debug!("Renewed leader lease");
            }
        }
        
        Ok(())
    }
    
    /// Handle incoming heartbeat
    pub async fn handle_heartbeat(&self, request: HeartbeatRequest) -> Result<HeartbeatResponse> {
        let current_term = *self.state.current_term.read().await;
        
        if request.term < current_term as i64 {
            // Reject heartbeat from old term
            debug!("Rejecting heartbeat from {} (term {} < our term {})", 
                request.leader_id, request.term, current_term);
            
            Ok(HeartbeatResponse {
                term: current_term as i64,
                success: false,
            })
        } else {
            // Accept heartbeat and record it
            self.state.record_heartbeat(&request.leader_id, request.term as u64).await;
            
            Ok(HeartbeatResponse {
                term: request.term,
                success: true,
            })
        }
    }
}