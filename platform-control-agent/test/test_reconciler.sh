#!/bin/bash
# Test script for configuration reconciliation

set -euo pipefail

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Testing Configuration Reconciliation...${NC}"

# Start server with reconciliation enabled
echo "Starting server with reconciliation enabled..."
export RECONCILER_ENABLED=true
export RECONCILER_INTERVAL=10  # Fast interval for testing
export RECONCILER_AUTO_CORRECT=true
export SKIP_UNIX_SOCKET=true  # Skip unix socket for testing

# Start the server in background
cargo run -- serve --dev-mode &
SERVER_PID=$!

# Wait for server to start
sleep 3

# Function to apply configuration
apply_config() {
    local config_json=$1
    grpcurl -plaintext -d "$config_json" localhost:50000 platform.api.MachineService/ApplyConfiguration
}

# Function to get status
get_status() {
    grpcurl -plaintext -d '{}' localhost:50000 platform.api.MachineService/GetStatus
}

# Test 1: Apply a configuration
echo -e "\n${YELLOW}Test 1: Apply initial configuration${NC}"
CONFIG='{
  "config": {
    "version": "v1.0.0",
    "machine_type": "test-node",
    "cluster": {
      "endpoint": "https://k8s.example.com:6443",
      "ca_certificate": "test-cert",
      "dns_ip": "10.96.0.10",
      "dns_domain": "cluster.local"
    },
    "network": {
      "hostname": "test-host"
    }
  }
}'

if apply_config "$CONFIG"; then
    echo -e "${GREEN}✓ Configuration applied successfully${NC}"
else
    echo -e "${RED}✗ Failed to apply configuration${NC}"
    kill $SERVER_PID 2>/dev/null
    exit 1
fi

# Wait for reconciliation to run
echo -e "\n${YELLOW}Waiting for reconciliation loop to run...${NC}"
sleep 15

# Test 2: Check event stream for reconciliation events
echo -e "\n${YELLOW}Test 2: Check for reconciliation events${NC}"
timeout 5s grpcurl -plaintext -d '{
  "filters": {
    "event_types": ["ReconciliationStarted", "ReconciliationCompleted", "ConfigurationDriftDetected", "ConfigurationDriftCorrected"]
  }
}' localhost:50000 platform.api.MachineService/StreamEvents || true

# Test 3: Simulate drift by resetting
echo -e "\n${YELLOW}Test 3: Simulate configuration drift (reset)${NC}"
grpcurl -plaintext -d '{"graceful": true}' localhost:50000 platform.api.MachineService/Reset

# Wait for reconciliation to detect and correct drift
echo -e "\n${YELLOW}Waiting for drift detection and correction...${NC}"
sleep 15

# Test 4: Check events for drift detection
echo -e "\n${YELLOW}Test 4: Check for drift detection events${NC}"
timeout 5s grpcurl -plaintext -d '{
  "filters": {
    "event_types": ["ConfigurationDriftDetected", "ConfigurationDriftCorrected", "ReconciliationCompleted"]
  }
}' localhost:50000 platform.api.MachineService/StreamEvents || true

# Test 5: Verify configuration was restored
echo -e "\n${YELLOW}Test 5: Verify configuration status${NC}"
if get_status | grep -q "CONFIGURED"; then
    echo -e "${GREEN}✓ Configuration was restored by reconciler${NC}"
else
    echo -e "${RED}✗ Configuration was not restored${NC}"
fi

# Test 6: Test with reconciliation disabled
echo -e "\n${YELLOW}Test 6: Test with reconciliation disabled${NC}"
kill $SERVER_PID 2>/dev/null
sleep 2

export RECONCILER_ENABLED=false
cargo run -- serve --dev-mode &
SERVER_PID=$!
sleep 3

# Apply config and reset
apply_config "$CONFIG" >/dev/null
sleep 2
grpcurl -plaintext -d '{"graceful": true}' localhost:50000 platform.api.MachineService/Reset >/dev/null

# Wait and check - drift should NOT be corrected
sleep 15
if get_status | grep -q "NOT_CONFIGURED"; then
    echo -e "${GREEN}✓ Drift was not corrected (reconciliation disabled)${NC}"
else
    echo -e "${RED}✗ Drift was corrected when it shouldn't be${NC}"
fi

# Cleanup
echo -e "\n${YELLOW}Cleaning up...${NC}"
kill $SERVER_PID 2>/dev/null
rm -f state.json

echo -e "\n${GREEN}Reconciliation tests completed!${NC}"