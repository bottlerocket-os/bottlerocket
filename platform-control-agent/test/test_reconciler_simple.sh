#!/bin/bash
# Simple reconciliation test without grpcurl

set -euo pipefail

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Testing Reconciliation Loop...${NC}"

# Test 1: Reconciliation enabled
echo -e "\n${YELLOW}Test 1: Reconciliation Enabled${NC}"
export SKIP_UNIX_SOCKET=true
export PLATFORM_STATE_DIR="./test_state"
export RECONCILER_ENABLED=true
export RECONCILER_INTERVAL=30
export RUST_LOG=platform_control=debug
mkdir -p ./test_state

echo "Starting server with reconciliation enabled..."
timeout 35 cargo run -- serve --dev-mode 2>&1 | tee test1.log &
PID=$!

# Wait for reconciliation to run at least once
sleep 35

# Check for reconciliation logs
if grep -q "Starting reconciliation loop" test1.log && \
   grep -q "ReconciliationStarted" test1.log && \
   grep -q "Starting reconciliation check" test1.log; then
    echo -e "${GREEN}✓ Reconciliation loop is running${NC}"
else
    echo -e "${RED}✗ Reconciliation loop not detected${NC}"
fi

# Count reconciliation checks (should be at least 2 in 35 seconds with 30s interval)
CHECK_COUNT=$(grep -c "Starting reconciliation check" test1.log || echo 0)
if [ $CHECK_COUNT -ge 2 ]; then
    echo -e "${GREEN}✓ Reconciliation ran $CHECK_COUNT times${NC}"
else
    echo -e "${RED}✗ Reconciliation only ran $CHECK_COUNT times (expected at least 2)${NC}"
fi

# Test 2: Reconciliation disabled
echo -e "\n${YELLOW}Test 2: Reconciliation Disabled${NC}"
export RECONCILER_ENABLED=false
rm -rf ./test_state
mkdir -p ./test_state

echo "Starting server with reconciliation disabled..."
timeout 35 cargo run -- serve --dev-mode 2>&1 | tee test2.log &
PID=$!

# Wait
sleep 35

# Check that reconciliation is disabled
if grep -q "Reconciliation loop is disabled" test2.log && \
   ! grep -q "Starting reconciliation check" test2.log; then
    echo -e "${GREEN}✓ Reconciliation is properly disabled${NC}"
else
    echo -e "${RED}✗ Reconciliation not properly disabled${NC}"
fi

# Test 3: Custom interval
echo -e "\n${YELLOW}Test 3: Custom Interval (60s)${NC}"
export RECONCILER_ENABLED=true
export RECONCILER_INTERVAL=60
rm -rf ./test_state
mkdir -p ./test_state

echo "Starting server with 60s reconciliation interval..."
timeout 65 cargo run -- serve --dev-mode 2>&1 | tee test3.log &
PID=$!

# Wait for just over one interval
sleep 65

# Should see exactly 2 checks (initial + one after 60s)
CHECK_COUNT=$(grep -c "Starting reconciliation check" test3.log || echo 0)
if [ $CHECK_COUNT -eq 2 ]; then
    echo -e "${GREEN}✓ Reconciliation interval working correctly (2 checks in 65s with 60s interval)${NC}"
else
    echo -e "${RED}✗ Unexpected number of checks: $CHECK_COUNT (expected 2)${NC}"
fi

# Cleanup
echo -e "\n${YELLOW}Cleaning up...${NC}"
rm -rf ./test_state test1.log test2.log test3.log

echo -e "\n${GREEN}Reconciliation tests completed!${NC}"