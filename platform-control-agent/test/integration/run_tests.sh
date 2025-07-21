#!/bin/bash
# Integration test suite for Platform Control Agent

set -euo pipefail

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test configuration
PLATFORM_CONTROL_ADDR="${PLATFORM_CONTROL_ADDR:-localhost:50000}"
GRPC_OPTS="-cacert /etc/platform/ca.crt -cert /etc/platform/client.crt -key /etc/platform/client.key"

echo -e "${YELLOW}Starting Platform Control Agent Integration Tests${NC}"
echo "Target: $PLATFORM_CONTROL_ADDR"

# Wait for service to be ready
echo -e "\n${YELLOW}Waiting for service to be ready...${NC}"
for i in {1..30}; do
    if grpcurl $GRPC_OPTS $PLATFORM_CONTROL_ADDR list >/dev/null 2>&1; then
        echo -e "${GREEN}✓ Service is ready${NC}"
        break
    fi
    if [ $i -eq 30 ]; then
        echo -e "${RED}✗ Service failed to start${NC}"
        exit 1
    fi
    sleep 1
done

# Test counter
TESTS_RUN=0
TESTS_PASSED=0

# Helper function to run a test
run_test() {
    local test_name=$1
    local test_function=$2
    
    echo -e "\n${YELLOW}Test: $test_name${NC}"
    TESTS_RUN=$((TESTS_RUN + 1))
    
    if $test_function; then
        echo -e "${GREEN}✓ $test_name passed${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}✗ $test_name failed${NC}"
    fi
}

# Test 1: Health Check
test_health_check() {
    grpcurl $GRPC_OPTS \
        -d '{}' \
        $PLATFORM_CONTROL_ADDR \
        platform.api.MachineService/GetStatus >/dev/null
}

# Test 2: Apply Configuration
test_apply_configuration() {
    local response=$(grpcurl $GRPC_OPTS \
        -d '{
            "config": {
                "version": "v1.0.0",
                "type": "MACHINE_TYPE_WORKER",
                "cluster": {
                    "name": "test-cluster",
                    "endpoint": "https://k8s.example.com:6443",
                    "ca_certificate": "test-cert",
                    "bootstrap_token": "test-token",
                    "dns_ip": "10.96.0.10",
                    "dns_domain": "cluster.local"
                }
            }
        }' \
        $PLATFORM_CONTROL_ADDR \
        platform.api.MachineService/ApplyConfiguration)
    
    echo "$response" | jq -e '.success == true' >/dev/null
}

# Test 3: Get Configuration
test_get_configuration() {
    local response=$(grpcurl $GRPC_OPTS \
        -d '{}' \
        $PLATFORM_CONTROL_ADDR \
        platform.api.MachineService/GetConfiguration)
    
    echo "$response" | jq -e '.config.version == "v1.0.0"' >/dev/null
}

# Test 4: Get Status
test_get_status() {
    local response=$(grpcurl $GRPC_OPTS \
        -d '{}' \
        $PLATFORM_CONTROL_ADDR \
        platform.api.MachineService/GetStatus)
    
    echo "$response" | jq -e '.state == "STATE_CONFIGURED"' >/dev/null &&
    echo "$response" | jq -e '.uptimeSeconds > 0' >/dev/null
}

# Test 5: Validate Configuration
test_validate_configuration() {
    # Valid configuration
    local response=$(grpcurl $GRPC_OPTS \
        -d '{
            "config": {
                "version": "v2.0.0",
                "type": "MACHINE_TYPE_WORKER",
                "cluster": {
                    "name": "test-cluster",
                    "endpoint": "https://k8s.example.com:6443",
                    "ca_certificate": "test-cert",
                    "bootstrap_token": "test-token",
                    "dns_ip": "10.96.0.10",
                    "dns_domain": "cluster.local"
                }
            }
        }' \
        $PLATFORM_CONTROL_ADDR \
        platform.api.MachineService/ValidateConfiguration)
    
    echo "$response" | jq -e '.valid == true' >/dev/null
}

# Test 6: Stream Events
test_stream_events() {
    # Start streaming events in background
    timeout 5s grpcurl $GRPC_OPTS \
        -d '{"filters": {"eventTypes": ["ConfigurationApplied"]}}' \
        $PLATFORM_CONTROL_ADDR \
        platform.api.MachineService/StreamEvents > events.log 2>&1 &
    
    sleep 1
    
    # Apply a configuration to generate an event
    grpcurl $GRPC_OPTS \
        -d '{
            "config": {
                "version": "v3.0.0",
                "type": "MACHINE_TYPE_WORKER",
                "cluster": {
                    "name": "test-cluster",
                    "endpoint": "https://k8s.example.com:6443",
                    "ca_certificate": "test-cert",
                    "bootstrap_token": "test-token",
                    "dns_ip": "10.96.0.10",
                    "dns_domain": "cluster.local"
                }
            }
        }' \
        $PLATFORM_CONTROL_ADDR \
        platform.api.MachineService/ApplyConfiguration >/dev/null
    
    sleep 2
    
    # Check if we received events
    grep -q "ConfigurationApplied" events.log
}

# Test 7: Reset
test_reset() {
    local response=$(grpcurl $GRPC_OPTS \
        -d '{"graceful": true, "clearData": true}' \
        $PLATFORM_CONTROL_ADDR \
        platform.api.MachineService/Reset)
    
    echo "$response" | jq -e '.success == true' >/dev/null
    
    # Verify configuration was cleared
    sleep 1
    ! grpcurl $GRPC_OPTS \
        -d '{}' \
        $PLATFORM_CONTROL_ADDR \
        platform.api.MachineService/GetConfiguration >/dev/null 2>&1
}

# Test 8: Reconciliation
test_reconciliation() {
    # Apply a configuration
    grpcurl $GRPC_OPTS \
        -d '{
            "config": {
                "version": "v4.0.0",
                "type": "MACHINE_TYPE_WORKER",
                "cluster": {
                    "name": "reconcile-test",
                    "endpoint": "https://k8s.example.com:6443",
                    "ca_certificate": "test-cert",
                    "bootstrap_token": "test-token",
                    "dns_ip": "10.96.0.10",
                    "dns_domain": "cluster.local"
                }
            }
        }' \
        $PLATFORM_CONTROL_ADDR \
        platform.api.MachineService/ApplyConfiguration >/dev/null
    
    # Wait for reconciliation to run
    echo "Waiting for reconciliation cycle..."
    sleep 35
    
    # Check for reconciliation events
    timeout 5s grpcurl $GRPC_OPTS \
        -d '{"filters": {"eventTypes": ["ReconciliationCompleted"]}}' \
        $PLATFORM_CONTROL_ADDR \
        platform.api.MachineService/StreamEvents | grep -q "ReconciliationCompleted"
}

# Run all tests
run_test "Health Check" test_health_check
run_test "Apply Configuration" test_apply_configuration
run_test "Get Configuration" test_get_configuration
run_test "Get Status" test_get_status
run_test "Validate Configuration" test_validate_configuration
run_test "Stream Events" test_stream_events
run_test "Reset" test_reset
run_test "Reconciliation" test_reconciliation

# Summary
echo -e "\n${YELLOW}Test Summary${NC}"
echo "Tests run: $TESTS_RUN"
echo "Tests passed: $TESTS_PASSED"
echo "Tests failed: $((TESTS_RUN - TESTS_PASSED))"

if [ $TESTS_PASSED -eq $TESTS_RUN ]; then
    echo -e "\n${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "\n${RED}Some tests failed!${NC}"
    exit 1
fi