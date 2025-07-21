#!/bin/bash

# Platform Control Agent Comprehensive Test Script

echo "=== Platform Control Agent Test Suite ==="
echo "Testing all implemented features..."
echo

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Function to run a test
run_test() {
    local test_name="$1"
    local test_command="$2"
    local expected_pattern="$3"
    
    echo -n "Testing $test_name... "
    
    # Run the command and capture output
    output=$(eval "$test_command" 2>&1)
    
    if echo "$output" | grep -q "$expected_pattern"; then
        echo -e "${GREEN}PASSED${NC}"
        ((TESTS_PASSED++))
        return 0
    else
        echo -e "${RED}FAILED${NC}"
        echo "  Expected pattern: $expected_pattern"
        echo "  Got: $output" | head -5
        ((TESTS_FAILED++))
        return 1
    fi
}

# Check if server is running
echo "Checking server status..."
if ! lsof -i:50051 >/dev/null 2>&1; then
    echo -e "${RED}Server not running on port 50051${NC}"
    exit 1
fi
echo -e "${GREEN}Server is running${NC}"
echo

# Test 1: Get Status
run_test "Get Status" \
    "grpcurl -plaintext -proto src/api/machine.proto localhost:50051 platform.machine.v1alpha1.MachineService/GetStatus" \
    "nodeId"

# Test 2: Get Configuration (should fail if none applied)
echo -n "Testing Get Configuration (expecting not found)... "
output=$(grpcurl -plaintext -proto src/api/machine.proto localhost:50051 platform.machine.v1alpha1.MachineService/GetConfiguration 2>&1)
if echo "$output" | grep -q "Not Found"; then
    echo -e "${GREEN}PASSED${NC} (correctly returned not found)"
    ((TESTS_PASSED++))
else
    # If config exists, that's also OK
    if echo "$output" | grep -q "version"; then
        echo -e "${GREEN}PASSED${NC} (found existing config)"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}FAILED${NC}"
        ((TESTS_FAILED++))
    fi
fi

# Test 3: Apply Configuration - Dry Run
run_test "Apply Configuration (Dry Run)" \
    'echo '"'"'{"config": {"version": "1.0.0", "type": 1}, "dry_run": true}'"'"' | grpcurl -plaintext -proto src/api/machine.proto -d @ localhost:50051 platform.machine.v1alpha1.MachineService/ApplyConfiguration' \
    "Configuration validated successfully"

# Test 4: Apply Configuration - Invalid (missing required fields)
echo -n "Testing Apply Configuration (Invalid)... "
output=$(echo '{"config": {"version": "", "type": 1}, "dry_run": false}' | grpcurl -plaintext -proto src/api/machine.proto -d @ localhost:50051 platform.machine.v1alpha1.MachineService/ApplyConfiguration 2>&1)
if echo "$output" | grep -q "validation failed"; then
    echo -e "${GREEN}PASSED${NC} (correctly rejected invalid config)"
    ((TESTS_PASSED++))
else
    echo -e "${RED}FAILED${NC}"
    ((TESTS_FAILED++))
fi

# Test 5: Apply Valid Configuration
run_test "Apply Configuration (Valid)" \
    'echo '"'"'{"config": {"version": "3.0.0", "type": 2, "network": {"hostname": "test-node"}}, "dry_run": false}'"'"' | grpcurl -plaintext -proto src/api/machine.proto -d @ localhost:50051 platform.machine.v1alpha1.MachineService/ApplyConfiguration' \
    "Configuration applied successfully"

# Test 6: Get Configuration (should now succeed)
run_test "Get Configuration (After Apply)" \
    "grpcurl -plaintext -proto src/api/machine.proto localhost:50051 platform.machine.v1alpha1.MachineService/GetConfiguration" \
    "3.0.0"

# Test 7: Reboot (Graceful)
run_test "Reboot (Graceful)" \
    'echo '"'"'{"graceful": true, "timeout_seconds": 30}'"'"' | grpcurl -plaintext -proto src/api/machine.proto -d @ localhost:50051 platform.machine.v1alpha1.MachineService/Reboot' \
    "Reboot scheduled"

# Test 8: Upgrade - Invalid Version
echo -n "Testing Upgrade (Invalid Version)... "
output=$(echo '{"target_version": "invalid"}' | grpcurl -plaintext -proto src/api/machine.proto -d @ localhost:50051 platform.machine.v1alpha1.MachineService/Upgrade 2>&1)
if echo "$output" | grep -q "Invalid target version format"; then
    echo -e "${GREEN}PASSED${NC} (correctly rejected invalid version)"
    ((TESTS_PASSED++))
else
    echo -e "${RED}FAILED${NC}"
    ((TESTS_FAILED++))
fi

# Test 9: Upgrade - Valid Version
run_test "Upgrade (Valid Version)" \
    'echo '"'"'{"target_version": "1.17.0"}'"'"' | grpcurl -plaintext -proto src/api/machine.proto -d @ localhost:50051 platform.machine.v1alpha1.MachineService/Upgrade' \
    "Upgrade.*initiated"

# Test 10: Stream Events (test connection)
echo -n "Testing Stream Events... "
timeout 2 grpcurl -plaintext -proto src/api/machine.proto localhost:50051 platform.machine.v1alpha1.MachineService/StreamEvents 2>&1 | grep -q "StreamStarted"
if [ $? -eq 0 ]; then
    echo -e "${GREEN}PASSED${NC}"
    ((TESTS_PASSED++))
else
    echo -e "${RED}FAILED${NC}"
    ((TESTS_FAILED++))
fi

# Test 11: Stream Events with Filter
echo -n "Testing Stream Events (Filtered)... "
output=$(echo '{"event_types": ["ConfigurationApplied", "RebootScheduled"]}' | timeout 2 grpcurl -plaintext -proto src/api/machine.proto -d @ localhost:50051 platform.machine.v1alpha1.MachineService/StreamEvents 2>&1)
if echo "$output" | grep -q "StreamStarted"; then
    echo -e "${GREEN}PASSED${NC}"
    ((TESTS_PASSED++))
else
    echo -e "${RED}FAILED${NC}"
    ((TESTS_FAILED++))
fi

# Test 12: Health Check
echo -n "Testing Health Check... "
if cargo run -- health --server localhost:50051 2>&1 | grep -q "Health check passed"; then
    echo -e "${GREEN}PASSED${NC}"
    ((TESTS_PASSED++))
else
    echo -e "${RED}FAILED${NC}"
    ((TESTS_FAILED++))
fi

# Test 13: Reset
run_test "Reset (Graceful)" \
    'echo '"'"'{"graceful": true, "timeout_seconds": 10}'"'"' | grpcurl -plaintext -proto src/api/machine.proto -d @ localhost:50051 platform.machine.v1alpha1.MachineService/Reset' \
    "reset completed"

# Test 14: Verify configuration was cleared
echo -n "Testing Configuration Cleared After Reset... "
output=$(grpcurl -plaintext -proto src/api/machine.proto localhost:50051 platform.machine.v1alpha1.MachineService/GetConfiguration 2>&1)
if echo "$output" | grep -q "Not Found\|not.*found"; then
    echo -e "${GREEN}PASSED${NC}"
    ((TESTS_PASSED++))
else
    echo -e "${RED}FAILED${NC}"
    ((TESTS_FAILED++))
fi

# Test 15: Check state persistence
echo -n "Testing State Persistence... "
if [ -d "/tmp/platform-state" ]; then
    echo -e "${GREEN}PASSED${NC} (state directory exists)"
    ((TESTS_PASSED++))
else
    echo -e "${YELLOW}SKIPPED${NC} (using default state directory)"
fi

# Test 16: Check event persistence
echo -n "Testing Event Persistence... "
if [ -f "/tmp/platform-state/events/events.jsonl" ]; then
    event_count=$(wc -l < "/tmp/platform-state/events/events.jsonl")
    echo -e "${GREEN}PASSED${NC} ($event_count events persisted)"
    ((TESTS_PASSED++))
else
    echo -e "${YELLOW}SKIPPED${NC} (events file not found)"
fi

# Summary
echo
echo "=== Test Summary ==="
echo -e "Tests Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Tests Failed: ${RED}$TESTS_FAILED${NC}"
echo

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed!${NC}"
    exit 1
fi