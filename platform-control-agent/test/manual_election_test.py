#!/usr/bin/env python3
"""
Manual test for election priority scoring
"""

import random

class NodeInfo:
    def __init__(self, node_id, uptime_mins, cpu_percent, memory_gb, packet_loss, user_priority):
        self.node_id = node_id
        self.uptime_mins = uptime_mins
        self.cpu_percent = cpu_percent
        self.memory_gb = memory_gb
        self.packet_loss = packet_loss
        self.user_priority = user_priority

def calculate_priority_score(node):
    """Calculate priority score based on Rust implementation"""
    # Base score from node ID hash (0-100)
    base_score = hash(node.node_id) % 100
    
    # Stability bonus (up to 1000)
    uptime_score = min(node.uptime_mins, 600)  # Max 600
    network_score = (100 - node.packet_loss) * 4  # Max 400
    stability_bonus = uptime_score + network_score
    
    # Resource bonus (up to 500)
    cpu_score = min(node.cpu_percent * 2.5, 250)  # Max 250
    memory_score = min(node.memory_gb * 25, 250)  # Max 250
    resource_bonus = cpu_score + memory_score
    
    # Total
    total = base_score + stability_bonus + resource_bonus + node.user_priority
    
    return {
        'base': base_score,
        'stability': stability_bonus,
        'resources': resource_bonus,
        'user': node.user_priority,
        'total': total
    }

def simulate_election():
    """Simulate an election with 3 nodes"""
    print("=== Election Priority Scoring Test ===\n")
    
    nodes = [
        NodeInfo("node-1", 600, 80.0, 8.0, 0.0, 100),  # Stable node
        NodeInfo("node-2", 300, 70.0, 6.0, 1.0, 90),   # Medium node
        NodeInfo("node-3", 60, 60.0, 4.0, 5.0, 80),    # New node
    ]
    
    results = []
    
    for node in nodes:
        score = calculate_priority_score(node)
        results.append((node, score))
        
        print(f"Node: {node.node_id}")
        print(f"  Uptime: {node.uptime_mins} minutes")
        print(f"  CPU Available: {node.cpu_percent}%")
        print(f"  Memory Available: {node.memory_gb} GB")
        print(f"  Packet Loss: {node.packet_loss}%")
        print(f"  User Priority: {node.user_priority}")
        print(f"\n  Priority Score Breakdown:")
        print(f"    Base Score: {score['base']}")
        print(f"    Stability Bonus: {score['stability']} (uptime: {min(node.uptime_mins, 600)}, network: {(100 - node.packet_loss) * 4})")
        print(f"    Resource Bonus: {score['resources']} (cpu: {min(node.cpu_percent * 2.5, 250):.0f}, memory: {min(node.memory_gb * 25, 250):.0f})")
        print(f"    User Priority: {score['user']}")
        print(f"    TOTAL: {score['total']}")
        print()
    
    # Find winner
    winner = max(results, key=lambda x: x[1]['total'])
    print(f"=== Election Winner: {winner[0].node_id} with score {winner[1]['total']} ===")

if __name__ == "__main__":
    simulate_election()