use anyhow::{Result, anyhow};
use openssl::{
    asn1::Asn1Time,
    bn::{BigNum, MsbOption},
    hash::MessageDigest,
    pkey::{PKey, Private},
    rsa::Rsa,
    x509::{
        extension::{BasicConstraints, KeyUsage, SubjectKeyIdentifier, AuthorityKeyIdentifier, SubjectAlternativeName},
        X509NameBuilder, X509Builder, X509,
    },
};
use tracing::{info, debug};
use prost_types::Timestamp;

use crate::proto::pki::{Certificate as ProtoCertificate, CertificateType, CertificateStatus};

/// Certificate with associated private key
#[derive(Clone)]
pub struct Certificate {
    pub cert: X509,
    pub key: PKey<Private>,
    pub cert_type: CertificateType,
}

impl Certificate {
    /// Convert to PEM encoded strings
    pub fn to_pem(&self) -> Result<(String, String)> {
        let cert_pem = String::from_utf8(self.cert.to_pem()?)?;
        let key_pem = String::from_utf8(self.key.private_key_to_pem_pkcs8()?)?;
        Ok((cert_pem, key_pem))
    }
    
    /// Convert ASN1 time to prost Timestamp
    fn asn1_to_timestamp(time: &openssl::asn1::Asn1TimeRef) -> Timestamp {
        // Get seconds since epoch
        let epoch = Asn1Time::from_unix(0).unwrap();
        let diff = time.diff(&epoch).unwrap();
        Timestamp {
            seconds: diff.days as i64 * 86400 + diff.secs as i64,
            nanos: 0,
        }
    }
    
    /// Convert to proto Certificate (without private key)
    pub fn to_proto(&self) -> Result<ProtoCertificate> {
        let cert_pem = String::from_utf8(self.cert.to_pem()?)?;
        let fingerprint = hex::encode(self.cert.digest(MessageDigest::sha256())?);
        
        Ok(ProtoCertificate {
            fingerprint,
            common_name: self.get_common_name()?,
            r#type: self.cert_type as i32,
            not_before: Some(Self::asn1_to_timestamp(self.cert.not_before())),
            not_after: Some(Self::asn1_to_timestamp(self.cert.not_after())),
            dns_names: self.get_san_dns_names()?,
            ip_addresses: self.get_san_ip_addresses()?,
            email_addresses: vec![],
            issuer: self.get_issuer()?,
            serial_number: self.cert.serial_number().to_bn()?.to_string(),
            status: CertificateStatus::Active as i32,
            pem_encoded: cert_pem,
        })
    }
    
    fn get_common_name(&self) -> Result<String> {
        let subject = self.cert.subject_name();
        for entry in subject.entries() {
            if entry.object().to_string() == "commonName" {
                return Ok(entry.data().as_utf8()?.to_string());
            }
        }
        Err(anyhow!("No common name found"))
    }
    
    fn get_issuer(&self) -> Result<String> {
        let issuer = self.cert.issuer_name();
        for entry in issuer.entries() {
            if entry.object().to_string() == "commonName" {
                return Ok(entry.data().as_utf8()?.to_string());
            }
        }
        Ok("Unknown".to_string())
    }
    
    fn get_san_dns_names(&self) -> Result<Vec<String>> {
        // TODO: Extract DNS names from SAN extension
        Ok(vec![])
    }
    
    fn get_san_ip_addresses(&self) -> Result<Vec<String>> {
        // TODO: Extract IP addresses from SAN extension
        Ok(vec![])
    }
}

/// PKI configuration
#[derive(Debug, Clone)]
pub struct PKIConfig {
    pub organization: String,
    pub country: String,
    pub locality: String,
    pub key_size: u32,
    pub root_ca_validity_years: u32,
    pub intermediate_ca_validity_years: u32,
    pub server_cert_validity_years: u32,
    pub client_cert_validity_years: u32,
}

impl Default for PKIConfig {
    fn default() -> Self {
        Self {
            organization: "Platform Kubernetes".to_string(),
            country: "US".to_string(),
            locality: "Cloud".to_string(),
            key_size: 4096,
            root_ca_validity_years: 10,
            intermediate_ca_validity_years: 5,
            server_cert_validity_years: 1,
            client_cert_validity_years: 1,
        }
    }
}

/// Certificate Authority
pub struct CertificateAuthority {
    config: PKIConfig,
    root_ca: Option<Certificate>,
    kubernetes_ca: Option<Certificate>,
    etcd_ca: Option<Certificate>,
    front_proxy_ca: Option<Certificate>,
}

impl CertificateAuthority {
    pub fn new(config: PKIConfig) -> Self {
        Self {
            config,
            root_ca: None,
            kubernetes_ca: None,
            etcd_ca: None,
            front_proxy_ca: None,
        }
    }
    
    /// Initialize the PKI system by generating all CAs
    pub fn initialize(&mut self) -> Result<()> {
        info!("Initializing PKI system");
        
        // Generate root CA
        self.root_ca = Some(self.generate_root_ca()?);
        info!("Generated root CA");
        
        // Generate intermediate CAs
        self.kubernetes_ca = Some(self.generate_intermediate_ca(
            "Kubernetes CA",
            &self.root_ca.as_ref().unwrap()
        )?);
        info!("Generated Kubernetes CA");
        
        self.etcd_ca = Some(self.generate_intermediate_ca(
            "etcd CA",
            &self.root_ca.as_ref().unwrap()
        )?);
        info!("Generated etcd CA");
        
        self.front_proxy_ca = Some(self.generate_intermediate_ca(
            "Front Proxy CA",
            &self.root_ca.as_ref().unwrap()
        )?);
        info!("Generated Front Proxy CA");
        
        Ok(())
    }
    
    /// Generate root CA certificate
    fn generate_root_ca(&self) -> Result<Certificate> {
        debug!("Generating root CA with {} bit RSA key", self.config.key_size);
        
        // Generate RSA key
        let rsa = Rsa::generate(self.config.key_size)?;
        let key = PKey::from_rsa(rsa)?;
        
        // Build certificate
        let mut builder = X509Builder::new()?;
        builder.set_version(2)?; // X509 v3
        
        // Set serial number (1 for root CA)
        let mut serial = BigNum::new()?;
        serial.add_word(1)?;
        let serial_asn1 = serial.to_asn1_integer()?;
        builder.set_serial_number(&serial_asn1)?;
        
        // Set subject and issuer (self-signed)
        let mut name_builder = X509NameBuilder::new()?;
        name_builder.append_entry_by_text("C", &self.config.country)?;
        name_builder.append_entry_by_text("O", &self.config.organization)?;
        name_builder.append_entry_by_text("L", &self.config.locality)?;
        name_builder.append_entry_by_text("CN", "Platform Root CA")?;
        let name = name_builder.build();
        
        builder.set_subject_name(&name)?;
        builder.set_issuer_name(&name)?;
        
        // Set validity period
        let not_before = Asn1Time::days_from_now(0)?;
        let validity_days = self.config.root_ca_validity_years.saturating_mul(365);
        let not_after = Asn1Time::days_from_now(validity_days)?;
        builder.set_not_before(&not_before)?;
        builder.set_not_after(&not_after)?;
        
        // Set public key
        builder.set_pubkey(&key)?;
        
        // Add extensions
        // Basic constraints: CA=true, no path length limit
        let basic_constraints = BasicConstraints::new()
            .critical()
            .ca()
            .build()?;
        builder.append_extension(basic_constraints)?;
        
        // Key usage
        let key_usage = KeyUsage::new()
            .critical()
            .key_cert_sign()
            .crl_sign()
            .build()?;
        builder.append_extension(key_usage)?;
        
        // Subject key identifier
        let context = builder.x509v3_context(None, None);
        let subject_key_id = SubjectKeyIdentifier::new()
            .build(&context)?;
        builder.append_extension(subject_key_id)?;
        
        // Sign the certificate
        builder.sign(&key, MessageDigest::sha256())?;
        let cert = builder.build();
        
        Ok(Certificate {
            cert,
            key,
            cert_type: CertificateType::RootCa,
        })
    }
    
    /// Generate intermediate CA certificate
    fn generate_intermediate_ca(&self, cn: &str, issuer: &Certificate) -> Result<Certificate> {
        debug!("Generating intermediate CA: {}", cn);
        
        // Generate RSA key
        let rsa = Rsa::generate(self.config.key_size)?;
        let key = PKey::from_rsa(rsa)?;
        
        // Build certificate
        let mut builder = X509Builder::new()?;
        builder.set_version(2)?;
        
        // Set serial number
        let mut serial = BigNum::new()?;
        serial.rand(64, MsbOption::MAYBE_ZERO, false)?;
        let serial_asn1 = serial.to_asn1_integer()?;
        builder.set_serial_number(&serial_asn1)?;
        
        // Set subject
        let mut name_builder = X509NameBuilder::new()?;
        name_builder.append_entry_by_text("C", &self.config.country)?;
        name_builder.append_entry_by_text("O", &self.config.organization)?;
        name_builder.append_entry_by_text("L", &self.config.locality)?;
        name_builder.append_entry_by_text("CN", cn)?;
        let name = name_builder.build();
        builder.set_subject_name(&name)?;
        
        // Set issuer (from root CA)
        builder.set_issuer_name(issuer.cert.subject_name())?;
        
        // Set validity period
        let not_before = Asn1Time::days_from_now(0)?;
        let validity_days = self.config.intermediate_ca_validity_years.saturating_mul(365);
        let not_after = Asn1Time::days_from_now(validity_days)?;
        builder.set_not_before(&not_before)?;
        builder.set_not_after(&not_after)?;
        
        // Set public key
        builder.set_pubkey(&key)?;
        
        // Add extensions
        // Basic constraints: CA=true, path length = 0
        let basic_constraints = BasicConstraints::new()
            .critical()
            .ca()
            .pathlen(0)
            .build()?;
        builder.append_extension(basic_constraints)?;
        
        // Key usage
        let key_usage = KeyUsage::new()
            .critical()
            .key_cert_sign()
            .crl_sign()
            .build()?;
        builder.append_extension(key_usage)?;
        
        // Extensions that need context
        let subject_key_id = {
            let context = builder.x509v3_context(Some(&issuer.cert), None);
            SubjectKeyIdentifier::new().build(&context)?
        };
        builder.append_extension(subject_key_id)?;
        
        // Authority key identifier
        let auth_key_id = {
            let context = builder.x509v3_context(Some(&issuer.cert), None);
            AuthorityKeyIdentifier::new()
                .keyid(false)
                .issuer(false)
                .build(&context)?
        };
        builder.append_extension(auth_key_id)?;
        
        // Sign with root CA
        builder.sign(&issuer.key, MessageDigest::sha256())?;
        let cert = builder.build();
        
        Ok(Certificate {
            cert,
            key,
            cert_type: CertificateType::IntermediateCa,
        })
    }
    
    /// Generate server certificate
    pub fn generate_server_cert(
        &self,
        cn: &str,
        dns_names: Vec<String>,
        ip_addresses: Vec<String>,
        ca_type: CAType,
    ) -> Result<Certificate> {
        let ca = match ca_type {
            CAType::Kubernetes => self.kubernetes_ca.as_ref(),
            CAType::Etcd => self.etcd_ca.as_ref(),
            CAType::FrontProxy => self.front_proxy_ca.as_ref(),
        }.ok_or_else(|| anyhow!("CA not initialized"))?;
        
        debug!("Generating server certificate: {}", cn);
        
        // Generate RSA key
        let rsa = Rsa::generate(self.config.key_size)?;
        let key = PKey::from_rsa(rsa)?;
        
        // Build certificate
        let mut builder = X509Builder::new()?;
        builder.set_version(2)?;
        
        // Set serial number
        let mut serial = BigNum::new()?;
        serial.rand(64, MsbOption::MAYBE_ZERO, false)?;
        let serial_asn1 = serial.to_asn1_integer()?;
        builder.set_serial_number(&serial_asn1)?;
        
        // Set subject
        let mut name_builder = X509NameBuilder::new()?;
        name_builder.append_entry_by_text("C", &self.config.country)?;
        name_builder.append_entry_by_text("O", &self.config.organization)?;
        name_builder.append_entry_by_text("CN", cn)?;
        let name = name_builder.build();
        builder.set_subject_name(&name)?;
        
        // Set issuer
        builder.set_issuer_name(ca.cert.subject_name())?;
        
        // Set validity period
        let not_before = Asn1Time::days_from_now(0)?;
        let validity_days = self.config.server_cert_validity_years.saturating_mul(365);
        let not_after = Asn1Time::days_from_now(validity_days)?;
        builder.set_not_before(&not_before)?;
        builder.set_not_after(&not_after)?;
        
        // Set public key
        builder.set_pubkey(&key)?;
        
        // Add extensions
        // Basic constraints: CA=false
        let basic_constraints = BasicConstraints::new()
            .critical()
            .build()?;
        builder.append_extension(basic_constraints)?;
        
        // Key usage
        let key_usage = KeyUsage::new()
            .critical()
            .digital_signature()
            .key_encipherment()
            .build()?;
        builder.append_extension(key_usage)?;
        
        // Subject Alternative Names (needs context)
        let context = builder.x509v3_context(Some(&ca.cert), None);
        let mut san = SubjectAlternativeName::new();
        for dns in dns_names {
            san.dns(&dns);
        }
        for ip in ip_addresses {
            san.ip(&ip);
        }
        let san_ext = san.build(&context)?;
        builder.append_extension(san_ext)?;
        
        // Sign with CA
        builder.sign(&ca.key, MessageDigest::sha256())?;
        let cert = builder.build();
        
        Ok(Certificate {
            cert,
            key,
            cert_type: CertificateType::Server,
        })
    }
    
    /// Get root CA
    pub fn get_root_ca(&self) -> Option<&Certificate> {
        self.root_ca.as_ref()
    }
    
    /// Get Kubernetes CA
    pub fn get_kubernetes_ca(&self) -> Option<&Certificate> {
        self.kubernetes_ca.as_ref()
    }
    
    /// Get etcd CA
    pub fn get_etcd_ca(&self) -> Option<&Certificate> {
        self.etcd_ca.as_ref()
    }
    
    /// Get front proxy CA
    pub fn get_front_proxy_ca(&self) -> Option<&Certificate> {
        self.front_proxy_ca.as_ref()
    }
    
    /// Get CA bundle (all CA certificates)
    pub fn get_ca_bundle(&self) -> Result<String> {
        let mut bundle = String::new();
        
        if let Some(root) = &self.root_ca {
            bundle.push_str(&String::from_utf8(root.cert.to_pem()?)?);
        }
        
        if let Some(k8s) = &self.kubernetes_ca {
            bundle.push_str(&String::from_utf8(k8s.cert.to_pem()?)?);
        }
        
        if let Some(etcd) = &self.etcd_ca {
            bundle.push_str(&String::from_utf8(etcd.cert.to_pem()?)?);
        }
        
        if let Some(proxy) = &self.front_proxy_ca {
            bundle.push_str(&String::from_utf8(proxy.cert.to_pem()?)?);
        }
        
        Ok(bundle)
    }
}

/// CA type for certificate generation
pub enum CAType {
    Kubernetes,
    Etcd,
    FrontProxy,
}