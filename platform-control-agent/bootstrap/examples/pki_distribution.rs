use anyhow::Result;
use tracing::{info, error};
use std::sync::Arc;

use platform_bootstrap::{
    pki::{PKIConfig, CertificateAuthority, CertificateStore, PKIDistributor},
    election::{ElectionState, ElectionConfig, NodeInfo},
    proto::pki::{CertificateRequest, CertificateType},
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("PKI Distribution Example");
    
    // Create election state (mock as leader for demo)
    let node_info = NodeInfo {
        node_id: "leader-node".to_string(),
        address: "127.0.0.1:50100".to_string(),
        uptime: Duration::from_secs(300),
        cpu_available_percent: 80.0,
        memory_available_gb: 8.0,
        packet_loss_percent: 0.0,
        election_priority: 100,
    };
    let election_config = ElectionConfig::default();
    let election_state = Arc::new(ElectionState::new("leader-node".to_string(), node_info, election_config));
    
    // Create PKI components
    let store = Arc::new(CertificateStore::new());
    let distributor = Arc::new(PKIDistributor::new(store.clone(), election_state.clone()));
    
    // Create CA and generate hierarchy
    info!("Generating PKI hierarchy...");
    let pki_config = PKIConfig::default();
    let mut ca = CertificateAuthority::new(pki_config);
    ca.initialize()?;
    
    // Store CAs
    info!("Storing CA certificates...");
    store.store_root_ca(ca.get_root_ca().unwrap().clone()).await?;
    if let Some(k8s_ca) = ca.get_kubernetes_ca() {
        store.store_kubernetes_ca(k8s_ca.clone()).await?;
    }
    if let Some(etcd_ca) = ca.get_etcd_ca() {
        store.store_etcd_ca(etcd_ca.clone()).await?;
    }
    
    // Register auth tokens for demo nodes
    info!("Registering node authentication tokens...");
    distributor.register_auth_token("node-1".to_string(), "secret-token-1".to_string()).await?;
    distributor.register_auth_token("node-2".to_string(), "secret-token-2".to_string()).await?;
    
    // Simulate certificate request from node-1
    info!("Simulating certificate request from node-1...");
    let request = CertificateRequest {
        common_name: "node-1.cluster.local".to_string(),
        r#type: CertificateType::Server as i32,
        dns_names: vec![
            "node-1.cluster.local".to_string(),
            "node-1".to_string(),
        ],
        ip_addresses: vec!["10.0.0.100".to_string()],
        email_addresses: vec![],
        validity_days: 365,
        organizations: vec!["Platform Kubernetes".to_string()],
        organizational_units: vec!["Nodes".to_string()],
        node_id: "node-1".to_string(),
        csr: vec![], // Empty CSR means generate new key pair
        auth_token: "secret-token-1".to_string(),
    };
    
    // Note: This will fail because we're not actually the leader
    // In a real implementation, only the elected leader can issue certificates
    match distributor.process_certificate_request(request).await {
        Ok(response) => {
            info!("Certificate issued successfully!");
            if let Some(cert) = response.certificate {
                info!("  Common Name: {}", cert.common_name);
                info!("  Fingerprint: {}", cert.fingerprint);
                info!("  DNS Names: {:?}", cert.dns_names);
                info!("  Renewal Time: {}", response.next_renewal_time);
            }
        }
        Err(e) => {
            error!("Failed to issue certificate: {}", e);
            info!("This is expected in the demo - only elected leaders can issue certificates");
        }
    }
    
    // Demonstrate certificate validation
    info!("\nDemonstrating certificate validation...");
    let root_ca = ca.get_root_ca().unwrap();
    let (cert_pem, _) = root_ca.to_pem()?;
    
    match distributor.validate_certificate_chain(&cert_pem).await {
        Ok(_) => info!("Certificate chain validation successful!"),
        Err(e) => error!("Certificate chain validation failed: {}", e),
    }
    
    // Check for certificates needing renewal
    info!("\nChecking for certificates needing renewal...");
    let expiring = distributor.check_renewals().await?;
    info!("Certificates needing renewal: {}", expiring.len());
    
    Ok(())
}