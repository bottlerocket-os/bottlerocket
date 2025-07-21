mod state;
mod service;
mod algorithm;
mod messages;

pub use state::{ElectionState, NodeState, PriorityScore, ElectionConfig};
pub use service::ElectionService;
pub use messages::{VoteRequest, VoteResponse, HeartbeatRequest};

// Re-export proto types
pub use crate::proto::election::*;