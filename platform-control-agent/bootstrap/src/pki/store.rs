use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use tracing::{info, debug};

use crate::pki::ca::Certificate;
use crate::proto::pki::CertificateType;

/// Thread-safe certificate store
#[derive(Clone)]
pub struct CertificateStore {
    /// Root CA certificate
    root_ca: Arc<RwLock<Option<Certificate>>>,
    
    /// Kubernetes intermediate CA
    kubernetes_ca: Arc<RwLock<Option<Certificate>>>,
    
    /// etcd intermediate CA
    etcd_ca: Arc<RwLock<Option<Certificate>>>,
    
    /// Front proxy intermediate CA
    front_proxy_ca: Arc<RwLock<Option<Certificate>>>,
    
    /// Issued certificates by fingerprint
    certificates: Arc<RwLock<HashMap<String, Certificate>>>,
    
    /// Index by type
    by_type: Arc<RwLock<HashMap<CertificateType, Vec<String>>>>,
}

impl CertificateStore {
    pub fn new() -> Self {
        Self {
            root_ca: Arc::new(RwLock::new(None)),
            kubernetes_ca: Arc::new(RwLock::new(None)),
            etcd_ca: Arc::new(RwLock::new(None)),
            front_proxy_ca: Arc::new(RwLock::new(None)),
            certificates: Arc::new(RwLock::new(HashMap::new())),
            by_type: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Store root CA
    pub async fn store_root_ca(&self, cert: Certificate) -> Result<()> {
        let mut root_ca = self.root_ca.write().await;
        *root_ca = Some(cert);
        info!("Root CA stored successfully");
        Ok(())
    }
    
    /// Get root CA
    pub async fn get_root_ca(&self) -> Option<Certificate> {
        self.root_ca.read().await.clone()
    }
    
    /// Store Kubernetes CA
    pub async fn store_kubernetes_ca(&self, cert: Certificate) -> Result<()> {
        let mut k8s_ca = self.kubernetes_ca.write().await;
        *k8s_ca = Some(cert);
        info!("Kubernetes CA stored successfully");
        Ok(())
    }
    
    /// Get Kubernetes CA
    pub async fn get_kubernetes_ca(&self) -> Option<Certificate> {
        self.kubernetes_ca.read().await.clone()
    }
    
    /// Store etcd CA
    pub async fn store_etcd_ca(&self, cert: Certificate) -> Result<()> {
        let mut etcd_ca = self.etcd_ca.write().await;
        *etcd_ca = Some(cert);
        info!("etcd CA stored successfully");
        Ok(())
    }
    
    /// Get etcd CA
    pub async fn get_etcd_ca(&self) -> Option<Certificate> {
        self.etcd_ca.read().await.clone()
    }
    
    /// Store front proxy CA
    pub async fn store_front_proxy_ca(&self, cert: Certificate) -> Result<()> {
        let mut fp_ca = self.front_proxy_ca.write().await;
        *fp_ca = Some(cert);
        info!("Front proxy CA stored successfully");
        Ok(())
    }
    
    /// Get front proxy CA
    pub async fn get_front_proxy_ca(&self) -> Option<Certificate> {
        self.front_proxy_ca.read().await.clone()
    }
    
    /// Store a general certificate
    pub async fn store_certificate(&self, cert: Certificate) -> Result<String> {
        use openssl::hash::MessageDigest;
        
        // Calculate fingerprint
        let fingerprint = hex::encode(cert.cert.digest(MessageDigest::sha256())?);
        
        // Store by fingerprint
        let mut certificates = self.certificates.write().await;
        certificates.insert(fingerprint.clone(), cert.clone());
        
        // Index by type
        let mut by_type = self.by_type.write().await;
        by_type
            .entry(cert.cert_type)
            .or_insert_with(Vec::new)
            .push(fingerprint.clone());
        
        debug!("Certificate stored: {} (type: {:?})", fingerprint, cert.cert_type);
        Ok(fingerprint)
    }
    
    /// Get certificate by fingerprint
    pub async fn get_certificate(&self, fingerprint: &str) -> Option<Certificate> {
        self.certificates.read().await.get(fingerprint).cloned()
    }
    
    /// List certificates by type
    pub async fn list_by_type(&self, cert_type: CertificateType) -> Vec<Certificate> {
        let certificates = self.certificates.read().await;
        let by_type = self.by_type.read().await;
        
        by_type
            .get(&cert_type)
            .map(|fingerprints| {
                fingerprints
                    .iter()
                    .filter_map(|fp| certificates.get(fp).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// List all certificates
    pub async fn list_all(&self) -> Vec<Certificate> {
        self.certificates.read().await.values().cloned().collect()
    }
    
    /// Generate a certificate (helper method for distribution)
    pub async fn generate_certificate(
        &self,
        common_name: &str,
        cert_type: CertificateType,
        signing_ca: Option<Certificate>,
        validity_days: Option<i32>,
        dns_names: Vec<String>,
        ip_addresses: Vec<String>,
    ) -> Result<Certificate> {
        use openssl::{
            asn1::{Asn1Time, Asn1Integer},
            bn::{BigNum, MsbOption},
            hash::MessageDigest,
            pkey::PKey,
            rsa::Rsa,
            x509::{
                extension::{BasicConstraints, KeyUsage, SubjectAlternativeName},
                X509NameBuilder, X509Builder,
            },
        };
        
        // Generate key pair
        let rsa = Rsa::generate(4096)?;
        let key = PKey::from_rsa(rsa)?;
        
        // Build certificate
        let mut builder = X509Builder::new()?;
        builder.set_version(2)?; // X509v3
        
        // Set serial number
        let mut serial = BigNum::new()?;
        serial.rand(128, MsbOption::MAYBE_ZERO, false)?;
        let serial = Asn1Integer::from_bn(&serial)?;
        builder.set_serial_number(&serial)?;
        
        // Build subject
        let mut subject_builder = X509NameBuilder::new()?;
        subject_builder.append_entry_by_text("CN", common_name)?;
        if let Some(ca) = &signing_ca {
            // Copy organization info from CA if available
            let ca_subject = ca.cert.subject_name();
            for entry in ca_subject.entries() {
                match entry.object().to_string().as_str() {
                    "organizationName" => {
                        subject_builder.append_entry_by_text("O", &entry.data().as_utf8()?.to_string())?;
                    }
                    "countryName" => {
                        subject_builder.append_entry_by_text("C", &entry.data().as_utf8()?.to_string())?;
                    }
                    "localityName" => {
                        subject_builder.append_entry_by_text("L", &entry.data().as_utf8()?.to_string())?;
                    }
                    _ => {}
                }
            }
        }
        let subject = subject_builder.build();
        builder.set_subject_name(&subject)?;
        
        // Set issuer (from CA or self-signed)
        if let Some(ca) = &signing_ca {
            builder.set_issuer_name(ca.cert.subject_name())?;
        } else {
            builder.set_issuer_name(&subject)?;
        }
        
        // Set validity
        let not_before = Asn1Time::days_from_now(0)?;
        let validity_days = validity_days.unwrap_or(365) as u32;
        let not_after = Asn1Time::days_from_now(validity_days)?;
        builder.set_not_before(&not_before)?;
        builder.set_not_after(&not_after)?;
        
        // Set public key
        builder.set_pubkey(&key)?;
        
        // Add extensions based on certificate type
        match cert_type {
            CertificateType::Server => {
                builder.append_extension(BasicConstraints::new().build()?)?;
                builder.append_extension(
                    KeyUsage::new()
                        .digital_signature()
                        .key_encipherment()
                        .build()?
                )?;
                
                // Add SAN if provided
                if !dns_names.is_empty() || !ip_addresses.is_empty() {
                    let mut san = SubjectAlternativeName::new();
                    for dns in &dns_names {
                        san.dns(dns);
                    }
                    for ip in &ip_addresses {
                        san.ip(ip);
                    }
                    
                    let ca_cert = signing_ca.as_ref().map(|c| &c.cert);
                    builder.append_extension(san.build(&builder.x509v3_context(ca_cert, None))?)?;
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
                
                // Add SAN for peer certificates
                if !dns_names.is_empty() || !ip_addresses.is_empty() {
                    let mut san = SubjectAlternativeName::new();
                    for dns in &dns_names {
                        san.dns(dns);
                    }
                    for ip in &ip_addresses {
                        san.ip(ip);
                    }
                    
                    let ca_cert = signing_ca.as_ref().map(|c| &c.cert);
                    builder.append_extension(san.build(&builder.x509v3_context(ca_cert, None))?)?;
                }
            }
            _ => {}
        }
        
        // Sign the certificate
        if let Some(ca) = signing_ca {
            builder.sign(&ca.key, MessageDigest::sha256())?;
        } else {
            builder.sign(&key, MessageDigest::sha256())?;
        }
        
        let cert = builder.build();
        
        Ok(Certificate {
            cert,
            key,
            cert_type,
        })
    }
}