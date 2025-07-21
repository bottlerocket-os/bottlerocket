# Feature: Platform Control Agent

## Overview
The Platform Control Agent is a privileged host container that provides API-driven management capabilities for Bottlerocket nodes, enabling Talos-like operations while maintaining FIPS compliance.

## Status
- **Feature Status**: 🟡 In Development
- **Target Release**: v1.0.0
- **Feature Owner**: Platform Team
- **Last Updated**: 2025-01-21
- **Implementation Progress**: Basic scaffolding complete, critical blockers identified

## Summary
The Platform Control Agent transforms Bottlerocket from a traditional SSH-managed OS into a fully API-driven platform. It runs as a superpowered host container with access to the Bottlerocket Settings API and exposes a gRPC interface for external management.

## Current Implementation Status

### ✅ Completed
- [x] Basic gRPC server scaffolding with protobuf definitions
- [x] Docker-based development environment with mock Bottlerocket API
- [x] Makefile and build infrastructure
- [x] Basic monitoring setup (Prometheus/Grafana)
- [x] Development certificate generation scripts

### 🚧 In Progress
- [ ] Unix socket client for Bottlerocket API (Critical Blocker)
- [ ] mTLS implementation for production security (Critical Blocker)

### ❌ Not Started
- [ ] State persistence and recovery
- [ ] Core gRPC method implementations (get_status, reset, upgrade)
- [ ] Configuration reconciliation loop
- [ ] Comprehensive test coverage
- [ ] Production-ready FIPS container

## Implementation Roadmap

### Phase 1: Critical Production Blockers (Week 1-2)

#### 1.1 Unix Socket Client Implementation
**Priority**: 🔴 Critical  
**Status**: 🚧 In Progress  
**Blocker**: Current `reqwest` client doesn't support Unix domain sockets
- [ ] Replace with `hyperlocal` or custom `hyper` connector
- [ ] Support both HTTP (dev) and Unix socket (prod) modes
- [ ] Add connection retry logic with exponential backoff
- [ ] Implement proper error handling for socket errors

#### 1.2 mTLS Security Implementation  
**Priority**: 🔴 Critical  
**Status**: ❌ Not Started  
**Blocker**: gRPC server has no TLS configuration
- [ ] Implement `ServerTlsConfig` in main.rs
- [ ] Add client certificate validation
- [ ] Support dev (self-signed) and prod (CA-signed) modes
- [ ] Implement certificate rotation mechanism
- [ ] Add TLS configuration to Dockerfile

### Phase 2: Core Functionality (Week 2-3)

#### 2.1 State Persistence
**Priority**: 🟡 High  
**Status**: ❌ Not Started
- [ ] Implement config serialization to `/var/lib/platform/config.json`
- [ ] Add startup recovery logic
- [ ] Handle config version migrations
- [ ] Implement atomic write operations
- [ ] Add backup/restore functionality

#### 2.2 Core gRPC Methods
**Priority**: 🟡 High  
**Status**: ❌ Not Started
- [ ] **get_status**: Query real system metrics from /proc and Bottlerocket API
- [ ] **reset**: Implement safe node reset with pre-flight checks
- [ ] **upgrade**: Integrate with Bottlerocket update system
- [ ] **stream_events**: Add event streaming with backpressure handling
- [ ] **apply_configuration**: Add validation and rollback capability

#### 2.3 Configuration Translation
**Priority**: 🟡 High  
**Status**: ❌ Not Started
- [ ] Implement comprehensive MachineConfig to Bottlerocket settings translator
- [ ] Add validation for all configuration fields
- [ ] Support incremental updates (patch semantics)
- [ ] Handle configuration conflicts gracefully
- [ ] Add dry-run mode for testing

### Phase 3: Reliability & Testing (Week 3-4)

#### 3.1 Reconciliation Loop
**Priority**: 🟠 Medium  
**Status**: ❌ Not Started
- [ ] Implement background reconciliation task
- [ ] Add drift detection logic
- [ ] Create event emission for drift events
- [ ] Add configurable reconciliation interval
- [ ] Implement circuit breaker for failed reconciliations

#### 3.2 Comprehensive Testing
**Priority**: 🟠 Medium  
**Status**: ❌ Not Started
- [ ] Unit tests for all service methods (target: >80% coverage)
- [ ] Integration tests using docker-compose environment
- [ ] Mock Bottlerocket API response fixtures
- [ ] Error injection and chaos testing
- [ ] Performance benchmarks for API operations

#### 3.3 Production Hardening
**Priority**: 🟠 Medium  
**Status**: ❌ Not Started
- [ ] Implement rate limiting for API calls
- [ ] Add request/response logging with privacy controls
- [ ] Implement graceful shutdown handling
- [ ] Add resource limits and health checks
- [ ] Create operational runbooks

### Quick Wins (Can implement immediately)

#### Error Handling Improvements
**Priority**: 🟢 Low  
**Effort**: 1-2 hours
- [ ] Map internal errors to specific gRPC status codes
- [ ] Add structured error logging with context
- [ ] Improve client-facing error messages
- [ ] Add error metrics for monitoring

#### Health Check Implementation
**Priority**: 🟢 Low  
**Effort**: 1-2 hours
- [ ] Implement real health check logic
- [ ] Check gRPC server responsiveness
- [ ] Verify Bottlerocket API connectivity
- [ ] Add readiness vs liveness distinction

#### Basic Config Persistence
**Priority**: 🟢 Low  
**Effort**: 2-3 hours
- [ ] Save config to disk on apply
- [ ] Load config on startup
- [ ] Add config validation on load
- [ ] Implement config backup rotation

## Exit Criteria (Updated with Progress)

### 1. Core API Implementation
**Issue**: [#001](https://github.com/org/repo/issues/001)
- [x] gRPC server implementation scaffolding
- [ ] mTLS implementation (🚧 Blocked on TLS config)
- [x] Machine configuration API structure
- [ ] Status reporting with real data
- [ ] Health check endpoints (currently stubbed)
- [x] Protobuf definitions

### 2. Bottlerocket Integration
**Issue**: [#002](https://github.com/org/repo/issues/002)
- [ ] Settings API client implementation (🚧 Blocked on Unix socket)
- [ ] Configuration translation layer (partial implementation)
- [x] Host container packaging (Dockerfile exists)
- [x] Volume mount configuration
- [ ] Systemd unit integration

### 3. Security & Compliance
**Issue**: [#003](https://github.com/org/repo/issues/003)
- [ ] FIPS-compliant container build
- [ ] mTLS certificate management (🚧 In Progress)
- [ ] RBAC implementation
- [ ] Audit logging
- [ ] Security scanning in CI

### 4. Testing & Validation
**Issue**: [#004](https://github.com/org/repo/issues/004)
- [ ] Unit tests (currently 0% coverage)
- [ ] Integration tests with Bottlerocket
- [ ] E2E tests for configuration scenarios
- [ ] Performance benchmarks
- [ ] Chaos testing scenarios

### 5. Documentation & Tooling
**Issue**: [#005](https://github.com/org/repo/issues/005)
- [x] Basic API documentation (in proto files)
- [x] Development setup guide (README.md)
- [ ] CLI tool (platformctl)
- [ ] Production deployment guide
- [ ] Troubleshooting guide
- [ ] Migration guide from SSH

## Technical Design

### API Surface
```protobuf
service PlatformControl {
  rpc ApplyConfiguration(ConfigRequest) returns (ConfigResponse);
  rpc GetConfiguration(Empty) returns (MachineConfig);
  rpc GetStatus(Empty) returns (MachineStatus);
  rpc Reset(ResetRequest) returns (ResetResponse);
  rpc Reboot(RebootRequest) returns (RebootResponse);
}
```

### Container Configuration
```toml
[settings.host-containers.platform-control]
enabled = true
source = "registry.io/platform-control:v1.0.0-fips"
superpowered = true

[settings.host-containers.platform-control.mounts]
api-socket = { source = "/run/api.sock", destination = "/run/api.sock" }
certs = { source = "/etc/platform/certs", destination = "/etc/platform/certs" }
```

## Dependencies
- Bottlerocket Settings API v2
- gRPC-go with FIPS support
- Container build pipeline with FIPS validation

## Known Issues & Technical Debt

### Critical Blockers
1. **Unix Socket Support**: The current `reqwest` HTTP client cannot connect to Bottlerocket's Unix domain socket API (`unix:///run/api.sock`)
   - **Impact**: Agent cannot communicate with Bottlerocket in production
   - **Solution**: Migrate to `hyperlocal` or custom `hyper` connector

2. **Missing TLS Implementation**: The gRPC server runs without TLS, violating security requirements
   - **Impact**: Cannot deploy to production environments
   - **Solution**: Implement mTLS with proper certificate management

### Technical Debt
1. **Hardcoded Mock Data**: All status responses return static values
2. **No State Persistence**: Configuration lost on container restart  
3. **Missing Error Handling**: Many error paths return generic internal errors
4. **No Test Coverage**: 0% test coverage for critical business logic
5. **Incomplete Translation Logic**: Config to Bottlerocket settings mapping is partial

## Risks & Mitigations
| Risk | Impact | Mitigation |
|------|--------|------------|
| Settings API limitations | High | Early testing, upstream patches |
| Container privilege escalation | High | SELinux policies, minimal attack surface |
| Certificate management complexity | Medium | Automated rotation, monitoring |
| Unix socket connectivity | Critical | Use proven libraries, extensive testing |
| State loss on restart | High | Implement persistent storage immediately |

## Development Guidelines

### Local Development
```bash
# Start development environment
make dev

# Run with mock Bottlerocket API
docker-compose up

# Test with grpcurl
grpcurl -plaintext localhost:50051 platform.machine.v1alpha1.MachineService/GetStatus

# Generate certificates for testing
./scripts/generate-dev-certs.sh
```

### Code Structure
```
platform-control-agent/
├── src/
│   ├── api/          # gRPC API definitions
│   ├── bottlerocket/ # Bottlerocket client (needs Unix socket support)
│   ├── services/     # gRPC service implementations
│   └── main.rs       # Server setup (needs TLS)
├── mock-bottlerocket/ # Mock API for development
└── monitoring/        # Prometheus/Grafana setup
```

### Testing Strategy
1. **Unit Tests**: Mock Bottlerocket client, test service logic
2. **Integration Tests**: Use docker-compose environment
3. **E2E Tests**: Deploy to real Bottlerocket nodes
4. **Chaos Tests**: Network failures, API timeouts, state corruption

## Success Metrics
- API response time < 100ms (p99)
- Zero CVEs in container dependencies
- 99.99% uptime for API availability
- Configuration convergence < 30s

## Related Features
- [Cluster Bootstrap](./cluster-bootstrap.md)
- [Machine Configuration](./machine-configuration.md)
- [Update Orchestration](./update-orchestration.md)