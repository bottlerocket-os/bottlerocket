# Platform Control Agent Tests

This directory contains test scripts for the Platform Control Agent.

## Test Scripts

### integration_test.sh
Comprehensive test suite that validates all gRPC methods:
- Get Status
- Apply/Get Configuration
- Reboot
- Reset  
- Upgrade
- Stream Events
- Health Check
- State Persistence

Run: `./test/integration_test.sh`

### test_events.sh
Tests the event streaming functionality:
- Basic event streaming
- Filtered event streams
- Event generation from various operations

Run: `./test/test_events.sh`

### test_persistence.sh
Tests state persistence:
- Configuration persistence across restarts
- Reset clears persisted state
- Event persistence to disk

Run: `./test/test_persistence.sh`

### test_mtls.sh
Tests mTLS (mutual TLS) support:
- Generates test certificates
- Shows how to start server with TLS
- Tests client certificate authentication

Run: `./test/test_mtls.sh`

### test_unix_socket.sh
Tests Unix socket client functionality:
- Creates a mock Bottlerocket API server
- Tests communication over Unix sockets
- Useful for development without real Bottlerocket

Run: `./test/test_unix_socket.sh`

## Running Tests

1. Start the platform-control-agent:
```bash
SKIP_UNIX_SOCKET=1 PLATFORM_STATE_DIR=/tmp/platform-state cargo run -- serve --dev-mode -b 0.0.0.0:50051
```

2. Run individual test scripts:
```bash
chmod +x test/*.sh
./test/integration_test.sh
```

## Test Environment Variables

- `SKIP_UNIX_SOCKET=1` - Skip real Unix socket calls (for development)
- `PLATFORM_STATE_DIR=/tmp/platform-state` - Custom state directory
- `RUST_LOG=debug` - Enable debug logging

## Prerequisites

- grpcurl installed (`brew install grpcurl`)
- jq installed (`brew install jq`)
- Python 3 (for mock Unix socket)
- OpenSSL (for certificate generation)