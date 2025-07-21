#!/bin/bash

# Test state persistence functionality

echo "=== State Persistence Test ==="
echo

STATE_DIR=${PLATFORM_STATE_DIR:-/tmp/platform-state}
echo "Using state directory: $STATE_DIR"
echo

# Test 1: Apply configuration and verify it's saved
echo "1. Applying configuration..."
cat <<EOF | grpcurl -plaintext -proto src/api/machine.proto -d @ localhost:50051 platform.machine.v1alpha1.MachineService/ApplyConfiguration
{
  "config": {
    "version": "1.0.0-persistence-test",
    "type": 1,
    "network": {
      "hostname": "persistence-test-node"
    }
  }
}
EOF

echo "2. Checking if configuration was persisted..."
if [ -f "$STATE_DIR/config.json" ]; then
    echo "✓ Config file exists"
    echo "Content:"
    jq . "$STATE_DIR/config.json" | head -20
else
    echo "✗ Config file not found!"
    exit 1
fi

echo
echo "3. Getting configuration via API..."
grpcurl -plaintext -proto src/api/machine.proto localhost:50051 platform.machine.v1alpha1.MachineService/GetConfiguration | jq .

echo
echo "4. Testing reset clears configuration..."
echo '{"graceful": false}' | grpcurl -plaintext -proto src/api/machine.proto -d @ localhost:50051 platform.machine.v1alpha1.MachineService/Reset

if [ -f "$STATE_DIR/config.json" ]; then
    echo "✗ Config file still exists after reset!"
else
    echo "✓ Config file properly removed"
fi

echo
echo "5. Checking event persistence..."
if [ -f "$STATE_DIR/events/events.jsonl" ]; then
    EVENT_COUNT=$(wc -l < "$STATE_DIR/events/events.jsonl")
    echo "✓ Events file exists with $EVENT_COUNT events"
    echo "Last 5 events:"
    tail -5 "$STATE_DIR/events/events.jsonl" | jq -c '{id, event_type, timestamp}'
else
    echo "✗ Events file not found!"
fi

echo
echo "State persistence tests complete!"