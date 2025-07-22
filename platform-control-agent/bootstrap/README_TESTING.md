# Testing Bootstrap Components with Docker Compose

This guide explains how to test the election, PKI, and etcd components using
Docker Compose.

## Quick Start

Run the automated test script:

```bash
./test/test_bootstrap_cluster.sh
```

## Manual Testing

### 1. Start the Bootstrap Cluster

```bash
# Build and start a 3-node cluster
docker-compose -f docker-compose.bootstrap.yml up --build
```

This starts:

- 3 bootstrap nodes with different priorities (200, 150, 100)
- Mock Bottlerocket APIs for each node
- gRPC UI for API testing
- Prometheus for metrics

### 2. Monitor Election Process

Watch the logs to see leader election:

```bash
# Watch all logs
docker-compose -f docker-compose.bootstrap.yml logs -f

# Filter for election events
docker-compose -f docker-compose.bootstrap.yml logs -f | grep -i election

# See which node became leader
docker-compose -f docker-compose.bootstrap.yml logs | grep "Transitioning to leader"
```

Expected behavior:

- Node 1 (priority 200) should become leader
- Nodes 2 and 3 remain followers
- Election completes within 5-10 seconds

### 3. Test PKI Distribution

The elected leader initializes the PKI system:

```bash
# Watch PKI events
docker-compose -f docker-compose.bootstrap.yml logs -f | grep -i pki

# Check certificate generation
docker-compose -f docker-compose.bootstrap.yml logs | grep "Generated.*CA"
```

Expected behavior:

- Leader generates Root CA, Kubernetes CA, etcd CA, Front Proxy CA
- Only the leader can issue certificates
- Followers can request certificates from the leader

### 4. Monitor etcd Formation

Watch etcd cluster formation:

```bash
# Watch etcd events
docker-compose -f docker-compose.bootstrap.yml logs -f | grep -i etcd

# Check static pod generation
docker-compose -f docker-compose.bootstrap.yml logs | grep "static pod"
```

Expected behavior:

- Leader initializes etcd cluster
- Static pod manifests are generated
- Followers join using secure tokens

## Testing with gRPC UI

Access the gRPC UI at http://localhost:8082 to:

1. **Election Service**:

   - `GetLeader` - Check current leader
   - `Observe` - Stream election events
   - `Campaign` - Force new election

2. **PKI Service**:

   - `InitializePKI` - Initialize PKI (leader only)
   - `GetCABundle` - Retrieve CA certificates
   - `ListCertificates` - List issued certificates

3. **etcd Service**:
   - `GetStatus` - Check etcd cluster status
   - `ObserveCluster` - Stream cluster events

## Monitoring with Prometheus

Access metrics at http://localhost:9092:

- `election_term` - Current election term
- `election_state` - Node state (follower/candidate/leader)
- `pki_certificates_issued` - Number of certificates issued
- `etcd_cluster_size` - Number of etcd members

## Troubleshooting

### No Leader Elected

- Check network connectivity between nodes
- Verify all nodes have unique IDs
- Check logs for timeout issues

### PKI Initialization Fails

- Ensure only the leader attempts initialization
- Check for OpenSSL/FIPS errors
- Verify certificate storage permissions

### etcd Formation Issues

- Check PKI certificates are available
- Verify network ports are accessible
- Look for token expiration

## Clean Up

```bash
# Stop all containers
docker-compose -f docker-compose.bootstrap.yml down

# Remove volumes (certificates, data)
docker-compose -f docker-compose.bootstrap.yml down -v
```

## Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│ Bootstrap Node 1│     │ Bootstrap Node 2│     │ Bootstrap Node 3│
│ Priority: 200   │     │ Priority: 150   │     │ Priority: 100   │
│ (Leader)        │     │ (Follower)      │     │ (Follower)      │
└────────┬────────┘     └────────┬────────┘     └────────┬────────┘
         │                       │                       │
         └───────────────────────┴───────────────────────┘
                            gRPC Communication
                                    │
                        ┌───────────┴───────────┐
                        │                       │
                    Election              PKI & etcd
                    (Raft consensus)      (Leader-driven)
```

## Expected Test Results

1. **Election**: Node with highest priority becomes leader
2. **PKI**: 4 CAs generated, certificate distribution ready
3. **etcd**: Cluster initialized, static pods configured
