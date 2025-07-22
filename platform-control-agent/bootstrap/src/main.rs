use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use std::time::Duration;
use tonic::transport::Server;
use tracing::{info, error};

use platform_bootstrap::{
    election::{ElectionService, ElectionState, ElectionConfig, NodeInfo},
    pki::PKIService,
    etcd::EtcdService,
    coordinator::BootstrapCoordinator,
};

#[derive(Parser)]
#[command(name = "platform-bootstrap")]
#[command(about = "Bootstrap service for Platform Control Agent")]
struct Cli {
    /// Node ID (defaults to generated UUID)
    #[arg(long, env = "NODE_ID")]
    node_id: Option<String>,
    
    /// Bind address for gRPC server
    #[arg(long, env = "BIND_ADDR", default_value = "0.0.0.0:50100")]
    bind: String,
    
    /// Cluster members (comma-separated addresses)
    #[arg(long, env = "CLUSTER_MEMBERS")]
    members: Option<String>,
    
    /// Election priority (0-1000)
    #[arg(long, env = "NODE_PRIORITY", default_value = "100")]
    priority: u64,
    
    /// Development mode (no TLS)
    #[arg(long, env = "DEV_MODE")]
    dev_mode: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "platform_bootstrap=debug".into()),
        )
        .json()
        .init();

    let cli = Cli::parse();
    
    info!("Platform Bootstrap Service starting...");
    
    // Generate node ID if not provided
    let node_id = cli.node_id.unwrap_or_else(|| {
        let id = uuid::Uuid::new_v4().to_string();
        info!("Generated node ID: {}", id);
        id
    });
    
    // Parse cluster members
    let mut cluster_members = vec![];
    if let Some(members_str) = cli.members {
        cluster_members = members_str.split(',').map(|s| s.trim().to_string()).collect();
    }
    
    // Extract IP address from bind address for node info
    let bind_ip = if cli.dev_mode {
        // In dev mode, use 127.0.0.1 for local testing
        "127.0.0.1".to_string()
    } else {
        // Extract IP from bind address (remove port)
        cli.bind.split(':').next().unwrap_or("0.0.0.0").to_string()
    };
    
    // Create node info
    let node_info = NodeInfo {
        node_id: node_id.clone(),
        address: bind_ip,
        uptime: Duration::from_secs(0),
        cpu_available_percent: 80.0, // TODO: Get real metrics
        memory_available_gb: 8.0,
        packet_loss_percent: 0.0,
        election_priority: cli.priority,
    };
    
    // Create election state
    let election_config = ElectionConfig::default();
    let election_state = Arc::new(ElectionState::new(
        node_id.clone(),
        node_info,
        election_config,
    ));
    
    // Add cluster members
    for member_addr in cluster_members {
        // Extract just the hostname part for node_id (before the colon)
        let node_id = member_addr.split(':').next().unwrap_or(&member_addr).to_string();
        let member_info = NodeInfo {
            node_id,
            address: member_addr,
            uptime: Duration::from_secs(0),
            cpu_available_percent: 50.0,
            memory_available_gb: 4.0,
            packet_loss_percent: 0.0,
            election_priority: 100,
        };
        election_state.update_member(member_info).await;
    }
    
    // Initialize services
    let election_service = Arc::new(ElectionService::new(election_state.clone()));
    let pki_service = Arc::new(PKIService::new(election_state.clone()));
    let etcd_service = Arc::new(EtcdService::with_dev_mode(
        election_state.clone(),
        pki_service.clone(),
        cli.dev_mode,
    ));
    
    // Start election service background tasks
    if let Err(e) = election_service.start().await {
        error!("Failed to start election service: {}", e);
        return Err(e);
    }
    
    // Create bootstrap coordinator
    let coordinator = Arc::new(BootstrapCoordinator::new(
        election_service.clone(),
        election_state.clone(),
        pki_service.clone(),
        etcd_service.clone(),
    ));
    
    // Start bootstrap process in background
    let coordinator_clone = coordinator.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(5)).await;
        if let Err(e) = coordinator_clone.start_bootstrap().await {
            error!("Bootstrap failed: {}", e);
        }
    });
    
    // Build gRPC server with optional TLS
    let addr = cli.bind.parse()?;
    
    let mut server_builder = Server::builder();
    
    // Try to load TLS certificates - if they exist, use TLS; otherwise, use plaintext
    if let (Ok(cert), Ok(key), Ok(ca_cert)) = (
        std::fs::read("/etc/platform/certs/tls.crt"),
        std::fs::read("/etc/platform/certs/tls.key"),
        std::fs::read("/etc/platform/certs/ca.crt")
    ) {
        let server_identity = tonic::transport::Identity::from_pem(cert, key);
        let ca_cert = tonic::transport::Certificate::from_pem(ca_cert);
        
        let tls_config = tonic::transport::ServerTlsConfig::new()
            .identity(server_identity)
            .client_ca_root(ca_cert);
        
        server_builder = server_builder.tls_config(tls_config)?;
        info!("Platform Bootstrap Service ready on {} (TLS enabled)", addr);
    } else {
        info!("Platform Bootstrap Service ready on {} (plaintext - no TLS certificates found)", addr);
    }
    
    server_builder
        .add_service(
            platform_bootstrap::proto::election::election_service_server::ElectionServiceServer::from_arc(
                election_service
            )
        )
        .add_service(
            platform_bootstrap::proto::pki::pki_service_server::PkiServiceServer::from_arc(
                pki_service
            )
        )
        .add_service(
            platform_bootstrap::proto::etcd::etcd_service_server::EtcdServiceServer::from_arc(
                etcd_service
            )
        )
        .serve_with_shutdown(addr, async {
            tokio::signal::ctrl_c().await.ok();
            info!("Platform Bootstrap Service shutting down");
        })
        .await?;

    Ok(())
}