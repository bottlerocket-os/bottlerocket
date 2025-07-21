use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use std::time::Duration;
use tonic::transport::Server;
use tracing::{info, error};

use platform_bootstrap::{
    election::{ElectionService, ElectionState, NodeInfo, ElectionConfig},
    pki::PKIService,
    etcd::EtcdService,
    coordinator::BootstrapCoordinator,
};

#[derive(Parser)]
#[command(name = "platform-bootstrap")]
#[command(about = "Bootstrap service for Platform Control Agent")]
struct Cli {
    /// Node ID (defaults to generated UUID)
    #[arg(long)]
    node_id: Option<String>,
    
    /// Bind address for gRPC server
    #[arg(long, default_value = "0.0.0.0:50100")]
    bind: String,
    
    /// Cluster members (comma-separated addresses)
    #[arg(long)]
    members: Option<String>,
    
    /// Election priority (0-1000)
    #[arg(long, default_value = "100")]
    priority: u64,
    
    /// Development mode (no TLS)
    #[arg(long)]
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
    
    // Create node info
    let node_info = NodeInfo {
        node_id: node_id.clone(),
        address: cli.bind.clone(),
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
        let member_info = NodeInfo {
            node_id: format!("node-{}", member_addr),
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
    let election_service = Arc::new(ElectionService::new(election_state));
    let pki_service = Arc::new(PKIService::new());
    let etcd_service = Arc::new(EtcdService::new());
    
    // Start election service background tasks
    if let Err(e) = election_service.start().await {
        error!("Failed to start election service: {}", e);
        return Err(e);
    }
    
    // Create bootstrap coordinator
    let coordinator = Arc::new(BootstrapCoordinator::new(
        election_service.clone(),
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
    
    // Build gRPC server
    let addr = cli.bind.parse()?;
    
    info!("Platform Bootstrap Service ready on {}", addr);
    
    Server::builder()
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