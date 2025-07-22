# Platform Control Agent Implementation TODO

## Current Status

### Phase 1: Unix Socket Support ✅ COMPLETE

- [x] Unix socket server implementation
- [x] mTLS support for secure communication
- [x] State persistence with transactional updates
- [x] Core gRPC methods (Apply, Watch, Get, List)
- [x] Event notification system
- [x] Reconciliation loop
- [x] Status reporting

### Phase 2: Cluster Formation ⚠️ IN PROGRESS

#### Bootstrap Module Core Implementation ✅ COMPLETE

- [x] Raft consensus with priority-based voting (election/)
  - State machine implementation
  - Priority scoring algorithm (uptime, resources, network stability, user
    priority)
  - Pre-vote phase optimization
- [x] PKI certificate authority generation (pki/)
  - FIPS-compliant RSA 4096-bit keys
  - Certificate hierarchy (Root CA → Intermediate CAs)
  - Certificate store management
- [x] Proto definitions for all services
  - Election service (proto/election.proto)
  - PKI service (proto/pki.proto)
  - etcd service (proto/etcd.proto)
- [x] Basic coordinator structure (coordinator/)

#### Critical Bug Fixes ✅ COMPLETE

- [x] Fix 4 remaining async Send trait errors in election service
  - [x] Fix tokio::spawn for election_timer_task (line 72)
  - [x] Fix tokio::spawn for heartbeat_task (line 79)
  - [x] Fix campaign gRPC method Send compatibility (line 127)
  - [x] Ensure all error types in async contexts are Send + Sync

**Resolution**: Fixed by pre-formatting all error messages and removing tracing
macro interpolation in async contexts. All futures are now Send + Sync
compatible.

#### PKI Production Features - Dual-Mode Certificate Management ⚠️ CRITICAL

**Priority: HIGH** - Current implementation has critical production gaps

**Current Issues to Address:**

- No persistent certificate storage (lost on restart)
- No support for manual CA upload
- Incomplete certificate lifecycle management
- Rigid initialization preventing flexible deployment modes

**Implementation Timeline:**

### Phase 1: Foundational Persistence (CRITICAL PATH - 10-15 dev-days)

**Status: MUST BE COMPLETED FIRST** - All other PKI features are blocked by this

- [ ] **Core Infrastructure**
  - [ ] Implement persistent certificate storage system (SQLite recommended)
    - [ ] Design certificate database schema with atomic transactions
    - [ ] Add certificate CRUD operations
    - [ ] Implement certificate metadata tracking (creation, expiry, source)
  - [ ] Refactor PKIService initialization to read state from persistent store
  - [ ] Create certificate validation framework
    - [ ] Add certificate chain validation
    - [ ] Implement expiry checking and warnings
    - [ ] Add certificate format validation (PEM/DER)
  - [ ] Unit tests for storage layer

### Phase 2A: Manual Certificate Upload (HIGH PRIORITY - 8-12 dev-days)

**Status: Can begin in parallel after Phase 1 complete**

- [ ] **Mode 1: Manual Certificate Upload**

  - [ ] Extend bootstrap CLI with certificate management commands
    - [ ] `bootstrap pki upload-ca --cert-file <path> --key-file <path>`
    - [ ] `bootstrap pki status` - show current CA status and expiry
    - [ ] `bootstrap pki export-ca` - export public CA for distribution
  - [ ] Add gRPC endpoints for certificate operations
    - [ ] `UploadCertificateAuthority(cert, key, metadata)`
    - [ ] `GetCertificateStatus()` - validation and expiry info
    - [ ] `RevokeCertificate(cert_id)` - certificate revocation
  - [ ] Implement certificate validation pipeline
    - [ ] Verify CA certificate format and constraints
    - [ ] Validate private key matches public certificate
    - [ ] Check certificate is suitable for CA usage (basicConstraints)

### Phase 2B: Auto-Generated Self-Bootstrapping (MEDIUM PRIORITY - 5-8 dev-days)

**Status: Can begin in parallel after Phase 1 complete**

- [ ] **Mode 2: Auto-Generated Self-Bootstrapping**
  - [ ] Integrate crypto library for CA generation
  - [ ] Refactor `initialize_pki` to trigger and persist auto-generated CAs
    - [ ] Generate FIPS-compliant CA (RSA 4096, SHA256)
    - [ ] Set appropriate certificate extensions and constraints
    - [ ] Store generated CA with metadata (auto-generated flag)
  - [ ] Ensure mode detection logic is robust (block if manual CA exists)
  - [ ] Integration tests for auto-generation workflow

### Phase 3: Enhanced PKI Service Architecture (MEDIUM PRIORITY - 6-10 dev-days)

**Status: Begin after Phase 2A/2B complete**

- [ ] **Enhanced PKI Service Architecture**
  - [ ] Refactor PKIService for dual-mode support
    - [ ] Add mode detection and initialization logic
    - [ ] Implement certificate source tracking (manual vs auto)
    - [ ] Add configuration validation and error handling
  - [ ] Update data model for certificate management
    - [ ] Separate public certificates from private keys
    - [ ] Add certificate metadata (issuer, subject, validity)
    - [ ] Implement certificate chain storage
  - [ ] Add comprehensive error handling
    - [ ] Certificate validation errors with detailed messages
    - [ ] Graceful handling of certificate expiry
    - [ ] Proper error propagation to clients

### Phase 4: Certificate Lifecycle Management (DEFERRED - 10-15 dev-days)

**Status: Can be implemented after basic modes are production-ready**

- [ ] **Certificate Lifecycle Management**
  - [ ] Implement `renew_certificate` and `RevokeCertificate` gRPC endpoints
  - [ ] Add certificate lifecycle management
    - [ ] Implement renewal mechanism (75% lifetime threshold)
    - [ ] Add automatic certificate rotation
    - [ ] Implement graceful certificate rollover
  - [ ] Create certificate distribution system
    - [ ] Distribute CA certificates to cluster members
    - [ ] Handle certificate updates across cluster
    - [ ] Implement certificate synchronization on node join
  - [ ] End-to-end testing for certificate expiry and renewal scenarios

### PKI Testing Requirements (ONGOING - 5-8 dev-days total)

**Status: Should be implemented alongside each phase**

- [ ] **Testing Infrastructure**
  - [ ] Unit tests for certificate operations
    - [ ] Test certificate validation logic
    - [ ] Test certificate storage and retrieval
    - [ ] Test certificate lifecycle management
  - [ ] Integration tests for dual-mode scenarios
    - [ ] Test manual CA upload workflow
    - [ ] Test auto-generation and renewal
    - [ ] Test certificate distribution across cluster
  - [ ] End-to-end testing with real certificates
    - [ ] Test with production-like certificate chains
    - [ ] Test certificate expiry and renewal scenarios
    - [ ] Test certificate revocation and rollback

---

## PKI Implementation Summary

**Total Estimated Effort:** 44-58 developer-days across 4 phases

**Critical Path:** Phase 1 (Foundational Persistence) must be completed before
any other PKI work can begin. Current PKI service is not production-ready due to
in-memory state that is lost on restart.

**Quick Wins for Immediate Value:**

1. Add `bootstrap pki status` CLI command (even before persistence)
2. Formalize certificate paths as configurable defaults
3. Improve error handling with specific validation messages

**Deployment Strategy:**

- **Phase 1**: Enables persistent PKI state - required for production
- **Phase 2A**: Manual CA upload - enables enterprise deployments
- **Phase 2B**: Auto-generation - enables turnkey cloud deployments
- **Phase 3**: Enhanced architecture - improves reliability and operations
- **Phase 4**: Lifecycle management - ensures long-term stability

**Dependencies:** Phase 1 blocks all other work. Phases 2A and 2B can be
developed in parallel. Phase 3 requires completion of Phase 2. Phase 4 can be
deferred until after production deployment.

#### Testing Infrastructure ✅ COMPLETE

- [x] Docker Compose multi-node cluster testing
  - [x] 3-node bootstrap cluster configuration with priority-based election
  - [x] Self-signed certificate generation for HTTPS testing
  - [x] Dynamic TLS support (falls back to HTTP if certs unavailable)
  - [x] Mock Bottlerocket API services for integration testing
  - [x] gRPC UI and Prometheus monitoring for development
- [x] Successfully demonstrated election, PKI, and etcd formation phases

#### Remaining Phase 2 Tasks

- [ ] etcd static pod generation
  - [ ] Generate etcd manifests with proper TLS configuration
  - [ ] Implement pod specification templates
  - [ ] Add health check configurations
- [ ] etcd cluster initialization
  - [ ] Implement initial cluster bootstrapping
  - [ ] Add member discovery and join logic
  - [ ] Create cluster health monitoring
- [ ] Bootstrap coordinator orchestration
  - [ ] Wire up all three services (election, PKI, etcd)
  - [ ] Implement state machine for bootstrap phases
  - [ ] Add error recovery and rollback logic
- [ ] Comprehensive testing
  - [ ] Unit tests for election scenarios
  - [ ] Integration tests for PKI generation
  - [ ] End-to-end cluster formation tests

### Phase 3: Multi-Cluster Management 📋 TODO

- [ ] Federation model design
- [ ] Cross-cluster discovery
- [ ] Unified control plane
- [ ] Multi-cluster health aggregation

### Phase 4: Agent Capabilities 📋 TODO

- [ ] Dynamic capability registration
- [ ] Resource monitoring integration
- [ ] Custom health checks
- [ ] Extensible action framework

## Technical Debt & Improvements

- [ ] Add metrics and observability (Prometheus/OpenTelemetry)
- [ ] Implement rate limiting for API endpoints
- [ ] Add comprehensive error codes and documentation
- [ ] Create deployment automation scripts
- [ ] Performance optimization for large clusters

## Documentation Needs

- [ ] API reference documentation
- [ ] Deployment guide
- [ ] Security best practices
- [ ] Troubleshooting guide
- [ ] Architecture deep-dive

## Notes

- Bootstrap module compilation: 2,106 lines of Rust + 575 lines of protobuf
- Error reduction: 64 → 11 → 4 → 0 ✅ (all async Send trait issues resolved)
- Bootstrap module now compiles successfully
- Next immediate action: Implement PKI distribution mechanism
