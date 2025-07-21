use std::sync::Arc;
use std::pin::Pin;
use std::time::SystemTime;
use tonic::{Request, Response, Status};
use tokio_stream::{Stream, StreamExt};
use tokio::sync::mpsc;
use prost_types::Timestamp;

use crate::proto::election::{
    election_service_server::ElectionService as ElectionServiceTrait,
    GetLeaderRequest, GetLeaderResponse,
    CampaignRequest, CampaignResponse,
    ResignRequest, ResignResponse,
    ObserveRequest, ElectionEvent,
    VoteRequest, VoteResponse,
    HeartbeatRequest, HeartbeatResponse,
    ElectionState as ProtoElectionState,
    NodeState as ProtoNodeState,
};

use super::state::{ElectionState, NodeState};
use super::algorithm::RaftElection;

/// Convert SystemTime to prost Timestamp
fn system_time_to_timestamp(time: SystemTime) -> Timestamp {
    let duration = time.duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    Timestamp {
        seconds: duration.as_secs() as i64,
        nanos: duration.subsec_nanos() as i32,
    }
}

/// gRPC service implementation for election
pub struct ElectionService {
    state: Arc<ElectionState>,
    raft: Arc<RaftElection>,
    event_tx: mpsc::Sender<ElectionEvent>,
    event_rx: Arc<tokio::sync::Mutex<mpsc::Receiver<ElectionEvent>>>,
}

impl ElectionService {
    pub fn new(state: Arc<ElectionState>) -> Self {
        let raft = Arc::new(RaftElection::new(state.clone()));
        let (event_tx, event_rx) = mpsc::channel(1000);
        
        // TODO: Set up state change callbacks
        // Callback registration temporarily disabled due to async/sync issues
        // Will need to refactor to use async-safe callback mechanism
        
        // TODO: Set up leader change callbacks
        // Callback registration temporarily disabled due to async/sync issues
        // Will need to refactor to use async-safe callback mechanism
        
        Self {
            state,
            raft,
            event_tx,
            event_rx: Arc::new(tokio::sync::Mutex::new(event_rx)),
        }
    }
    
    /// Start background tasks
    pub async fn start(&self) -> anyhow::Result<()> {
        // Initialize connections
        self.raft.initialize_connections().await?;
        
        // Start election timer task
        let state_clone = self.state.clone();
        let raft_clone = self.raft.clone();
        let _handle1 = tokio::task::spawn(async move {
            Self::election_timer_task(state_clone, raft_clone).await
        });
        
        // Start heartbeat task
        let state_clone = self.state.clone();
        let raft_clone = self.raft.clone();
        let _handle2 = tokio::task::spawn(async move {
            Self::heartbeat_task(state_clone, raft_clone).await
        });
        
        Ok(())
    }
}

#[tonic::async_trait]
impl ElectionServiceTrait for ElectionService {
    async fn get_leader(
        &self,
        _request: Request<GetLeaderRequest>,
    ) -> Result<Response<GetLeaderResponse>, Status> {
        let current_term = *self.state.current_term.read().await;
        let current_state = *self.state.state.read().await;
        let current_leader = self.state.current_leader.read().await.clone();
        let last_heartbeat = *self.state.last_heartbeat.read().await;
        
        let is_leader = current_state == NodeState::Leader;
        let priority = super::state::PriorityScore::calculate(&self.state.node_info);
        
        let cluster_members = self.state.cluster_members.read().await;
        let mut member_ids: Vec<String> = cluster_members.keys().cloned().collect();
        member_ids.push(self.state.node_id.clone());
        
        let election_state = ProtoElectionState {
            node_id: self.state.node_id.clone(),
            term: current_term as i64,
            state: ProtoNodeState::from(current_state) as i32,
            leader_id: current_leader.clone().unwrap_or_default(),
            cluster_members: member_ids,
            priority: Some(priority.into()),
            last_heartbeat: Some(system_time_to_timestamp(last_heartbeat)),
        };
        
        Ok(Response::new(GetLeaderResponse {
            leader_id: current_leader.unwrap_or_default(),
            term: current_term as i64,
            last_heartbeat: Some(system_time_to_timestamp(last_heartbeat)),
            is_leader,
            state: Some(election_state),
        }))
    }
    
    async fn campaign(
        &self,
        request: Request<CampaignRequest>,
    ) -> Result<Response<CampaignResponse>, Status> {
        let req = request.into_inner();
        
        // Check current state
        let current_state = *self.state.state.read().await;
        if current_state == NodeState::Leader && !req.force {
            return Ok(Response::new(CampaignResponse {
                success: false,
                reason: "Already leader".to_string(),
                term: *self.state.current_term.read().await as i64,
            }));
        }
        
        if current_state == NodeState::Candidate && !req.force {
            return Ok(Response::new(CampaignResponse {
                success: false,
                reason: "Already campaigning".to_string(),
                term: *self.state.current_term.read().await as i64,
            }));
        }
        
        // Start election
        match self.raft.start_election().await {
            Ok(won) => {
                Ok(Response::new(CampaignResponse {
                    success: won,
                    reason: if won { "Won election".to_string() } else { "Lost election".to_string() },
                    term: *self.state.current_term.read().await as i64,
                }))
            }
            Err(e) => {
                let error_msg = format!("Failed to start election: {}", e);
                tracing::error!("{}", error_msg);
                Err(Status::internal(error_msg))
            }
        }
    }
    
    async fn resign(
        &self,
        _request: Request<ResignRequest>,
    ) -> Result<Response<ResignResponse>, Status> {
        let current_state = *self.state.state.read().await;
        if current_state != NodeState::Leader {
            return Ok(Response::new(ResignResponse {
                success: false,
                new_leader: String::new(),
            }));
        }
        
        tracing::info!("Resigning from leadership");
        let term = *self.state.current_term.read().await;
        self.state.become_follower(term, None).await;
        
        Ok(Response::new(ResignResponse {
            success: true,
            new_leader: String::new(), // Unknown until new election
        }))
    }
    
    type ObserveElectionStream = Pin<Box<dyn Stream<Item = Result<ElectionEvent, Status>> + Send>>;
    
    async fn observe_election(
        &self,
        request: Request<ObserveRequest>,
    ) -> Result<Response<Self::ObserveElectionStream>, Status> {
        let _req = request.into_inner();
        
        // Create a new receiver by cloning the sender
        let (_tx, rx) = mpsc::channel(100);
        let _event_tx = self.event_tx.clone();
        
        // Forward events from the main channel to this client
        tokio::spawn(async move {
            // This is a simplified version - in production, you'd want to
            // properly multiplex events to multiple observers
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                // Send periodic heartbeat events or state updates
            }
        });
        
        let stream = tokio_stream::wrappers::ReceiverStream::new(rx)
            .map(|event| Ok(event));
        
        Ok(Response::new(Box::pin(stream)))
    }
    
    async fn request_vote(
        &self,
        request: Request<VoteRequest>,
    ) -> Result<Response<VoteResponse>, Status> {
        let req = request.into_inner();
        
        match self.raft.handle_vote_request(req).await {
            Ok(response) => Ok(Response::new(response)),
            Err(e) => {
                let error_msg = format!("Failed to handle vote request: {}", e);
                tracing::error!("{}", error_msg);
                Err(Status::internal(error_msg))
            }
        }
    }
    
    async fn heartbeat(
        &self,
        request: Request<HeartbeatRequest>,
    ) -> Result<Response<HeartbeatResponse>, Status> {
        let req = request.into_inner();
        
        match self.raft.handle_heartbeat(req).await {
            Ok(response) => Ok(Response::new(response)),
            Err(e) => {
                let error_msg = format!("Failed to handle heartbeat: {}", e);
                tracing::error!("{}", error_msg);
                Err(Status::internal(error_msg))
            }
        }
    }
}

impl ElectionService {
    /// Background task for election timeout
    async fn election_timer_task(state: Arc<ElectionState>, raft: Arc<RaftElection>) {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            
            let current_state = *state.state.read().await;
            if current_state == NodeState::Follower && state.is_election_timeout().await {
                tracing::info!("Election timeout, starting campaign");
                if let Err(e) = raft.start_election().await {
                    let msg = format!("Failed to start election: {}", e);
                    tracing::error!("{}", msg);
                }
            }
        }
    }
    
    /// Background task for sending heartbeats
    async fn heartbeat_task(state: Arc<ElectionState>, raft: Arc<RaftElection>) {
        let heartbeat_interval = state.config.heartbeat_interval;
        loop {
            tokio::time::sleep(heartbeat_interval).await;
            
            let current_state = *state.state.read().await;
            if current_state == NodeState::Leader {
                if let Err(e) = raft.send_heartbeats().await {
                    let msg = format!("Failed to send heartbeats: {}", e);
                    tracing::error!("{}", msg);
                }
            }
        }
    }
}