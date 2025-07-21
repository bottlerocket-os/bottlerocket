use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use tonic::transport::{Certificate, Identity, ServerTlsConfig};
use tracing::info;

/// Create TLS configuration for the gRPC server
pub fn create_tls_config(
    cert_path: &str,
    key_path: &str,
    ca_path: Option<&str>,
) -> Result<ServerTlsConfig> {
    info!("Loading TLS certificates...");
    
    // Load server certificate and key
    let cert = load_file(cert_path)
        .with_context(|| format!("Failed to load server certificate from {}", cert_path))?;
    let key = load_file(key_path)
        .with_context(|| format!("Failed to load server key from {}", key_path))?;
    
    // Create server identity
    let identity = Identity::from_pem(cert, key);
    
    // Start building TLS config
    let mut tls_config = ServerTlsConfig::new().identity(identity);
    
    // If CA certificate is provided, enable mTLS (mutual TLS)
    if let Some(ca_path) = ca_path {
        info!("Enabling mTLS with client certificate verification");
        let ca_cert = load_file(ca_path)
            .with_context(|| format!("Failed to load CA certificate from {}", ca_path))?;
        let ca = Certificate::from_pem(ca_cert);
        tls_config = tls_config.client_ca_root(ca);
    }
    
    info!("TLS configuration created successfully");
    Ok(tls_config)
}

/// Load a file as bytes
fn load_file(path: &str) -> Result<Vec<u8>> {
    let path = Path::new(path);
    if !path.exists() {
        return Err(anyhow::anyhow!("File does not exist: {}", path.display()));
    }
    
    fs::read(path).with_context(|| format!("Failed to read file: {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_load_file() {
        // Create a temporary file
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = b"test content";
        temp_file.write_all(content).unwrap();
        
        // Test loading the file
        let loaded = load_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(loaded, content);
    }
    
    #[test]
    fn test_load_file_not_exists() {
        let result = load_file("/tmp/nonexistent/file.txt");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }
    
    #[test]
    fn test_create_tls_config_missing_cert() {
        let result = create_tls_config(
            "/tmp/nonexistent/cert.pem",
            "/tmp/nonexistent/key.pem",
            None,
        );
        assert!(result.is_err());
    }
}