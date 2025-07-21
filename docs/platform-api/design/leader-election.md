# Design: Leader Election System

## Overview
This document details the design of the leader election system for the Bottlerocket Platform API's cluster bootstrap feature. The system enables deterministic, fault-tolerant leader election among control plane nodes.

## Design Goals

1. **Deterministic**: Given the same set of nodes, the same leader should be elected
2. **Fast Convergence**: Election should complete within 30 seconds
3. **Split-Brain Prevention**: At most one leader at any time
4. **Network Partition Tolerant**: Handle temporary network splits gracefully
5. **Observable**: Full visibility into election state and history
6. **Secure**: Prevent unauthorized participation or manipulation

## Technical Approach

### Algorithm Choice: Modified Raft

We use a modified Raft consensus algorithm with the following enhancements:

1. **Priority-Based Voting**: Nodes vote based on candidate priority scores
2. **Pre-Vote Phase**: Reduces disruption from partitioned nodes
3. **Leader Lease**: Time-based leadership with automatic renewal
4. **Witness Nodes**: Support for non-voting observer nodes

### State Machine

```
                    ┌─────────────┐
                    │  FOLLOWER   │
                    │ (initial)   │
                    └─────┬───────┘
                          │ election timeout
                          ▼
                    ┌─────────────┐
                    │  CANDIDATE  │
                    │ (pre-vote)  │
                    └─────┬───────┘
                          │ pre-vote success
                          ▼
                    ┌─────────────┐
         ┌──────────│  CANDIDATE  │──────────┐
         │          │   (vote)    │          │
         │          └─────┬───────┘          │
    lose │                │ win              │ higher term
         │                ▼                  │ discovered
         │          ┌─────────────┐          │
         └──────────│   LEADER    │──────────┘
                    │  (active)   │
                    └─────────────┘
```

## Implementation Details

### Priority Score Calculation

```rust
pub struct PriorityScore {
    base_score: u64,
    stability_bonus: u64,
    resource_bonus: u64,
    user_priority: u64,
}

impl PriorityScore {
    pub fn calculate(node: &NodeInfo) -> Self {
        let mut score = PriorityScore::default();
        
        // Base score from node ID (deterministic tiebreaker)
        score.base_score = hash(node.id) % 100;
        
        // Stability bonus (up to 1000 points)
        // - Uptime: 1 point per minute, max 600
        score.stability_bonus += min(node.uptime_seconds / 60, 600);
        // - Network stability: 0-400 based on packet loss
        score.stability_bonus += (100 - node.packet_loss_percent) * 4;
        
        // Resource bonus (up to 500 points)
        // - CPU availability: 0-250
        score.resource_bonus += (node.cpu_available_percent * 250) / 100;
        // - Memory availability: 0-250
        score.resource_bonus += min(node.memory_available_gb * 25, 250);
        
        // User-defined priority (0-1000)
        score.user_priority = node.config.election_priority;
        
        score
    }
    
    pub fn total(&self) -> u64 {
        self.base_score + self.stability_bonus + 
        self.resource_bonus + self.user_priority
    }
}
```

### Message Types

```protobuf
// Pre-vote request (doesn't increment term)
message PreVoteRequest {
    string candidate_id = 1;
    int64 term = 2;
    int64 last_log_index = 3;
    int64 last_log_term = 4;
    PriorityScore priority = 5;
}

// Vote request (increments term)
message VoteRequest {
    string candidate_id = 1;
    int64 term = 2;
    int64 last_log_index = 3;
    int64 last_log_term = 4;
    PriorityScore priority = 5;
    bytes pre_vote_proof = 6;  // Signed pre-vote responses
}

// Vote response
message VoteResponse {
    int64 term = 1;
    bool vote_granted = 2;
    string reason = 3;  // For debugging
    bytes signature = 4;  // Signature of the vote
}

// Leader heartbeat
message HeartbeatRequest {
    string leader_id = 1;
    int64 term = 2;
    int64 commit_index = 3;
    google.protobuf.Timestamp lease_expiry = 4;
}
```

### Election Process

```rust
impl ElectionState {
    async fn start_election(&mut self) -> Result<()> {
        // Phase 1: Pre-vote
        self.state = NodeState::Candidate;
        self.current_term += 1;
        
        let pre_vote_req = PreVoteRequest {
            candidate_id: self.node_id.clone(),
            term: self.current_term,
            priority: self.calculate_priority(),
            ..Default::default()
        };
        
        let pre_votes = self.broadcast_pre_vote(pre_vote_req).await?;
        if pre_votes.len() <= self.cluster_size / 2 {
            self.become_follower();
            return Ok(());
        }
        
        // Phase 2: Actual vote
        let vote_req = VoteRequest {
            candidate_id: self.node_id.clone(),
            term: self.current_term,
            priority: self.calculate_priority(),
            pre_vote_proof: self.sign_pre_votes(&pre_votes),
            ..Default::default()
        };
        
        let votes = self.broadcast_vote(vote_req).await?;
        if votes.len() > self.cluster_size / 2 {
            self.become_leader().await?;
        } else {
            self.become_follower();
        }
        
        Ok(())
    }
    
    async fn handle_vote_request(&mut self, req: VoteRequest) -> VoteResponse {
        let mut vote_granted = false;
        let mut reason = String::new();
        
        // Check term
        if req.term < self.current_term {
            reason = "Outdated term".to_string();
        }
        // Check if already voted in this term
        else if self.voted_for.is_some() && self.voted_for != Some(req.candidate_id.clone()) {
            reason = "Already voted for another candidate".to_string();
        }
        // Check pre-vote proof
        else if !self.verify_pre_vote_proof(&req.pre_vote_proof) {
            reason = "Invalid pre-vote proof".to_string();
        }
        // Compare priorities
        else if let Some(current_leader) = &self.current_leader {
            let leader_priority = self.get_node_priority(current_leader);
            if req.priority.total() <= leader_priority.total() {
                reason = "Candidate priority not higher than current leader".to_string();
            } else {
                vote_granted = true;
                self.voted_for = Some(req.candidate_id);
            }
        } else {
            vote_granted = true;
            self.voted_for = Some(req.candidate_id);
        }
        
        VoteResponse {
            term: self.current_term,
            vote_granted,
            reason,
            signature: self.sign_vote(vote_granted, req.candidate_id),
        }
    }
}
```

### Leader Lease Mechanism

```rust
pub struct LeaderLease {
    holder: String,
    expiry: SystemTime,
    term: u64,
}

impl LeaderLease {
    const LEASE_DURATION: Duration = Duration::from_secs(10);
    const RENEWAL_INTERVAL: Duration = Duration::from_secs(3);
    
    pub fn new(leader_id: String, term: u64) -> Self {
        Self {
            holder: leader_id,
            expiry: SystemTime::now() + Self::LEASE_DURATION,
            term,
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
}
```

## Network Partition Handling

### Scenario 1: Leader Isolated
```
[A*] | [B, C]  (* = leader)

1. A continues as leader but cannot commit
2. B and C detect leader timeout
3. B or C becomes new leader
4. When partition heals, A steps down (higher term)
```

### Scenario 2: Minority Partition
```
[A, B*] | [C]  (* = leader)

1. B remains leader (has majority)
2. C cannot elect new leader (minority)
3. When partition heals, C accepts B's leadership
```

### Scenario 3: Even Split (4 nodes)
```
[A*, B] | [C, D]  (* = leader)

1. Neither partition can elect leader (no majority)
2. System remains unavailable
3. Requires witness node or manual intervention
```

## Security Measures

### 1. Authentication
- All election messages signed with node's private key
- Certificates validated against cluster CA
- Node identity verified before accepting votes

### 2. Authorization
- Only nodes with `control-plane` role can participate
- Role verified through signed attributes in certificate
- Audit log of all election attempts

### 3. Anti-Tampering
- Pre-vote proof prevents vote manipulation
- Monotonic term counter prevents replay attacks
- Leader lease prevents unauthorized takeover

## Monitoring & Observability

### Metrics
```prometheus
# Current node state (1=follower, 2=candidate, 3=leader)
platform_election_node_state 1

# Current term number
platform_election_term 42

# Time since last leader contact
platform_election_last_leader_contact_seconds 2.5

# Election attempts
platform_election_attempts_total{result="success"} 5
platform_election_attempts_total{result="failed"} 2

# Vote statistics
platform_election_votes_cast_total 47
platform_election_votes_received_total 23

# Priority score components
platform_election_priority_score{component="base"} 42
platform_election_priority_score{component="stability"} 850
platform_election_priority_score{component="resource"} 425
platform_election_priority_score{component="user"} 100
```

### Events
```json
{
  "type": "election.started",
  "timestamp": "2025-01-21T10:00:00Z",
  "node_id": "node-1",
  "term": 42,
  "reason": "leader_timeout"
}

{
  "type": "election.completed",
  "timestamp": "2025-01-21T10:00:15Z",
  "node_id": "node-1",
  "term": 42,
  "result": "elected",
  "votes_received": 2,
  "votes_needed": 2
}

{
  "type": "leader.changed",
  "timestamp": "2025-01-21T10:00:16Z",
  "previous_leader": "node-3",
  "new_leader": "node-1",
  "term": 42
}
```

## Testing Strategy

### Unit Tests
1. Priority calculation correctness
2. Vote counting logic
3. State transitions
4. Message serialization
5. Lease management

### Integration Tests
1. Three-node election
2. Five-node election
3. Leader failure detection
4. Network partition scenarios
5. Concurrent elections

### Chaos Tests
1. Random network delays
2. Packet loss simulation  
3. Clock skew scenarios
4. Byzantine node behavior
5. Resource exhaustion

## Performance Considerations

### Optimization Points
1. **Batch Processing**: Collect votes for 100ms before processing
2. **Connection Pooling**: Reuse gRPC connections between nodes
3. **Parallel Voting**: Send vote requests concurrently
4. **Caching**: Cache priority scores for lease duration

### Benchmarks
- Election completion: < 5 seconds (p50), < 30 seconds (p99)
- Message latency: < 10ms within datacenter
- CPU usage during election: < 10% of one core
- Memory usage: < 100MB resident

## Future Enhancements

### Phase 1 (MVP)
- Basic Raft implementation
- Priority-based voting
- Simple monitoring

### Phase 2
- Pre-vote optimization
- Witness node support
- Advanced metrics

### Phase 3
- Multi-region awareness
- Pluggable priority algorithms
- Machine learning for priority tuning

## Related Documents
- [Cluster Bootstrap Feature](../features/cluster-bootstrap.md)
- [PKI System Design](./pki-system.md)
- [etcd Formation Design](./etcd-formation.md)