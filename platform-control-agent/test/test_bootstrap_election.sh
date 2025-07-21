#!/bin/bash
# Test script for bootstrap election system

set -e

echo "=== Testing Bootstrap Election System ==="

# Build the bootstrap module
echo "Building platform-bootstrap..."
cargo build -p platform-bootstrap

# Run tests
echo "Running election tests..."
cargo test -p platform-bootstrap election_test

# Start a 3-node cluster in the background
echo "Starting 3-node test cluster..."

# Node 1
RUST_LOG=platform_bootstrap=debug cargo run -p platform-bootstrap -- \
    --node-id node-1 \
    --bind 127.0.0.1:50101 \
    --members "127.0.0.1:50102,127.0.0.1:50103" \
    --priority 100 \
    --dev-mode &
PID1=$!

# Node 2
RUST_LOG=platform_bootstrap=debug cargo run -p platform-bootstrap -- \
    --node-id node-2 \
    --bind 127.0.0.1:50102 \
    --members "127.0.0.1:50101,127.0.0.1:50103" \
    --priority 90 \
    --dev-mode &
PID2=$!

# Node 3
RUST_LOG=platform_bootstrap=debug cargo run -p platform-bootstrap -- \
    --node-id node-3 \
    --bind 127.0.0.1:50103 \
    --members "127.0.0.1:50101,127.0.0.1:50102" \
    --priority 80 \
    --dev-mode &
PID3=$!

echo "Nodes started with PIDs: $PID1, $PID2, $PID3"
echo "Waiting 10 seconds for election..."
sleep 10

# Check leader status
echo "Checking leader status..."
for port in 50101 50102 50103; do
    echo "Node on port $port:"
    grpcurl -plaintext \
        -d '{}' \
        localhost:$port \
        platform.bootstrap.election.v1alpha1.ElectionService/GetLeader || true
done

# Cleanup
echo "Stopping nodes..."
kill $PID1 $PID2 $PID3 2>/dev/null || true

echo "=== Test Complete ==="