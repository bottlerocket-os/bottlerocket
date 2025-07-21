use std::collections::HashMap;
use anyhow::Result;
use crate::proto::pki::{Certificate as ProtoCertificate, CertificateType};

/// In-memory certificate store
pub struct CertificateStore {
    certificates: HashMap<String, ProtoCertificate>,
    by_type: HashMap<CertificateType, Vec<String>>,
}

impl CertificateStore {
    pub fn new() -> Self {
        Self {
            certificates: HashMap::new(),
            by_type: HashMap::new(),
        }
    }
    
    /// Store a certificate
    pub fn store(&mut self, cert: ProtoCertificate) -> Result<()> {
        let fingerprint = cert.fingerprint.clone();
        let cert_type = CertificateType::try_from(cert.r#type)
            .unwrap_or(CertificateType::Unknown);
        
        // Store by fingerprint
        self.certificates.insert(fingerprint.clone(), cert);
        
        // Index by type
        self.by_type
            .entry(cert_type)
            .or_insert_with(Vec::new)
            .push(fingerprint);
        
        Ok(())
    }
    
    /// Get certificate by fingerprint
    pub fn get(&self, fingerprint: &str) -> Option<&ProtoCertificate> {
        self.certificates.get(fingerprint)
    }
    
    /// List certificates by type
    pub fn list_by_type(&self, cert_type: CertificateType) -> Vec<&ProtoCertificate> {
        self.by_type
            .get(&cert_type)
            .map(|fingerprints| {
                fingerprints
                    .iter()
                    .filter_map(|fp| self.certificates.get(fp))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// List all certificates
    pub fn list_all(&self) -> Vec<&ProtoCertificate> {
        self.certificates.values().collect()
    }
}