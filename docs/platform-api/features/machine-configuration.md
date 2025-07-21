# Feature: Machine Configuration

## Overview
Declarative machine configuration system that translates high-level platform configurations into Bottlerocket-specific settings, enabling GitOps workflows and configuration management.

## Status
- **Feature Status**: 🟡 In Design
- **Target Release**: v1.0.0
- **Feature Owner**: Platform Team
- **Last Updated**: 2025-01-20

## Summary
The Machine Configuration feature provides a declarative API for configuring Bottlerocket nodes. It abstracts Bottlerocket's settings into a cloud-native configuration format that supports validation, templating, and GitOps workflows.

## Exit Criteria

### 1. Configuration Schema
**Issue**: [#020](https://github.com/org/repo/issues/020)
- [ ] CRD definitions for MachineConfig
- [ ] OpenAPI schema validation
- [ ] Configuration versioning
- [ ] Backwards compatibility
- [ ] Schema documentation

### 2. Translation Engine
**Issue**: [#021](https://github.com/org/repo/issues/021)
- [ ] Platform config to Bottlerocket settings
- [ ] Network configuration mapping
- [ ] Security settings translation
- [ ] Kernel parameter management
- [ ] Validation and error handling

### 3. Templating System
**Issue**: [#022](https://github.com/org/repo/issues/022)
- [ ] Go template support
- [ ] Variable substitution
- [ ] Environment-specific overrides
- [ ] Secret reference resolution
- [ ] Template validation

### 4. GitOps Integration
**Issue**: [#023](https://github.com/org/repo/issues/023)
- [ ] Flux controller support
- [ ] ArgoCD integration
- [ ] Drift detection
- [ ] Reconciliation loop
- [ ] Status reporting

### 5. Compliance Profiles
**Issue**: [#024](https://github.com/org/repo/issues/024)
- [ ] STIG baseline profile
- [ ] CIS benchmark profile
- [ ] FedRAMP high profile
- [ ] Custom profile support
- [ ] Compliance validation

## Technical Design

### Configuration Schema
```yaml
apiVersion: config.platform.io/v1alpha1
kind: MachineConfiguration
metadata:
  name: control-plane-config
  labels:
    role: control-plane
spec:
  # Machine role and cluster membership
  machine:
    type: controlplane
    cluster: production
    
  # Network configuration
  network:
    hostname: cp-node-1
    interfaces:
      - name: eth0
        dhcp4: true
        dhcp6: false
    dns:
      nameservers:
        - 10.0.0.53
        - 10.0.0.54
        
  # Kubernetes settings
  kubernetes:
    version: "1.28.5"
    cloudProvider: external
    clusterDNS: 10.96.0.10
    clusterDomain: cluster.local
    apiServer:
      extraArgs:
        audit-log-maxage: "30"
        audit-log-maxbackup: "10"
        
  # Security configuration
  security:
    selinux:
      mode: enforcing
    kernel:
      lockdown: integrity
      parameters:
        "kernel.yama.ptrace_scope": "2"
        "kernel.unprivileged_bpf_disabled": "1"
    fips:
      enabled: true
      
  # Host containers
  hostContainers:
    falco:
      source: registry.io/falco:0.35.1-fips
      enabled: true
      superpowered: true
    compliance-scanner:
      source: registry.io/openscap:latest-fips
      enabled: true
      
  # Storage configuration  
  storage:
    filesystems:
      - device: /dev/sdb
        path: /var/lib/containerd
        format: xfs
        options:
          - noatime
          - nodiratime
```

### Translation Map
```go
type TranslationRule struct {
    Source  string // JSONPath in MachineConfig
    Target  string // Bottlerocket settings path
    Transform func(interface{}) interface{}
}

var TranslationRules = []TranslationRule{
    {
        Source: "$.spec.network.hostname",
        Target: "settings.network.hostname",
    },
    {
        Source: "$.spec.kubernetes.version",
        Target: "settings.kubernetes.kubernetes-version",
        Transform: validateK8sVersion,
    },
    {
        Source: "$.spec.security.kernel.parameters",
        Target: "settings.kernel.sysctl",
        Transform: validateSysctls,
    },
}
```

## Dependencies
- Bottlerocket Settings API documentation
- Kubernetes API machinery
- GitOps tooling (Flux/ArgoCD)
- YAML/JSON schema validators

## Risks & Mitigations
| Risk | Impact | Mitigation |
|------|--------|------------|
| Invalid translations | High | Comprehensive validation, testing |
| Breaking changes | High | Versioning, deprecation policy |
| Performance impact | Medium | Caching, optimized reconciliation |
| Schema complexity | Medium | Clear documentation, examples |

## Implementation Phases

### Phase 1: Core Schema (Weeks 1-3)
- Define CRDs
- Implement validation
- Basic translation engine

### Phase 2: Advanced Features (Weeks 4-6)
- Templating system
- Profile support
- GitOps integration

### Phase 3: Production Hardening (Weeks 7-8)
- Performance optimization
- Comprehensive testing
- Documentation

## Success Metrics
- Configuration application < 30 seconds
- Zero invalid configurations in production
- 100% compliance profile coverage
- GitOps sync time < 1 minute

## Validation Requirements
- Schema validation on all inputs
- Bottlerocket settings compatibility check
- Security policy compliance validation
- Network configuration verification

## Related Features
- [Platform Control Agent](./platform-control-agent.md)
- [GitOps Integration](./gitops-integration.md)
- [Compliance Automation](./compliance-automation.md)