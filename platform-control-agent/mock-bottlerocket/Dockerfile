# Mock Bottlerocket API Server
FROM rust:1.83-bookworm AS builder

WORKDIR /app

# Create a simple mock server
RUN cargo init --name mock-bottlerocket

# Create Cargo.toml with dependencies
RUN cat > Cargo.toml << 'EOF'
[package]
name = "mock-bottlerocket"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
tower = "0.4"
tower-http = { version = "0.6", features = ["trace", "cors"] }
EOF

# Create mock server
RUN cat > src/main.rs << 'INNEREOF'
use axum::{routing::{get, patch, post}, Json, Router};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, debug};
use tower_http::trace::TraceLayer;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Settings {
    #[serde(skip_serializing_if = "Option::is_none")]
    motd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    kubernetes: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    network: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OsInfo {
    arch: String,
    build_id: String,
    pretty_name: String,
    variant_id: String,
    version_id: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    let app = Router::new()
        .route("/settings", get(get_settings).patch(patch_settings))
        .route("/os", get(get_os_info))
        .route("/actions/reboot", post(reboot))
        .layer(TraceLayer::new_for_http());
    
    let addr = "0.0.0.0:8080";
    info!("Mock Bottlerocket API listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_settings() -> Json<Settings> {
    debug!("GET /settings");
    Json(Settings {
        motd: Some("Mock Bottlerocket".to_string()),
        kubernetes: None,
        network: None,
    })
}

async fn patch_settings(Json(settings): Json<Settings>) -> Json<HashMap<String, String>> {
    debug!("PATCH /settings: {:?}", settings);
    let mut response = HashMap::new();
    response.insert("status".to_string(), "success".to_string());
    Json(response)
}

async fn get_os_info() -> Json<OsInfo> {
    debug!("GET /os");
    Json(OsInfo {
        arch: "x86_64".to_string(),
        build_id: "mock-build-123".to_string(),
        pretty_name: "Mock Bottlerocket OS 1.16.0".to_string(),
        variant_id: "aws-k8s-1.28".to_string(),
        version_id: "1.16.0".to_string(),
    })
}

async fn reboot() -> Json<HashMap<String, String>> {
    debug!("POST /actions/reboot");
    let mut response = HashMap::new();
    response.insert("status".to_string(), "rebooting".to_string());
    Json(response)
}
INNEREOF

RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/mock-bottlerocket /usr/local/bin/

EXPOSE 8080
CMD ["/usr/local/bin/mock-bottlerocket"]
