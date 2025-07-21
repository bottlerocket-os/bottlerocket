# Feature: Platform Control Agent

## Overview
The Platform Control Agent is a privileged host container that provides API-driven management capabilities for Bottlerocket nodes, enabling Talos-like operations while maintaining FIPS compliance.

## Status
- **Feature Status**: 🟡 In Design
- **Target Release**: v1.0.0
- **Feature Owner**: Platform Team
- **Last Updated**: 2025-01-20

## Summary
The Platform Control Agent transforms Bottlerocket from a traditional SSH-managed OS into a fully API-driven platform. It runs as a superpowered host container with access to the Bottlerocket Settings API and exposes a gRPC interface for external management.

## Exit Criteria

### 1. Core API Implementation
**Issue**: [#001](https://github.com/org/repo/issues/001)
- [ ] gRPC server implementation with mTLS
- [ ] Machine configuration API (apply, get, reset)
- [ ] Status reporting API
- [ ] Health check endpoints
- [ ] OpenAPI/protobuf documentation

### 2. Bottlerocket Integration
**Issue**: [#002](https://github.com/org/repo/issues/002)
- [ ] Settings API client implementation
- [ ] Configuration translation layer
- [ ] Host container packaging
- [ ] Volume mount configuration
- [ ] Systemd unit integration

### 3. Security & Compliance
**Issue**: [#003](https://github.com/org/repo/issues/003)
- [ ] FIPS-compliant container build
- [ ] mTLS certificate management
- [ ] RBAC implementation
- [ ] Audit logging
- [ ] Security scanning in CI

### 4. Testing & Validation
**Issue**: [#004](https://github.com/org/repo/issues/004)
- [ ] Unit tests (>80% coverage)
- [ ] Integration tests with Bottlerocket
- [ ] E2E tests for configuration scenarios
- [ ] Performance benchmarks
- [ ] Chaos testing scenarios

### 5. Documentation & Tooling
**Issue**: [#005](https://github.com/org/repo/issues/005)
- [ ] API reference documentation
- [ ] Deployment guide
- [ ] CLI tool (platformctl)
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

## Risks & Mitigations
| Risk | Impact | Mitigation |
|------|--------|------------|
| Settings API limitations | High | Early testing, upstream patches |
| Container privilege escalation | High | SELinux policies, minimal attack surface |
| Certificate management complexity | Medium | Automated rotation, monitoring |

## Implementation Phases

### Phase 1: MVP (Weeks 1-4)
- Basic gRPC server
- Simple configuration application
- Manual testing

### Phase 2: Integration (Weeks 5-8)
- Full Bottlerocket API integration
- Automated testing
- Security hardening

### Phase 3: Production (Weeks 9-12)
- Performance optimization
- Monitoring integration
- Documentation completion

## Success Metrics
- API response time < 100ms (p99)
- Zero CVEs in container dependencies
- 99.99% uptime for API availability
- Configuration convergence < 30s

## Related Features
- [Cluster Bootstrap](./cluster-bootstrap.md)
- [Machine Configuration](./machine-configuration.md)
- [Update Orchestration](./update-orchestration.md)