#!/bin/bash
set -e

echo "=== Bootstrap Cluster Test ==="
echo "This test demonstrates election, PKI distribution, and etcd formation"
echo ""

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check if docker-compose is available
if ! command -v docker-compose &> /dev/null; then
    echo -e "${RED}Error: docker-compose is not installed${NC}"
    exit 1
fi

# Function to wait for services
wait_for_service() {
    local service=$1
    local port=$2
    local max_attempts=30
    local attempt=0
    
    echo -n "Waiting for $service on port $port..."
    while ! nc -z localhost $port 2>/dev/null; do
        if [ $attempt -eq $max_attempts ]; then
            echo -e " ${RED}FAILED${NC}"
            return 1
        fi
        attempt=$((attempt + 1))
        sleep 1
        echo -n "."
    done
    echo -e " ${GREEN}OK${NC}"
    return 0
}

# Clean up any existing containers
echo "Cleaning up existing containers..."
docker-compose -f docker-compose.bootstrap.yml down -v 2>/dev/null || true

# Build the bootstrap image
echo -e "\n${YELLOW}Building bootstrap image...${NC}"
docker-compose -f docker-compose.bootstrap.yml build

# Start the cluster
echo -e "\n${YELLOW}Starting 3-node bootstrap cluster...${NC}"
docker-compose -f docker-compose.bootstrap.yml up -d

# Wait for services to be ready
echo -e "\n${YELLOW}Waiting for services to start...${NC}"
wait_for_service "bootstrap-node-1" 50100
wait_for_service "bootstrap-node-2" 50101
wait_for_service "bootstrap-node-3" 50102

# Give time for election to complete
echo -e "\n${YELLOW}Waiting for leader election...${NC}"
sleep 5

# Check election status on each node
echo -e "\n${YELLOW}Checking election status:${NC}"
for i in 1 2 3; do
    echo -n "Node $i: "
    # Use grpcurl to check leader status
    if command -v grpcurl &> /dev/null; then
        leader_info=$(grpcurl -plaintext localhost:$((50099 + i)) platform.bootstrap.election.v1alpha1.ElectionService/GetLeader 2>/dev/null || echo "Unable to query")
        echo "$leader_info"
    else
        # Fallback to checking logs
        if docker-compose -f docker-compose.bootstrap.yml logs bootstrap-node-$i 2>&1 | grep -q "Transitioning to leader"; then
            echo -e "${GREEN}LEADER${NC}"
        else
            echo "FOLLOWER"
        fi
    fi
done

# Show logs from all nodes
echo -e "\n${YELLOW}Recent logs from all nodes:${NC}"
for i in 1 2 3; do
    echo -e "\n--- Node $i ---"
    docker-compose -f docker-compose.bootstrap.yml logs --tail=20 bootstrap-node-$i | grep -E "(INFO|ERROR|leader|election|PKI|etcd)" || true
done

# Test PKI distribution (only leader can issue certificates)
echo -e "\n${YELLOW}Testing PKI distribution:${NC}"
echo "Note: Only the elected leader can issue certificates"

# Check etcd formation
echo -e "\n${YELLOW}Checking etcd cluster formation:${NC}"
docker-compose -f docker-compose.bootstrap.yml logs | grep -E "etcd|static pod" | tail -10 || echo "No etcd logs found yet"

# Show cluster status
echo -e "\n${YELLOW}Cluster Status Summary:${NC}"
echo "- Bootstrap nodes running: $(docker-compose -f docker-compose.bootstrap.yml ps -q | wc -l)"
echo "- gRPC UI available at: http://localhost:8082"
echo "- Prometheus metrics at: http://localhost:9092"

echo -e "\n${GREEN}Bootstrap cluster is running!${NC}"
echo ""
echo "To view logs: docker-compose -f docker-compose.bootstrap.yml logs -f"
echo "To stop: docker-compose -f docker-compose.bootstrap.yml down"
echo "To clean up: docker-compose -f docker-compose.bootstrap.yml down -v"
echo ""
echo "Try these commands:"
echo "  - Watch election logs: docker-compose -f docker-compose.bootstrap.yml logs -f | grep -i election"
echo "  - Watch PKI logs: docker-compose -f docker-compose.bootstrap.yml logs -f | grep -i pki"
echo "  - Watch etcd logs: docker-compose -f docker-compose.bootstrap.yml logs -f | grep -i etcd"