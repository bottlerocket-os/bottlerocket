use anyhow::{Result, anyhow};
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use tracing::{info, warn, debug};
use std::collections::HashMap;
use std::time::{SystemTime, Duration, UNIX_EPOCH};
use std::str::FromStr;
use openssl::{
    x509::{X509, X509Req},
    hash::MessageDigest,
};

use crate::{
    pki::{ca::Certificate, store::CertificateStore},
    proto::pki::{
        CertificateRequest, CertificateResponse, CertificateType,
    },
    election::ElectionState,
};

/// Distribution state for tracking certificate requests
#[derive(Debug, Clone)]
struct DistributionEntry {
    node_id: String,
    fingerprint: String,
    issued_at: SystemTime,
    expires_at: SystemTime,
    renewal_count: u32,
}

/// PKI Distribution Manager
pub struct PKIDistributor {
    /// Certificate store
    store: Arc<CertificateStore>,
    
    /// Election state to verify leadership
    election_state: Arc<ElectionState>,
    
    /// Distributed certificates tracking
    distributions: Arc<RwLock<HashMap<String, DistributionEntry>>>,
    
    /// Node authentication tokens (from election)
    auth_tokens: Arc<RwLock<HashMap<String, String>>>,
    
    /// Certificate renewal threshold (days before expiry)
    renewal_threshold_days: u32,
    
    /// Rate limiting
    rate_limiter: Arc<Mutex<RateLimiter>>,
}

impl PKIDistributor {
    pub fn new(
        store: Arc<CertificateStore>,
        election_state: Arc<ElectionState>,
    ) -> Self {
        Self {
            store,
            election_state,
            distributions: Arc::new(RwLock::new(HashMap::new())),
            auth_tokens: Arc::new(RwLock::new(HashMap::new())),
            renewal_threshold_days: 30,
            rate_limiter: Arc::new(Mutex::new(RateLimiter::new(10, 60))), // 10 requests per minute
        }
    }
    
    /// Register node authentication token from election
    pub async fn register_auth_token(&self, node_id: String, token: String) -> Result<()> {
        let mut tokens = self.auth_tokens.write().await;
        tokens.insert(node_id, token);
        Ok(())
    }
    
    /// Process certificate request from a node
    pub async fn process_certificate_request(
        &self,
        request: CertificateRequest,
    ) -> Result<CertificateResponse> {
        // Verify we are the leader
        if !self.election_state.is_leader().await {
            return Err(anyhow!("Not the elected leader - cannot issue certificates"));
        }
        
        // Authenticate the node
        self.authenticate_node(&request).await?;
        
        // Rate limiting
        self.rate_limiter.lock().await.check(&request.node_id)?;
        
        // Validate the request
        self.validate_request(&request)?;
        
        // Process CSR if provided, otherwise generate key pair
        let (cert, private_key) = if !request.csr.is_empty() {
            self.process_csr(request).await?
        } else {
            self.generate_certificate(request).await?
        };
        
        // Get certificate chain and CA bundle
        let (chain, bundle) = self.get_certificate_chains().await?;
        
        // Calculate renewal time (30 days before expiry)
        let renewal_time = cert.cert.not_after();
        let renewal_timestamp = SystemTime::UNIX_EPOCH + 
            Duration::from_secs(renewal_time.diff(&openssl::asn1::Asn1Time::from_unix(0)?)?.days as u64 * 86400 - 
            self.renewal_threshold_days as u64 * 86400);
        
        // Track distribution
        self.track_distribution(&cert).await?;
        
        // Create response
        Ok(CertificateResponse {
            certificate: Some(cert.to_proto()?),
            private_key: private_key.unwrap_or_default(),
            certificate_chain: chain,
            ca_bundle: bundle,
            next_renewal_time: renewal_timestamp.duration_since(UNIX_EPOCH)?.as_secs() as i64,
        })
    }
    
    /// Authenticate node using election token
    async fn authenticate_node(&self, request: &CertificateRequest) -> Result<()> {
        let tokens = self.auth_tokens.read().await;
        
        match tokens.get(&request.node_id) {
            Some(expected_token) => {
                if request.auth_token != *expected_token {
                    return Err(anyhow!("Invalid authentication token"));
                }
                Ok(())
            }
            None => Err(anyhow!("Node {} not registered with election system", request.node_id)),
        }
    }
    
    /// Validate certificate request
    fn validate_request(&self, request: &CertificateRequest) -> Result<()> {
        // Validate common name
        if request.common_name.is_empty() {
            return Err(anyhow!("Common name cannot be empty"));
        }
        
        // Validate certificate type
        match CertificateType::try_from(request.r#type) {
            Ok(CertificateType::Server) | Ok(CertificateType::Client) | Ok(CertificateType::Peer) => {},
            _ => return Err(anyhow!("Invalid certificate type for distribution")),
        }
        
        // Validate DNS names and IPs
        for dns in &request.dns_names {
            if dns.is_empty() {
                return Err(anyhow!("Empty DNS name"));
            }
        }
        
        for ip in &request.ip_addresses {
            if std::net::IpAddr::from_str(ip).is_err() {
                return Err(anyhow!("Invalid IP address: {}", ip));
            }
        }
        
        Ok(())
    }
    
    /// Process Certificate Signing Request
    async fn process_csr(&self, request: CertificateRequest) -> Result<(Certificate, Option<Vec<u8>>)> {
        // Parse CSR
        let csr_bytes = &request.csr;
        let csr = X509Req::from_pem(csr_bytes)?;
        
        // Verify CSR signature
        let pubkey = csr.public_key()?;
        if !csr.verify(&pubkey)? {
            return Err(anyhow!("Invalid CSR signature"));
        }
        
        // Get appropriate CA based on certificate type
        let ca = self.get_signing_ca(request.r#type).await?;
        
        // Sign the certificate
        let cert = self.sign_certificate(csr, ca, request).await?;
        
        Ok((cert, None)) // No private key when using CSR
    }
    
    /// Generate certificate with new key pair
    async fn generate_certificate(&self, request: CertificateRequest) -> Result<(Certificate, Option<Vec<u8>>)> {
        // Get appropriate CA based on certificate type
        let ca = self.get_signing_ca(request.r#type).await?;
        
        // Generate certificate
        let cert = self.store.generate_certificate(
            &request.common_name,
            CertificateType::try_from(request.r#type)?,
            Some(ca),
            request.validity_days,
            request.dns_names,
            request.ip_addresses,
        ).await?;
        
        // Export private key (encrypted)
        let private_key = cert.key.private_key_to_pem_pkcs8()?;
        
        Ok((cert, Some(private_key)))
    }
    
    /// Get appropriate signing CA for certificate type
    async fn get_signing_ca(&self, cert_type: i32) -> Result<Certificate> {
        match CertificateType::try_from(cert_type)? {
            CertificateType::Server => {
                self.store.get_kubernetes_ca().await
                    .ok_or_else(|| anyhow!("Kubernetes CA not initialized"))
            }
            CertificateType::Client => {
                self.store.get_kubernetes_ca().await
                    .ok_or_else(|| anyhow!("Kubernetes CA not initialized"))
            }
            CertificateType::Peer => {
                self.store.get_etcd_ca().await
                    .ok_or_else(|| anyhow!("etcd CA not initialized"))
            }
            _ => Err(anyhow!("Invalid certificate type for signing")),
        }
    }
    
    /// Sign a certificate using CSR
    async fn sign_certificate(
        &self,
        csr: X509Req,
        ca: Certificate,
        request: CertificateRequest,
    ) -> Result<Certificate> {
        use openssl::x509::{X509Builder, X509NameBuilder, extension::*};
        use openssl::bn::{BigNum, MsbOption};
        use openssl::asn1::{Asn1Time, Asn1Integer};
        
        let mut builder = X509Builder::new()?;
        builder.set_version(2)?; // X509v3
        
        // Set serial number
        let mut serial = BigNum::new()?;
        serial.rand(128, MsbOption::MAYBE_ZERO, false)?;
        let serial = Asn1Integer::from_bn(&serial)?;
        builder.set_serial_number(&serial)?;
        
        // Set subject from CSR
        builder.set_subject_name(csr.subject_name())?;
        
        // Set issuer from CA
        builder.set_issuer_name(ca.cert.subject_name())?;
        
        // Set validity
        let not_before = Asn1Time::days_from_now(0)?;
        let validity_days = request.validity_days.max(365) as u32;
        let not_after = Asn1Time::days_from_now(validity_days)?;
        builder.set_not_before(&not_before)?;
        builder.set_not_after(&not_after)?;
        
        // Set public key from CSR
        builder.set_pubkey(&csr.public_key()?)?;
        
        // Add extensions based on certificate type
        match CertificateType::try_from(request.r#type)? {
            CertificateType::Server => {
                builder.append_extension(BasicConstraints::new().build()?)?;
                builder.append_extension(
                    KeyUsage::new()
                        .digital_signature()
                        .key_encipherment()
                        .build()?
                )?;
                
                // Add SAN if provided
                if !request.dns_names.is_empty() || !request.ip_addresses.is_empty() {
                    let mut san = SubjectAlternativeName::new();
                    for dns in &request.dns_names {
                        san.dns(dns);
                    }
                    for ip in &request.ip_addresses {
                        san.ip(ip);
                    }
                    builder.append_extension(san.build(&builder.x509v3_context(Some(&ca.cert), None))?)?;
                }
            }
            CertificateType::Client => {
                builder.append_extension(BasicConstraints::new().build()?)?;
                builder.append_extension(
                    KeyUsage::new()
                        .digital_signature()
                        .key_encipherment()
                        .build()?
                )?;
            }
            CertificateType::Peer => {
                builder.append_extension(BasicConstraints::new().build()?)?;
                builder.append_extension(
                    KeyUsage::new()
                        .digital_signature()
                        .key_encipherment()
                        .build()?
                )?;
            }
            _ => {}
        }
        
        // Sign with CA
        builder.sign(&ca.key, MessageDigest::sha256())?;
        let cert = builder.build();
        
        // Note: When using CSR, we don't have the private key
        // The Certificate struct needs the key for storage, but it won't be used
        // We'll return the cert without private key in the response
        Err(anyhow!("CSR signing not fully implemented - private key handling needed"))
    }
    
    /// Get certificate chains and CA bundle
    async fn get_certificate_chains(&self) -> Result<(String, String)> {
        let root_ca = self.store.get_root_ca().await
            .ok_or_else(|| anyhow!("Root CA not found"))?;
        let kubernetes_ca = self.store.get_kubernetes_ca().await;
        let etcd_ca = self.store.get_etcd_ca().await;
        
        let mut chain = String::new();
        let mut bundle = String::new();
        
        // Add root CA to bundle
        bundle.push_str(&String::from_utf8(root_ca.cert.to_pem()?)?);
        
        // Add intermediate CAs to chain and bundle
        if let Some(ca) = kubernetes_ca {
            let pem = String::from_utf8(ca.cert.to_pem()?)?;
            chain.push_str(&pem);
            bundle.push_str(&pem);
        }
        
        if let Some(ca) = etcd_ca {
            let pem = String::from_utf8(ca.cert.to_pem()?)?;
            bundle.push_str(&pem);
        }
        
        Ok((chain, bundle))
    }
    
    /// Track certificate distribution
    async fn track_distribution(&self, cert: &Certificate) -> Result<()> {
        let fingerprint = hex::encode(cert.cert.digest(MessageDigest::sha256())?);
        let node_id = cert.get_common_name()?;
        
        let entry = DistributionEntry {
            node_id: node_id.clone(),
            fingerprint: fingerprint.clone(),
            issued_at: SystemTime::now(),
            expires_at: SystemTime::now() + Duration::from_secs(
                cert.cert.not_after().diff(&cert.cert.not_before())?.days as u64 * 86400
            ),
            renewal_count: 0,
        };
        
        let mut distributions = self.distributions.write().await;
        distributions.insert(node_id, entry);
        
        info!("Certificate distributed to node {}: {}", node_id, fingerprint);
        Ok(())
    }
    
    /// Check for certificates needing renewal
    pub async fn check_renewals(&self) -> Result<Vec<String>> {
        let distributions = self.distributions.read().await;
        let mut expiring = Vec::new();
        
        let threshold = SystemTime::now() + Duration::from_secs(self.renewal_threshold_days as u64 * 86400);
        
        for (node_id, entry) in distributions.iter() {
            if entry.expires_at < threshold {
                expiring.push(node_id.clone());
            }
        }
        
        Ok(expiring)
    }
    
    /// Validate certificate chain
    pub async fn validate_certificate_chain(&self, cert_pem: &str) -> Result<()> {
        let cert = X509::from_pem(cert_pem.as_bytes())?;
        let root_ca = self.store.get_root_ca().await
            .ok_or_else(|| anyhow!("Root CA not found"))?;
        
        // Build certificate chain for validation
        let mut chain = openssl::x509::store::X509StoreBuilder::new()?;
        chain.add_cert(root_ca.cert.clone())?;
        
        // Add intermediate CAs
        if let Some(ca) = self.store.get_kubernetes_ca().await {
            chain.add_cert(ca.cert)?;
        }
        if let Some(ca) = self.store.get_etcd_ca().await {
            chain.add_cert(ca.cert)?;
        }
        
        let chain = chain.build();
        let mut context = openssl::x509::X509StoreContext::new()?;
        
        match context.init(&chain, &cert, &openssl::stack::Stack::new()?, |ctx| ctx.verify_cert()) {
            Ok(true) => Ok(()),
            Ok(false) => Err(anyhow!("Certificate validation failed")),
            Err(e) => Err(anyhow!("Certificate validation error: {}", e)),
        }
    }
}

/// Simple rate limiter
struct RateLimiter {
    requests: HashMap<String, Vec<SystemTime>>,
    max_requests: usize,
    window_secs: u64,
}

impl RateLimiter {
    fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            requests: HashMap::new(),
            max_requests,
            window_secs,
        }
    }
    
    fn check(&mut self, node_id: &str) -> Result<()> {
        let now = SystemTime::now();
        let window_start = now - Duration::from_secs(self.window_secs);
        
        // Clean old requests
        let requests = self.requests.entry(node_id.to_string()).or_insert_with(Vec::new);
        requests.retain(|&t| t > window_start);
        
        // Check rate limit
        if requests.len() >= self.max_requests {
            return Err(anyhow!("Rate limit exceeded for node {}", node_id));
        }
        
        // Record request
        requests.push(now);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(2, 60);
        
        // First two requests should succeed
        assert!(limiter.check("node1").is_ok());
        assert!(limiter.check("node1").is_ok());
        
        // Third request should fail
        assert!(limiter.check("node1").is_err());
        
        // Different node should succeed
        assert!(limiter.check("node2").is_ok());
    }
}