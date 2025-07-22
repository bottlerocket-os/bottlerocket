use std::sync::Arc;
use std::time::Duration;
use platform_bootstrap::election::{ElectionState, NodeInfo, ElectionConfig};

#[tokio::test]
async fn test_election_state_transitions() {
    let node_info = NodeInfo {
        node_id: "test-node-1".to_string(),
        address: "127.0.0.1:50100".to_string(),
        uptime: Duration::from_secs(300),
        cpu_available_percent: 80.0,
        memory_available_gb: 8.0,
        packet_loss_percent: 0.0,
        election_priority: 100,
    };
    
    let config = ElectionConfig::default();
    let state = Arc::new(ElectionState::new(
        "test-node-1".to_string(),
        node_info,
        config,
    ));
    
    // Test initial state is follower
    assert_eq!(*state.state.read().await, platform_bootstrap::election::NodeState::Follower);
    
    // Test becoming candidate
    state.become_candidate().await;
    assert_eq!(*state.state.read().await, platform_bootstrap::election::NodeState::Candidate);
    assert_eq!(*state.current_term.read().await, 1);
    assert_eq!(*state.voted_for.read().await, Some("test-node-1".to_string()));
    
    // Test becoming leader
    state.become_leader().await;
    assert_eq!(*state.state.read().await, platform_bootstrap::election::NodeState::Leader);
    assert_eq!(*state.current_leader.read().await, Some("test-node-1".to_string()));
    
    // Test stepping down
    state.become_follower(2, Some("test-node-2".to_string())).await;
    assert_eq!(*state.state.read().await, platform_bootstrap::election::NodeState::Follower);
    assert_eq!(*state.current_term.read().await, 2);
    assert_eq!(*state.current_leader.read().await, Some("test-node-2".to_string()));
}

#[test]
fn test_priority_score_calculation() {
    use platform_bootstrap::election::PriorityScore;
    
    let node_info = NodeInfo {
        node_id: "test-node".to_string(),
        address: "127.0.0.1:50100".to_string(),
        uptime: Duration::from_secs(3600), // 1 hour = 60 points
        cpu_available_percent: 80.0, // 80 * 2.5 = 200 points
        memory_available_gb: 8.0, // 8 * 25 = 200 points
        packet_loss_percent: 0.0, // (100 - 0) * 4 = 400 points
        election_priority: 100, // 100 points
    };
    
    let score = PriorityScore::calculate(&node_info);
    
    // Base score is hash-based, so we can't predict it exactly
    assert!(score.base_score < 100);
    
    // Stability bonus: 60 (uptime) + 400 (network) = 460
    assert_eq!(score.stability_bonus, 460);
    
    // Resource bonus: 200 (cpu) + 200 (memory) = 400
    assert_eq!(score.resource_bonus, 400);
    
    // User priority
    assert_eq!(score.user_priority, 100);
    
    // Total should be sum of all components
    let total = score.total();
    assert_eq!(total, score.base_score + 460 + 400 + 100);
}