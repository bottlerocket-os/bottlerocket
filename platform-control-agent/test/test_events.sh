#!/bin/bash

# Test event streaming functionality

echo "=== Event Streaming Test ==="
echo

# Test 1: Basic event streaming
echo "1. Starting basic event stream..."
grpcurl -plaintext -proto src/api/machine.proto localhost:50051 platform.machine.v1alpha1.MachineService/StreamEvents &
STREAM_PID=$!
sleep 2

echo "2. Triggering events..."
# Apply configuration
cat <<EOF | grpcurl -plaintext -proto src/api/machine.proto -d @ localhost:50051 platform.machine.v1alpha1.MachineService/ApplyConfiguration
{
  "config": {
    "version": "1.0.0-event-test",
    "type": 1,
    "cluster": {
      "name": "event-test-cluster",
      "endpoint": "https://k8s.example.com:6443",
      "ca_certificate": "test-cert",
      "bootstrap_token": "test-token",
      "dns_ip": "10.96.0.10",
      "dns_domain": "cluster.local"
    }
  }
}
EOF

# Schedule reboot
echo '{"graceful": true}' | grpcurl -plaintext -proto src/api/machine.proto -d @ localhost:50051 platform.machine.v1alpha1.MachineService/Reboot

sleep 3
kill $STREAM_PID 2>/dev/null
echo

# Test 2: Filtered event streaming
echo "3. Testing filtered event stream (only Reboot events)..."
echo '{"event_types": ["RebootScheduled", "RebootInitiated"]}' | grpcurl -plaintext -proto src/api/machine.proto -d @ localhost:50051 platform.machine.v1alpha1.MachineService/StreamEvents &
STREAM_PID=$!
sleep 2

# This should show up
echo '{"graceful": false}' | grpcurl -plaintext -proto src/api/machine.proto -d @ localhost:50051 platform.machine.v1alpha1.MachineService/Reboot

# This should NOT show up in the filtered stream
echo '{"graceful": true}' | grpcurl -plaintext -proto src/api/machine.proto -d @ localhost:50051 platform.machine.v1alpha1.MachineService/Reset

sleep 3
kill $STREAM_PID 2>/dev/null

echo
echo "Event streaming tests complete!"