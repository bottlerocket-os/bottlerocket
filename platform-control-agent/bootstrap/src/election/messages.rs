// Re-export message types from proto
pub use crate::proto::election::{
    VoteRequest, VoteResponse,
    HeartbeatRequest,
};

use serde::{Serialize, Deserialize};

/// Internal message for pre-vote coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreVoteMessage {
    pub node_id: String,
    pub term: u64,
    pub timestamp: i64,
}

/// Signed vote for cryptographic proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedVote {
    pub voter_id: String,
    pub candidate_id: String,
    pub term: u64,
    pub granted: bool,
    pub timestamp: i64,
    pub signature: Vec<u8>,
}

impl SignedVote {
    /// Create a new signed vote (placeholder - real implementation would sign)
    pub fn new(voter_id: String, candidate_id: String, term: u64, granted: bool) -> Self {
        Self {
            voter_id,
            candidate_id,
            term,
            granted,
            timestamp: chrono::Utc::now().timestamp(),
            signature: vec![], // TODO: Implement actual signing
        }
    }
    
    /// Verify the signature (placeholder)
    pub fn verify(&self) -> bool {
        // TODO: Implement actual verification
        true
    }
}