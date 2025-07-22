use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, patch, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use tower::ServiceBuilder;

#[derive(Clone)]
struct AppState {
    settings: Arc<RwLock<Settings>>,
    staged_settings: Arc<RwLock<Option<Settings>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Settings {
    motd: Option<String>,
    kubernetes: Option<KubernetesSettings>,
    network: Option<NetworkSettings>,
    kernel: Option<KernelSettings>,
    host_containers: Option<HashMap<String, HostContainer>>,
    ntp: Option<NtpSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KubernetesSettings {
    api_server: Option<String>,
    cluster_certificate: Option<String>,
    cluster_dns_ip: Option<String>,
    cluster_domain: Option<String>,
    node_labels: Option<HashMap<String, String>>,
    node_taints: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NetworkSettings {
    hostname: Option<String>,
    hosts: Option<HashMap<String, Vec<String>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KernelSettings {
    lockdown: Option<String>,
    sysctl: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HostContainer {
    enabled: bool,
    source: Option<String>,
    superpowered: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NtpSettings {
    time_servers: Option<Vec<String>>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            motd: Some("Welcome to Mock Bottlerocket".to_string()),
            kubernetes: None,
            network: Some(NetworkSettings {
                hostname: Some("mock-bottlerocket".to_string()),
                hosts: None,
            }),
            kernel: Some(KernelSettings {
                lockdown: Some("none".to_string()),
                sysctl: Some(HashMap::new()),
            }),
            host_containers: Some(HashMap::new()),
            ntp: Some(NtpSettings {
                time_servers: Some(vec!["time.aws.com".to_string()]),
            }),
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mock_bottlerocket=debug,tower_http=debug".into()),
        )
        .init();

    let state = AppState {
        settings: Arc::new(RwLock::new(Settings::default())),
        staged_settings: Arc::new(RwLock::new(None)),
    };

    let app = Router::new()
        .route("/", get(health))
        .route("/health", get(health))
        .route("/settings", get(get_settings).patch(patch_settings))
        .route("/tx/commit", post(commit))
        .route("/tx/apply", post(apply))
        .route("/actions/reboot", post(reboot))
        .route("/os", get(get_os_info))
        .layer(
            ServiceBuilder::new()
                .layer(tower_http::trace::TraceLayer::new_for_http())
        )
        .with_state(state);

    // Try Unix socket first
    let socket_path = "/run/api.sock";
    match std::fs::remove_file(socket_path) {
        Ok(_) => info!("Removed existing socket"),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => warn!("Failed to remove socket: {}", e),
    }

    // Also listen on HTTP for development
    let http_server = {
        let app = app.clone();
        tokio::spawn(async move {
            let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
                .await
                .unwrap();
            info!("Mock Bottlerocket API listening on http://0.0.0.0:8080");
            axum::serve(listener, app).await.unwrap();
        })
    };

    // For now, just run HTTP server
    // Unix socket support would require hyper-util UnixListener setup
    info!("Mock Bottlerocket API listening on http://0.0.0.0:8080 (Unix socket disabled for now)");
    http_server.await.unwrap();
}

async fn health() -> &'static str {
    "OK"
}

async fn get_settings(State(state): State<AppState>) -> Json<Settings> {
    Json(state.settings.read().await.clone())
}

async fn patch_settings(
    State(state): State<AppState>,
    Json(patch): Json<serde_json::Value>,
) -> Result<StatusCode, StatusCode> {
    info!("Received settings patch: {}", patch);
    
    // In a real implementation, we would merge the patch with current settings
    // For the mock, we'll update staged settings
    let mut staged = state.staged_settings.write().await;
    let mut current = state.settings.read().await.clone();
    
    // Simple merge logic
    if let Some(obj) = patch.as_object() {
        if let Some(motd) = obj.get("motd") {
            current.motd = Some(motd.as_str().unwrap_or("").to_string());
        }
        if let Some(k8s) = obj.get("kubernetes") {
            current.kubernetes = serde_json::from_value(k8s.clone()).ok();
        }
        if let Some(net) = obj.get("network") {
            current.network = serde_json::from_value(net.clone()).ok();
        }
        if let Some(kernel) = obj.get("kernel") {
            current.kernel = serde_json::from_value(kernel.clone()).ok();
        }
    }
    
    *staged = Some(current);
    Ok(StatusCode::NO_CONTENT)
}

async fn commit(State(state): State<AppState>) -> Result<StatusCode, StatusCode> {
    info!("Committing settings");
    
    let staged = state.staged_settings.read().await.clone();
    if staged.is_none() {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    Ok(StatusCode::NO_CONTENT)
}

async fn apply(State(state): State<AppState>) -> Result<StatusCode, StatusCode> {
    info!("Applying settings");
    
    let mut staged = state.staged_settings.write().await;
    if let Some(new_settings) = staged.take() {
        *state.settings.write().await = new_settings;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

async fn reboot() -> StatusCode {
    info!("Reboot requested (mock - not actually rebooting)");
    StatusCode::ACCEPTED
}

#[derive(Serialize)]
struct OsInfo {
    version_id: String,
    build_id: String,
    arch: String,
    variant: String,
}

async fn get_os_info() -> Json<OsInfo> {
    Json(OsInfo {
        version_id: "1.19.0".to_string(),
        build_id: "mock-build".to_string(),
        arch: std::env::consts::ARCH.to_string(),
        variant: "aws-k8s-1.28".to_string(),
    })
}
