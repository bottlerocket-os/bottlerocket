use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use std::sync::Arc;
use tonic::transport::Server;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod bottlerocket;
mod events;
mod persistence;
mod services;
mod system;
mod tls;

use services::machine_service::MachineServiceImpl;

#[derive(Parser)]
#[command(name = "platform-control")]
#[command(about = "Platform Control Agent for Bottlerocket", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the gRPC server
    Serve {
        /// Address to bind to
        #[arg(short, long, default_value = "0.0.0.0:50000")]
        bind: String,

        /// Enable development mode (disables TLS)
        #[arg(long)]
        dev_mode: bool,

        /// Path to TLS certificate
        #[arg(long)]
        tls_cert: Option<String>,

        /// Path to TLS key
        #[arg(long)]
        tls_key: Option<String>,

        /// Path to CA certificate for mTLS
        #[arg(long)]
        tls_ca: Option<String>,
    },

    /// Health check
    Health {
        /// Server address
        #[arg(short, long, default_value = "localhost:50000")]
        server: String,
    },

    /// Performance test
    PerfTest {
        /// Server address
        #[arg(short, long, default_value = "localhost:50000")]
        server: String,

        /// Number of requests
        #[arg(short, long, default_value = "1000")]
        requests: u32,

        /// Number of concurrent clients
        #[arg(short, long, default_value = "10")]
        clients: u32,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "platform_control=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Serve {
            bind,
            dev_mode,
            tls_cert,
            tls_key,
            tls_ca,
        } => {
            serve(bind, dev_mode, tls_cert, tls_key, tls_ca).await?;
        }
        Commands::Health { server } => {
            health_check(&server).await?;
        }
        Commands::PerfTest {
            server,
            requests,
            clients,
        } => {
            perf_test(&server, requests, clients).await?;
        }
    }

    Ok(())
}

async fn serve(
    bind: String,
    dev_mode: bool,
    tls_cert: Option<String>,
    tls_key: Option<String>,
    tls_ca: Option<String>,
) -> Result<()> {
    let addr: SocketAddr = bind.parse()?;

    info!("Starting Platform Control Agent on {}", addr);

    if dev_mode {
        warn!("Running in development mode - TLS disabled!");
    }

    // Initialize Bottlerocket API client
    let bottlerocket_url = std::env::var("BOTTLEROCKET_API_URL")
        .unwrap_or_else(|_| "unix:///run/api.sock".to_string());
    
    let br_client = bottlerocket::client::BottlerocketClient::new(&bottlerocket_url)?;

    // Initialize state manager
    let state_dir = std::env::var("PLATFORM_STATE_DIR").ok();
    let current_config = Arc::new(tokio::sync::RwLock::new(None));
    let state_manager = persistence::StateManager::new(state_dir.as_deref(), current_config)?;
    
    // Initialize event system
    events::EventSystem::init(state_dir.as_deref()).await
        .context("Failed to initialize event system")?;
    
    // Publish system startup event
    events::publish_event(
        events::EventType::SystemStartup,
        events::EventData::SystemLifecycle {
            action: "startup".to_string(),
            reason: Some("Platform Control Agent starting".to_string()),
        }
    );
    
    // Load saved configuration
    match state_manager.load_config().await {
        Ok(Some(config)) => {
            info!("Loaded saved configuration version: {}", config.version);
        }
        Ok(None) => {
            info!("No saved configuration found, starting fresh");
        }
        Err(e) => {
            warn!("Failed to load saved configuration: {}", e);
            if !dev_mode {
                return Err(anyhow::anyhow!("Failed to load configuration: {}", e));
            }
        }
    }

    // Create service implementation
    let machine_service = MachineServiceImpl::new(br_client, state_manager);

    // Build gRPC server
    let mut server_builder = Server::builder();

    // Configure TLS if not in dev mode
    if !dev_mode {
        if let (Some(cert), Some(key)) = (tls_cert, tls_key) {
            info!("Configuring TLS...");
            let tls_config = crate::tls::create_tls_config(&cert, &key, tls_ca.as_deref())?;
            server_builder = server_builder.tls_config(tls_config)?;
        } else {
            return Err(anyhow::anyhow!(
                "TLS certificate and key required when not in dev mode"
            ));
        }
    }

    // Add services
    let service = api::machine_service_server::MachineServiceServer::new(machine_service);

    info!("Platform Control Agent ready to serve requests");
    
    // Publish system ready event
    events::publish_event(
        events::EventType::SystemReady,
        events::EventData::SystemLifecycle {
            action: "ready".to_string(),
            reason: Some("All systems initialized".to_string()),
        }
    );

    // Start server
    server_builder
        .add_service(service)
        .serve(addr)
        .await?;

    Ok(())
}

async fn health_check(server: &str) -> Result<()> {
    info!("Performing health check on {}", server);
    
    // TODO: Implement actual gRPC health check
    // For now, just try to connect to the server
    use tonic::transport::Channel;
    use crate::api::machine_service_client::MachineServiceClient;
    
    let channel = Channel::from_shared(format!("http://{}", server))?
        .connect()
        .await
        .context("Failed to connect to gRPC server")?;
    
    let mut client = MachineServiceClient::new(channel);
    
    // Try to get status
    let request = tonic::Request::new(crate::api::GetStatusRequest {});
    match client.get_status(request).await {
        Ok(_) => {
            println!("✅ Health check passed - server is responding");
            Ok(())
        }
        Err(e) => {
            println!("❌ Health check failed: {}", e);
            Err(anyhow::anyhow!("Health check failed: {}", e))
        }
    }
}

async fn perf_test(server: &str, requests: u32, clients: u32) -> Result<()> {
    use std::time::Instant;
    use futures::future::join_all;
    use crate::api::machine_service_client::MachineServiceClient;
    use tonic::transport::Channel;
    
    info!(
        "Running performance test against {}: {} requests with {} clients",
        server, requests, clients
    );
    
    let start = Instant::now();
    let requests_per_client = requests / clients;
    
    // Create tasks for concurrent clients
    let mut tasks = Vec::new();
    for _client_id in 0..clients {
        let server = server.to_string();
        let task = tokio::spawn(async move {
            let channel = Channel::from_shared(format!("http://{}", server))
                .unwrap()
                .connect()
                .await
                .unwrap();
            
            let mut client = MachineServiceClient::new(channel);
            
            for _ in 0..requests_per_client {
                let request = tonic::Request::new(crate::api::GetStatusRequest {});
                let _ = client.get_status(request).await;
            }
        });
        tasks.push(task);
    }
    
    // Wait for all clients to complete
    join_all(tasks).await;
    
    let duration = start.elapsed();
    let total_requests = requests_per_client * clients;
    let requests_per_second = total_requests as f64 / duration.as_secs_f64();
    
    println!("Performance test completed:");
    println!("  Total requests: {}", total_requests);
    println!("  Duration: {:?}", duration);
    println!("  Requests/second: {:.2}", requests_per_second);
    
    Ok(())
}