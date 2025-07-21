# PKI Distribution Mechanism

## Overview

The PKI distribution mechanism provides secure certificate generation and distribution for cluster nodes. Only the elected leader can issue certificates, ensuring centralized control and preventing split-brain scenarios.

## Architecture

### Components

1. **PKIDistributor** (`src/pki/distribution.rs`)
   - Manages certificate requests and distribution
   - Authenticates nodes using election tokens
   - Rate limiting to prevent abuse
   - Certificate lifecycle tracking

2. **CertificateStore** (`src/pki/store.rs`)
   - Thread-safe storage for CA certificates
   - Certificate generation utilities
   - Indexed retrieval by type

3. **PKIService** (`src/pki/service.rs`)
   - gRPC service implementation
   - Leader verification
   - Event streaming for certificate lifecycle
   - Background renewal monitoring

4. **PKIClient** (`src/pki/client.rs`)
   - Client library for nodes
   - Certificate request API
   - CA bundle retrieval

## Security Features

### Authentication
- Nodes must be registered with the election system
- Each node receives a unique authentication token
- Tokens are validated before certificate issuance

### Authorization
- Only the elected leader can issue certificates
- Leadership is verified for each PKI operation
- Prevents unauthorized certificate generation

### Certificate Types
- **Server**: For API endpoints and services
- **Client**: For client authentication
- **Peer**: For etcd peer communication

### Rate Limiting
- 10 requests per minute per node
- Prevents certificate exhaustion attacks
- Configurable limits

## Certificate Lifecycle

### Issuance
1. Node authenticates with election token
2. Leader verifies request validity
3. Certificate generated with appropriate CA
4. Certificate and chain returned to node

### Renewal
- Automatic monitoring 30 days before expiry
- Event notifications for expiring certificates
- Zero-downtime renewal process

### Validation
- Full certificate chain validation
- FIPS-compliant algorithms (RSA 4096, SHA256)
- OpenSSL-based verification

## Usage Example

```rust
// Create certificate request
let request = CertificateRequest {
    common_name: "node-1.cluster.local".to_string(),
    r#type: CertificateType::Server as i32,
    dns_names: vec!["node-1.cluster.local".to_string()],
    ip_addresses: vec!["10.0.0.100".to_string()],
    validity_days: 365,
    node_id: "node-1".to_string(),
    auth_token: "secret-token".to_string(),
    ..Default::default()
};

// Process request (leader only)
let response = distributor.process_certificate_request(request).await?;
```

## Integration with Election System

The PKI distribution is tightly integrated with the election system:
- Only elected leaders can issue certificates
- Node authentication tokens from election
- Leadership changes invalidate pending requests

## Future Enhancements

1. **Certificate Revocation**
   - CRL generation and distribution
   - OCSP responder support

2. **Certificate Rotation**
   - Automated rotation policies
   - Service restart coordination

3. **Backup and Recovery**
   - Encrypted CA key backup
   - Disaster recovery procedures

4. **CSR Support**
   - Full certificate signing request support
   - External key management integration

## Testing

Run the PKI distribution tests:
```bash
cargo test -p platform-bootstrap pki_distribution
```

Run the example:
```bash
cargo run --example pki_distribution
```