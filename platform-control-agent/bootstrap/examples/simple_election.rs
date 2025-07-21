use std::sync::Arc;
use std::time::Duration;
use platform_bootstrap::election::{ElectionState, NodeInfo, ElectionConfig, NodeState};

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("platform_bootstrap=debug")
        .init();

    println!("=== Simple Election Test ===\n");

    // Create 3 nodes
    let nodes = vec![
        ("node-1", 100, 600, 80.0, 8.0),  // High priority, long uptime
        ("node-2", 90, 300, 70.0, 6.0),   // Medium priority
        ("node-3", 80, 60, 60.0, 4.0),    // Low priority, new node
    ];

    let mut states = Vec::new();

    for (id, priority, uptime_secs, cpu, memory) in nodes {
        let node_info = NodeInfo {
            node_id: id.to_string(),
            address: format!("127.0.0.1:{}", 50100 + states.len()),
            uptime: Duration::from_secs(uptime_secs),
            cpu_available_percent: cpu,
            memory_available_gb: memory,
            packet_loss_percent: 0.0,
            election_priority: priority,
        };

        let config = ElectionConfig::default();
        let state = Arc::new(ElectionState::new(
            id.to_string(),
            node_info.clone(),
            config,
        ));

        // Calculate and display priority score
        let score = platform_bootstrap::election::PriorityScore::calculate(&node_info);
        println!("Node: {}", id);
        println!("  Priority Score: {}", score.total());
        println!("    Base: {}", score.base_score);
        println!("    Stability: {} (uptime: {}s)", score.stability_bonus, uptime_secs);
        println!("    Resources: {} (cpu: {}%, mem: {}GB)", score.resource_bonus, cpu, memory);
        println!("    User Priority: {}", score.user_priority);
        println!();

        states.push((id, state, score.total()));
    }

    // Simulate election - node with highest score should win
    println!("=== Election Simulation ===");
    let (winner_id, _, winner_score) = states.iter()
        .max_by_key(|(_, _, score)| score)
        .unwrap();

    println!("Expected winner: {} (score: {})", winner_id, winner_score);

    // Test state transitions
    println!("\n=== State Transitions ===");
    let test_state = &states[0].1;
    
    println!("Initial state: {:?}", *test_state.state.read().await);
    
    test_state.become_candidate().await;
    println!("After become_candidate: {:?}", *test_state.state.read().await);
    println!("Term: {}", *test_state.current_term.read().await);
    println!("Voted for: {:?}", *test_state.voted_for.read().await);
    
    test_state.become_leader().await;
    println!("\nAfter become_leader: {:?}", *test_state.state.read().await);
    println!("Current leader: {:?}", *test_state.current_leader.read().await);
    
    test_state.become_follower(2, Some("node-2".to_string())).await;
    println!("\nAfter become_follower: {:?}", *test_state.state.read().await);
    println!("Term: {}", *test_state.current_term.read().await);
    println!("Current leader: {:?}", *test_state.current_leader.read().await);

    println!("\n=== Test Complete ===");
}