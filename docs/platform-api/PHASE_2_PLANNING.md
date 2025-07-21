# Phase 2: Cluster Formation Planning

## Overview
Phase 2 focuses on implementing autonomous cluster bootstrapping capabilities, enabling Bottlerocket nodes to form Kubernetes clusters without manual intervention.

## Timeline
**Duration**: Weeks 5-8 (4 weeks)  
**Start Date**: TBD  
**Target Completion**: TBD

## Key Deliverables

### 1. Leader Election System (Week 5-6)
**Goal**: Implement deterministic leader election for bootstrap coordination

#### Requirements
- **Algorithm**: Implement Raft-based consensus or similar
- **Features**:
  - Deterministic election based on node attributes
  - Split-brain prevention mechanisms
  - Network partition tolerance
  - Election status visibility via API
  
#### Technical Approach
- Use etcd's election API or implement custom using distributed locks
- Consider using HashiCorp's memberlist or similar gossip protocol
- Implement health checks and automatic re-election
- Add observability through events and metrics

#### API Endpoints
```protobuf
service ElectionService {
  rpc GetLeader(GetLeaderRequest) returns (GetLeaderResponse);
  rpc Campaign(CampaignRequest) returns (CampaignResponse);
  rpc Resign(ResignRequest) returns (ResignResponse);
  rpc ObserveElection(ObserveRequest) returns (stream ElectionEvent);
}
```

### 2. PKI Generation & Distribution (Week 6-7)
**Goal**: FIPS-compliant certificate authority and distribution system

#### Requirements
- **CA Generation**:
  - FIPS 140-3 compliant algorithms
  - Root CA with intermediate CAs
  - Certificate lifecycle management
  - Secure key storage
  
- **Distribution**:
  - Secure certificate distribution to nodes
  - Automatic certificate renewal
  - Certificate revocation support
  - Backup and disaster recovery

#### Technical Approach
- Use OpenSSL FIPS module for CA operations
- Implement certificate distribution via gRPC with mTLS
- Store certificates in encrypted format
- Use Kubernetes CSR API for node certificates

#### Components
1. **CA Service**: Manages root and intermediate CAs
2. **Certificate Controller**: Handles certificate lifecycle
3. **Distribution Service**: Secure cert distribution
4. **Backup Service**: Certificate backup/restore

### 3. etcd Cluster Formation (Week 7-8)
**Goal**: Automated etcd cluster bootstrapping

#### Requirements
- **Cluster Formation**:
  - Static pod generation for etcd
  - Initial cluster bootstrapping
  - Member addition/removal
  - Disaster recovery procedures
  
- **Security**:
  - FIPS-compliant TLS configuration
  - Client certificate authentication
  - Encrypted data at rest
  - Network policies

#### Technical Approach
- Generate etcd static pod manifests
- Use etcd's cluster formation APIs
- Implement health monitoring and auto-recovery
- Create backup/restore procedures

#### Configuration
```yaml
etcd:
  version: "3.5.x"
  dataDir: "/var/lib/etcd"
  tlsConfig:
    serverCert: "/etc/etcd/tls/server.crt"
    serverKey: "/etc/etcd/tls/server.key"
    clientCA: "/etc/etcd/tls/ca.crt"
  initialCluster:
    size: 3
    token: "bootstrap-token"
```

## Integration Points

### Platform Control Agent Extensions
1. **Election API Integration**
   - Add election service to existing gRPC server
   - Extend event system for election events
   - Add election status to machine status

2. **Certificate Management**
   - Integrate PKI with existing mTLS setup
   - Add certificate endpoints to API
   - Extend reconciliation for cert renewal

3. **etcd Management**
   - Add etcd configuration to MachineConfig
   - Implement etcd health checks
   - Add etcd metrics collection

### New Proto Definitions
```protobuf
// election.proto
message ElectionState {
  string leader_id = 1;
  int64 term = 2;
  repeated string candidates = 3;
  google.protobuf.Timestamp last_heartbeat = 4;
}

// pki.proto
message Certificate {
  string common_name = 1;
  bytes cert_data = 2;
  google.protobuf.Timestamp not_before = 3;
  google.protobuf.Timestamp not_after = 4;
  CertificateType type = 5;
}

// etcd.proto
message EtcdConfig {
  repeated string initial_cluster = 1;
  string cluster_token = 2;
  TLSConfig tls = 3;
  map<string, string> extra_args = 4;
}
```

## Testing Strategy

### Unit Tests
- Election algorithm correctness
- PKI generation and validation
- etcd configuration generation

### Integration Tests
- Multi-node election scenarios
- Certificate distribution flow
- etcd cluster formation

### Chaos Tests
- Network partition during election
- Certificate expiration handling
- etcd member failure

### Performance Tests
- Election convergence time
- Certificate generation throughput
- etcd cluster formation time

## Security Considerations

1. **Election Security**
   - Prevent unauthorized participation
   - Secure leader communication
   - Audit all election events

2. **PKI Security**
   - Hardware security module (HSM) support
   - Key escrow mechanisms
   - Certificate transparency logs

3. **etcd Security**
   - Encrypted communication
   - RBAC policies
   - Regular security audits

## Dependencies

### External Libraries
- etcd client library (v3.5+)
- OpenSSL FIPS module
- Raft consensus library (if custom implementation)

### Internal Dependencies
- Platform Control Agent (Phase 1)
- Event system
- Health check framework

## Risk Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| Split-brain in election | High | Implement proper quorum and fencing |
| Certificate compromise | Critical | HSM support, short-lived certs |
| etcd data loss | High | Automated backups, multi-region |
| Network partitions | Medium | Partition tolerance in design |

## Success Criteria

1. **Leader Election**
   - Election completes in < 30 seconds
   - Handles 3-node partition scenarios
   - Zero split-brain incidents

2. **PKI System**
   - FIPS 140-3 validated
   - Certificate issuance < 5 seconds
   - Automatic renewal 30 days before expiry

3. **etcd Formation**
   - 3-node cluster in < 2 minutes
   - Automatic disaster recovery
   - 99.99% availability

## Next Steps

1. **Design Reviews**
   - [ ] Election algorithm design review
   - [ ] PKI architecture review
   - [ ] etcd integration review

2. **Prototype Development**
   - [ ] Leader election prototype
   - [ ] PKI proof of concept
   - [ ] etcd formation test

3. **Documentation**
   - [ ] API documentation
   - [ ] Operations guide
   - [ ] Troubleshooting runbook