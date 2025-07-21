use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, patch, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
    time::Duration,
};
use tower_http::trace::TraceLayer;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
struct AppState {
    settings: Arc<RwLock<Settings>>,
    delay_ms: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mock_bottlerocket=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Get configuration from environment
    let delay_ms = std::env::var("MOCK_DELAY_MS")
        .unwrap_or_else(|_| "100".to_string())
        .parse::<u64>()
        .unwrap_or(100);

    // Initialize state with default settings
    let state = AppState {
        settings: Arc::new(RwLock::new(Settings::default())),
        delay_ms,
    };

    // Build router
    let app = Router::new()
        .route("/settings", get(get_settings).patch(patch_settings))
        .route("/os", get(get_os_info))
        .route("/actions/reboot", post(reboot))
        .route("/health", get(health))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("Mock Bottlerocket API listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn get_settings(State(state): State<AppState>) -> Result<Json<Settings>, StatusCode> {
    // Simulate API delay
    if state.delay_ms > 0 {
        tokio::time::sleep(Duration::from_millis(state.delay_ms)).await;
    }

    let settings = state.settings.read().unwrap();
    Ok(Json(settings.clone()))
}

async fn patch_settings(
    State(state): State<AppState>,
    Json(new_settings): Json<Settings>,
) -> Result<StatusCode, StatusCode> {
    // Simulate API delay
    if state.delay_ms > 0 {
        tokio::time::sleep(Duration::from_millis(state.delay_ms)).await;
    }

    info!("Applying new settings: {:?}", new_settings);

    let mut settings = state.settings.write().unwrap();
    
    // Merge settings (simplified - in real API this would be more sophisticated)
    if let Some(motd) = new_settings.motd {
        settings.motd = Some(motd);
    }
    if let Some(kubernetes) = new_settings.kubernetes {
        settings.kubernetes = Some(kubernetes);
    }
    if let Some(network) = new_settings.network {
        settings.network = Some(network);
    }
    if let Some(kernel) = new_settings.kernel {
        settings.kernel = Some(kernel);
    }
    if let Some(host_containers) = new_settings.host_containers {
        settings.host_containers = Some(host_containers);
    }
    if let Some(ntp) = new_settings.ntp {
        settings.ntp = Some(ntp);
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn get_os_info(State(state): State<AppState>) -> Json<OsInfo> {
    // Simulate API delay
    if state.delay_ms > 0 {
        tokio::time::sleep(Duration::from_millis(state.delay_ms)).await;
    }

    Json(OsInfo {
        arch: "x86_64".to_string(),
        build_id: "mock-build-12345".to_string(),
        pretty_name: "Bottlerocket OS 1.16.0 (aws-k8s-1.28)".to_string(),
        variant_id: "aws-k8s-1.28".to_string(),
        version_id: "1.16.0".to_string(),
    })
}

async fn reboot(State(state): State<AppState>) -> StatusCode {
    // Simulate API delay
    if state.delay_ms > 0 {
        tokio::time::sleep(Duration::from_millis(state.delay_ms)).await;
    }

    warn!("Mock reboot requested - not actually rebooting!");
    StatusCode::NO_CONTENT
}

async fn health() -> StatusCode {
    StatusCode::OK
}

// Settings structures (matching the real Bottlerocket API)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Settings {
    #[serde(skip_serializing_if = "Option::is_none")]
    motd: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    kubernetes: Option<KubernetesSettings>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    network: Option<NetworkSettings>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    kernel: Option<KernelSettings>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    host_containers: Option<HashMap<String, HostContainer>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    ntp: Option<NtpSettings>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            motd: Some("Welcome to Mock Bottlerocket".to_string()),
            kubernetes: Some(KubernetesSettings {
                api_server: Some("https://k8s.example.com:6443".to_string()),
                cluster_certificate: Some("LS0tLS1CRUdJTi...".to_string()),
                cluster_dns_ip: Some("10.96.0.10".to_string()),
                cluster_domain: Some("cluster.local".to_string()),
                node_labels: Some(HashMap::new()),
                node_taints: Some(HashMap::new()),
            }),
            network: Some(NetworkSettings {
                hostname: Some("mock-bottlerocket".to_string()),
                hosts: Some(HashMap::new()),
            }),
            kernel: Some(KernelSettings {
                lockdown: Some("integrity".to_string()),
                sysctl: Some(HashMap::new()),
            }),
            host_containers: Some(HashMap::new()),
            ntp: Some(NtpSettings {
                time_servers: Some(vec!["time.aws.com".to_string()]),
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KubernetesSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    api_server: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    cluster_certificate: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    cluster_dns_ip: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    cluster_domain: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    node_labels: Option<HashMap<String, String>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    node_taints: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NetworkSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    hostname: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    hosts: Option<HashMap<String, Vec<String>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KernelSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    lockdown: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    sysctl: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HostContainer {
    enabled: bool,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    superpowered: Option<bool>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    user_data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NtpSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    time_servers: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OsInfo {
    arch: String,
    build_id: String,
    pretty_name: String,
    variant_id: String,
    version_id: String,
}