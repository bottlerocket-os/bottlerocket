# Election System Demo

## Priority Scoring Algorithm

The election system uses a priority-based voting mechanism where nodes with better stability and resources have higher chances of becoming the leader.

### Score Calculation

```
Total Score = Base + Stability + Resources + User Priority

Where:
- Base (0-100): Deterministic hash of node ID
- Stability (0-1000): Uptime score + Network stability
  - Uptime: 1 point per minute, max 600
  - Network: (100 - packet_loss%) × 4, max 400
- Resources (0-500): CPU score + Memory score
  - CPU: available% × 2.5, max 250
  - Memory: available_GB × 25, max 250
- User Priority (0-1000): Configured priority value
```

### Example Scenario

Three nodes competing for leadership:

| Node   | Uptime | CPU | Memory | Network | User Priority | Total Score |
|--------|--------|-----|--------|---------|---------------|-------------|
| node-1 | 10 hrs | 80% | 8 GB   | 0% loss | 100          | **1544**    |
| node-2 | 5 hrs  | 70% | 6 GB   | 1% loss | 90           | 1155        |
| node-3 | 1 hr   | 60% | 4 GB   | 5% loss | 80           | 826         |

**Winner: node-1** (most stable, best resources)

### Key Features

1. **Deterministic**: Same inputs always produce same leader
2. **Fair**: Considers multiple factors, not just randomness
3. **Configurable**: User priority allows manual preference
4. **Observable**: All scores are transparent and auditable

### Election Process

```
1. Node timeout occurs (5-10 seconds)
2. Node becomes candidate
3. Pre-vote phase (optional)
   - Reduces disruption from partitioned nodes
4. Request votes from all nodes
   - Nodes compare candidate priority scores
   - Vote for highest priority candidate
5. If majority votes received → become leader
6. Leader sends heartbeats every 1 second
7. If heartbeat missed → new election
```

### Network Partition Handling

- **Scenario**: 3 nodes split [A] | [B, C]
- **Result**: B or C becomes new leader (has majority)
- **Recovery**: When partition heals, A accepts new leader

### Why Priority-Based?

Traditional Raft uses random timeouts, which can lead to:
- Unstable nodes becoming leaders
- Frequent leader changes
- Resource-constrained nodes handling critical tasks

Priority-based voting ensures:
- ✅ Stable nodes preferred
- ✅ Resource-rich nodes handle leadership
- ✅ Predictable leader selection
- ✅ Reduced leader churn