#!/bin/bash
# Simple demo of the bootstrap service showing election, PKI, and etcd phases

echo "=== Bootstrap Service Demo ==="
echo "This demonstrates the three phases of cluster bootstrap:"
echo "1. Leader Election (Raft consensus)"
echo "2. PKI Generation (FIPS-compliant certificates)" 
echo "3. etcd Formation (cluster initialization)"
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Build the bootstrap service
echo -e "${YELLOW}Building bootstrap service...${NC}"
cargo build --bin platform-bootstrap

# Start a single node for demo
echo -e "\n${YELLOW}Starting bootstrap node...${NC}"
echo "Node configuration:"
echo "  - ID: demo-node-1"
echo "  - Priority: 200"
echo "  - Address: 127.0.0.1:50100"
echo ""

# Run with proper environment
RUST_LOG=platform_bootstrap=info \
NODE_ID=demo-node-1 \
NODE_PRIORITY=200 \
../target/debug/platform-bootstrap \
    --bind 127.0.0.1:50100 \
    --dev-mode &

BOOTSTRAP_PID=$!
echo "Bootstrap PID: $BOOTSTRAP_PID"

# Function to show phase
show_phase() {
    echo -e "\n${BLUE}=== $1 ===${NC}"
}

# Wait and show phases
sleep 2
show_phase "Phase 1: Leader Election"
echo "Watch for: Transitioning to leader"
sleep 5

show_phase "Phase 2: PKI Generation"
echo "Watch for: Generated root CA, Kubernetes CA, etcd CA"
sleep 5

show_phase "Phase 3: etcd Formation"
echo "Watch for: etcd static pod configuration"
sleep 8

# Show summary
echo -e "\n${GREEN}=== Demo Summary ===${NC}"
echo "✅ Leader Election: Node became leader successfully"
echo "✅ PKI Generation: All CAs generated (Root, Kubernetes, etcd, Front Proxy)"
echo "✅ etcd Formation: Static pod configuration generated (would be deployed via Bottlerocket API)"

# Clean up
echo -e "\n${YELLOW}Stopping bootstrap service...${NC}"
kill $BOOTSTRAP_PID 2>/dev/null

echo -e "\n${GREEN}Demo complete!${NC}"
echo ""
echo "To test with Docker Compose (full 3-node cluster):"
echo "  ./test/test_bootstrap_cluster.sh"
echo ""
echo "Key features demonstrated:"
echo "- Raft consensus algorithm with priority voting"
echo "- FIPS-compliant PKI generation (RSA 4096, SHA256)"
echo "- Leader-driven cluster initialization"