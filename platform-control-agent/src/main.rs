use anyhow::Result;
use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use tonic::transport::Server;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod bottlerocket;
mod services;

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

    // Create service implementation
    let machine_service = MachineServiceImpl::new(br_client);

    // Build gRPC server
    let mut server_builder = Server::builder();

    // Configure TLS if not in dev mode
    if !dev_mode {
        if let (Some(cert), Some(key)) = (tls_cert, tls_key) {
            info!("Configuring TLS...");
            // TODO: Implement TLS configuration
            // let tls_config = create_tls_config(&cert, &key, tls_ca.as_deref())?;
            // server_builder = server_builder.tls_config(tls_config)?;
        } else {
            return Err(anyhow::anyhow!(
                "TLS certificate and key required when not in dev mode"
            ));
        }
    }

    // Add services
    let service = api::machine_service_server::MachineServiceServer::new(machine_service);

    info!("Platform Control Agent ready to serve requests");

    // Start server
    server_builder
        .add_service(service)
        .serve(addr)
        .await?;

    Ok(())
}

async fn health_check(server: &str) -> Result<()> {
    info!("Performing health check on {}", server);
    
    // TODO: Implement actual health check
    println!("Health check passed");
    
    Ok(())
}

async fn perf_test(server: &str, requests: u32, clients: u32) -> Result<()> {
    info!(
        "Running performance test: {} requests with {} clients",
        requests, clients
    );
    
    // TODO: Implement performance test
    println!("Performance test completed");
    
    Ok(())
}