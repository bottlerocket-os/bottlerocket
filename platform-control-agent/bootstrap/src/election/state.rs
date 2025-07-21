use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{RwLock, Mutex};
use tracing::{debug, info};

use crate::proto::election::{NodeState as ProtoNodeState, PriorityScore as ProtoPriorityScore};

/// Node states in the election process
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeState {
    Follower,
    Candidate,
    Leader,
}

impl From<NodeState> for ProtoNodeState {
    fn from(state: NodeState) -> Self {
        match state {
            NodeState::Follower => ProtoNodeState::Follower,
            NodeState::Candidate => ProtoNodeState::Candidate,
            NodeState::Leader => ProtoNodeState::Leader,
        }
    }
}

impl From<ProtoNodeState> for NodeState {
    fn from(state: ProtoNodeState) -> Self {
        match state {
            ProtoNodeState::Unknown => NodeState::Follower,
            ProtoNodeState::Follower => NodeState::Follower,
            ProtoNodeState::Candidate => NodeState::Candidate,
            ProtoNodeState::Leader => NodeState::Leader,
        }
    }
}

/// Priority score components for leader election
#[derive(Debug, Clone, Default)]
pub struct PriorityScore {
    pub base_score: u64,
    pub stability_bonus: u64,
    pub resource_bonus: u64,
    pub user_priority: u64,
}

impl PriorityScore {
    pub fn total(&self) -> u64 {
        self.base_score + self.stability_bonus + self.resource_bonus + self.user_priority
    }

    pub fn calculate(node_info: &NodeInfo) -> Self {
        let mut score = PriorityScore::default();
        
        // Base score from node ID hash (deterministic tiebreaker)
        let id_bytes = node_info.node_id.as_bytes();
        let mut hash: u64 = 0;
        for byte in id_bytes {
            hash = hash.wrapping_mul(31).wrapping_add(*byte as u64);
        }
        score.base_score = hash % 100;
        
        // Stability bonus (up to 1000 points)
        // Uptime: 1 point per minute, max 600
        let uptime_minutes = node_info.uptime.as_secs() / 60;
        score.stability_bonus += uptime_minutes.min(600);
        
        // Network stability: 0-400 based on packet loss
        let network_score = ((100.0 - node_info.packet_loss_percent) * 4.0) as u64;
        score.stability_bonus += network_score;
        
        // Resource bonus (up to 500 points)
        // CPU availability: 0-250
        let cpu_score = (node_info.cpu_available_percent * 2.5) as u64;
        score.resource_bonus += cpu_score.min(250);
        
        // Memory availability: 0-250
        let memory_score = (node_info.memory_available_gb * 25.0).min(250.0) as u64;
        score.resource_bonus += memory_score;
        
        // User-defined priority (0-1000)
        score.user_priority = node_info.election_priority;
        
        score
    }
}

impl From<PriorityScore> for ProtoPriorityScore {
    fn from(score: PriorityScore) -> Self {
        ProtoPriorityScore {
            base_score: score.base_score,
            stability_bonus: score.stability_bonus,
            resource_bonus: score.resource_bonus,
            user_priority: score.user_priority,
            total: score.total(),
        }
    }
}

impl From<ProtoPriorityScore> for PriorityScore {
    fn from(proto: ProtoPriorityScore) -> Self {
        PriorityScore {
            base_score: proto.base_score,
            stability_bonus: proto.stability_bonus,
            resource_bonus: proto.resource_bonus,
            user_priority: proto.user_priority,
        }
    }
}

/// Information about a node in the cluster
#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub node_id: String,
    pub address: String,
    pub uptime: Duration,
    pub cpu_available_percent: f64,
    pub memory_available_gb: f64,
    pub packet_loss_percent: f64,
    pub election_priority: u64,
}

/// Leader lease mechanism
#[derive(Debug, Clone)]
pub struct LeaderLease {
    pub holder: String,
    pub term: u64,
    pub expiry: SystemTime,
}

impl LeaderLease {
    pub const LEASE_DURATION: Duration = Duration::from_secs(10);
    pub const RENEWAL_INTERVAL: Duration = Duration::from_secs(3);
    
    pub fn new(leader_id: String, term: u64) -> Self {
        Self {
            holder: leader_id,
            term,
            expiry: SystemTime::now() + Self::LEASE_DURATION,
        }
    }
    
    pub fn is_valid(&self) -> bool {
        SystemTime::now() < self.expiry
    }
    
    pub fn needs_renewal(&self) -> bool {
        self.expiry.duration_since(SystemTime::now())
            .map(|d| d < Self::RENEWAL_INTERVAL)
            .unwrap_or(true)
    }
    
    pub fn renew(&mut self) {
        self.expiry = SystemTime::now() + Self::LEASE_DURATION;
    }
}

/// Election timing configuration
#[derive(Debug, Clone)]
pub struct ElectionConfig {
    pub election_timeout_min: Duration,
    pub election_timeout_max: Duration,
    pub heartbeat_interval: Duration,
    pub pre_vote_enabled: bool,
}

impl Default for ElectionConfig {
    fn default() -> Self {
        Self {
            election_timeout_min: Duration::from_secs(5),
            election_timeout_max: Duration::from_secs(10),
            heartbeat_interval: Duration::from_secs(1),
            pre_vote_enabled: true,
        }
    }
}

/// Core election state machine
pub struct ElectionState {
    // Identity
    pub node_id: String,
    pub node_info: NodeInfo,
    
    // Raft state
    pub current_term: Arc<RwLock<u64>>,
    pub voted_for: Arc<RwLock<Option<String>>>,
    pub state: Arc<RwLock<NodeState>>,
    
    // Leader tracking
    pub current_leader: Arc<RwLock<Option<String>>>,
    pub leader_lease: Arc<RwLock<Option<LeaderLease>>>,
    
    // Cluster membership
    pub cluster_members: Arc<RwLock<HashMap<String, NodeInfo>>>,
    
    // Vote tracking
    pub votes_received: Arc<Mutex<HashMap<String, bool>>>,
    pub pre_votes_received: Arc<Mutex<HashMap<String, bool>>>,
    
    // Timing
    pub last_heartbeat: Arc<RwLock<SystemTime>>,
    pub election_deadline: Arc<RwLock<SystemTime>>,
    pub config: ElectionConfig,
    
    // Callbacks
    pub on_state_change: Arc<RwLock<Option<Box<dyn Fn(NodeState, NodeState) + Send + Sync>>>>,
    pub on_leader_change: Arc<RwLock<Option<Box<dyn Fn(Option<String>, Option<String>) + Send + Sync>>>>,
}

impl ElectionState {
    pub fn new(node_id: String, node_info: NodeInfo, config: ElectionConfig) -> Self {
        let now = SystemTime::now();
        let election_deadline = now + config.election_timeout_max;
        
        Self {
            node_id,
            node_info,
            current_term: Arc::new(RwLock::new(0)),
            voted_for: Arc::new(RwLock::new(None)),
            state: Arc::new(RwLock::new(NodeState::Follower)),
            current_leader: Arc::new(RwLock::new(None)),
            leader_lease: Arc::new(RwLock::new(None)),
            cluster_members: Arc::new(RwLock::new(HashMap::new())),
            votes_received: Arc::new(Mutex::new(HashMap::new())),
            pre_votes_received: Arc::new(Mutex::new(HashMap::new())),
            last_heartbeat: Arc::new(RwLock::new(now)),
            election_deadline: Arc::new(RwLock::new(election_deadline)),
            config,
            on_state_change: Arc::new(RwLock::new(None)),
            on_leader_change: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Check if election timeout has occurred
    pub async fn is_election_timeout(&self) -> bool {
        let deadline = *self.election_deadline.read().await;
        SystemTime::now() > deadline
    }
    
    /// Reset election timer with random timeout
    pub async fn reset_election_timer(&self) {
        use rand::Rng;
        
        // Generate random timeout before the await
        let timeout_range = self.config.election_timeout_max - self.config.election_timeout_min;
        let random_millis = {
            let mut rng = rand::thread_rng();
            rng.gen_range(0..timeout_range.as_millis() as u64)
        };
        let random_timeout = self.config.election_timeout_min + Duration::from_millis(random_millis);
        
        let new_deadline = SystemTime::now() + random_timeout;
        *self.election_deadline.write().await = new_deadline;
        
        debug!(
            node_id = %self.node_id,
            timeout_ms = %random_timeout.as_millis(),
            "Reset election timer"
        );
    }
    
    /// Transition to follower state
    pub async fn become_follower(&self, term: u64, leader: Option<String>) {
        let mut current_state = self.state.write().await;
        let old_state = *current_state;
        
        if old_state != NodeState::Follower {
            info!(
                node_id = %self.node_id,
                term = %term,
                leader = ?leader,
                "Transitioning to follower"
            );
            
            *current_state = NodeState::Follower;
            drop(current_state);
            
            *self.current_term.write().await = term;
            *self.voted_for.write().await = None;
            *self.current_leader.write().await = leader.clone();
            *self.leader_lease.write().await = None;
            
            self.reset_election_timer().await;
            
            // Trigger state change callback
            if let Some(callback) = &*self.on_state_change.read().await {
                callback(old_state, NodeState::Follower);
            }
            
            // Trigger leader change callback if leader changed
            if leader.is_some() {
                if let Some(callback) = &*self.on_leader_change.read().await {
                    callback(None, leader);
                }
            }
        }
    }
    
    /// Transition to candidate state
    pub async fn become_candidate(&self) {
        let mut current_state = self.state.write().await;
        let old_state = *current_state;
        
        info!(
            node_id = %self.node_id,
            "Transitioning to candidate"
        );
        
        *current_state = NodeState::Candidate;
        drop(current_state);
        
        // Increment term
        let mut term = self.current_term.write().await;
        *term += 1;
        let new_term = *term;
        drop(term);
        
        // Vote for self
        *self.voted_for.write().await = Some(self.node_id.clone());
        *self.current_leader.write().await = None;
        *self.leader_lease.write().await = None;
        
        // Clear vote tracking
        self.votes_received.lock().await.clear();
        self.votes_received.lock().await.insert(self.node_id.clone(), true);
        
        self.reset_election_timer().await;
        
        // Trigger state change callback
        if let Some(callback) = &*self.on_state_change.read().await {
            callback(old_state, NodeState::Candidate);
        }
        
        debug!(
            node_id = %self.node_id,
            term = %new_term,
            "Became candidate, voted for self"
        );
    }
    
    /// Transition to leader state
    pub async fn become_leader(&self) {
        let mut current_state = self.state.write().await;
        let old_state = *current_state;
        
        let term = *self.current_term.read().await;
        let msg = format!("Transitioning to leader: node={}, term={}", self.node_id, term);
        info!("{}", msg);
        
        *current_state = NodeState::Leader;
        drop(current_state);
        
        let term = *self.current_term.read().await;
        *self.current_leader.write().await = Some(self.node_id.clone());
        
        // Create leader lease
        let lease = LeaderLease::new(self.node_id.clone(), term);
        *self.leader_lease.write().await = Some(lease);
        
        // Reset heartbeat tracking
        *self.last_heartbeat.write().await = SystemTime::now();
        
        // Trigger callbacks
        if let Some(callback) = &*self.on_state_change.read().await {
            callback(old_state, NodeState::Leader);
        }
        
        if let Some(callback) = &*self.on_leader_change.read().await {
            callback(None, Some(self.node_id.clone()));
        }
    }
    
    /// Update cluster membership
    pub async fn update_member(&self, node_info: NodeInfo) {
        let mut members = self.cluster_members.write().await;
        members.insert(node_info.node_id.clone(), node_info);
    }
    
    /// Remove member from cluster
    pub async fn remove_member(&self, node_id: &str) {
        let mut members = self.cluster_members.write().await;
        members.remove(node_id);
    }
    
    /// Get current cluster size
    pub async fn cluster_size(&self) -> usize {
        self.cluster_members.read().await.len() + 1 // +1 for self
    }
    
    /// Check if we have majority votes
    pub async fn has_majority(&self, votes: usize) -> bool {
        let cluster_size = self.cluster_size().await;
        votes > cluster_size / 2
    }
    
    /// Record heartbeat from leader
    pub async fn record_heartbeat(&self, leader_id: &str, term: u64) {
        let current_term = *self.current_term.read().await;
        
        if term >= current_term {
            *self.last_heartbeat.write().await = SystemTime::now();
            self.reset_election_timer().await;
            
            if term > current_term || self.current_leader.read().await.as_ref() != Some(&leader_id.to_string()) {
                self.become_follower(term, Some(leader_id.to_string())).await;
            }
        }
    }
    
    /// Check if this node is currently the leader
    pub async fn is_leader(&self) -> bool {
        let state = *self.state.read().await;
        let current_leader = self.current_leader.read().await;
        let leader_lease = self.leader_lease.read().await;
        
        // A node is the leader if:
        // 1. Its state is Leader
        // 2. It believes it is the current leader
        // 3. It has a valid lease
        state == NodeState::Leader 
            && current_leader.as_ref() == Some(&self.node_id)
            && leader_lease.as_ref().map_or(false, |lease| lease.is_valid())
    }
}