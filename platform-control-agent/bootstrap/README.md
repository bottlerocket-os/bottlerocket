# Platform Bootstrap Components

This module implements Phase 2 of the Platform API: autonomous cluster bootstrapping for Bottlerocket nodes.

## Overview

The bootstrap module enables Bottlerocket nodes to:
1. **Elect a Leader**: Deterministic leader election using modified Raft consensus
2. **Generate PKI**: FIPS-compliant certificate authority and distribution
3. **Form etcd Cluster**: Automated etcd cluster initialization

## Architecture

```
bootstrap/
├── src/
│   ├── election/      # Leader election implementation
│   │   ├── state.rs   # Election state machine
│   │   ├── service.rs # gRPC service implementation
│   │   └── algorithm.rs # Raft consensus logic
│   ├── pki/          # Certificate management
│   │   ├── ca.rs     # Certificate authority
│   │   ├── service.rs # gRPC service
│   │   └── store.rs  # Certificate storage
│   ├── etcd/         # etcd cluster formation
│   │   ├── config.rs # Static pod generation
│   │   ├── service.rs # gRPC service
│   │   └── client.rs # etcd client wrapper
│   └── coordinator/  # Bootstrap orchestration
│       └── mod.rs    # Coordinates all phases
├── proto/            # Protocol buffer definitions
└── tests/           # Integration tests
```

## Development Status

### Phase 2.1: Leader Election (Weeks 5-6)
- [ ] Core Raft implementation
- [ ] Priority-based voting
- [ ] Pre-vote optimization
- [ ] Network partition handling
- [ ] gRPC service endpoints
- [ ] Comprehensive testing

### Phase 2.2: PKI System (Weeks 6-7)
- [ ] FIPS-compliant CA generation
- [ ] Certificate hierarchy creation
- [ ] Distribution mechanism
- [ ] Automatic renewal
- [ ] Revocation support
- [ ] Security hardening

### Phase 2.3: etcd Formation (Weeks 7-8)
- [ ] Static pod generation
- [ ] Initial cluster bootstrap
- [ ] Member management
- [ ] Health monitoring
- [ ] Backup/restore
- [ ] Disaster recovery

## Quick Start

```bash
# Build the bootstrap module
cargo build -p platform-bootstrap

# Run tests
cargo test -p platform-bootstrap

# Run with mock nodes (development)
cargo run -p platform-bootstrap -- --dev-mode

# Generate certificates for testing
./scripts/generate-test-pki.sh
```

## Testing

### Unit Tests
```bash
# Election tests
cargo test -p platform-bootstrap election::

# PKI tests
cargo test -p platform-bootstrap pki::

# etcd tests
cargo test -p platform-bootstrap etcd::
```

### Integration Tests
```bash
# Three-node cluster bootstrap
./test/integration/bootstrap_three_nodes.sh

# Network partition scenarios
./test/integration/test_partitions.sh

# PKI distribution test
./test/integration/test_pki_distribution.sh
```

## Configuration

Bootstrap behavior is configured through the MachineConfig:

```yaml
apiVersion: platform.io/v1alpha1
kind: MachineConfig
metadata:
  name: control-plane
spec:
  role: control-plane
  bootstrap:
    enabled: true
    electionConfig:
      priority: 100
      timeouts:
        election: 30s
        heartbeat: 3s
    pkiConfig:
      keyAlgorithm: RSA-4096
      organization: "My Organization"
    etcdConfig:
      version: "3.5.12"
      dataDir: "/var/lib/etcd"
```

## Security

All bootstrap operations require:
- mTLS authentication between nodes
- Control plane role authorization
- Signed election messages
- Encrypted certificate distribution
- Audit logging of all operations

## Monitoring

Key metrics exposed:
- `platform_election_state` - Current election state
- `platform_election_term` - Current term number
- `platform_pki_certificates_total` - Certificates issued
- `platform_etcd_cluster_size` - Number of etcd members

## Dependencies

- Platform Control Agent v1.0.0
- etcd v3.5+ with FIPS support
- OpenSSL 3.0+ (FIPS module)

## Next Steps

After completing Phase 2, the platform will support:
1. Zero-touch cluster formation
2. Automatic PKI management
3. Self-healing etcd clusters
4. Foundation for Phase 3: Configuration Management