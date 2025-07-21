use tonic::transport::Channel;
use tonic_health::pb::health_client::HealthClient;
use tonic_health::pb::health_server::{Health, HealthServer};
use tonic_health::pb::{HealthCheckRequest, HealthCheckResponse};
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};
use tracing::{debug, info};

/// Health check status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HealthStatus {
    Unknown,
    Serving,
    NotServing,
}

impl From<HealthStatus> for i32 {
    fn from(status: HealthStatus) -> Self {
        match status {
            HealthStatus::Unknown => 0,
            HealthStatus::Serving => 1,
            HealthStatus::NotServing => 2,
        }
    }
}

/// Health check service implementation
pub struct HealthService {
    status: Arc<RwLock<HealthStatus>>,
}

impl HealthService {
    pub fn new() -> Self {
        Self {
            status: Arc::new(RwLock::new(HealthStatus::Serving)),
        }
    }

    pub async fn set_status(&self, status: HealthStatus) {
        *self.status.write().await = status;
        info!("Health status changed to: {:?}", status);
    }

    pub async fn get_status(&self) -> HealthStatus {
        *self.status.read().await
    }
}

#[tonic::async_trait]
impl Health for HealthService {
    async fn check(
        &self,
        request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        let service = request.into_inner().service;
        debug!("Health check requested for service: {}", service);
        
        let status = self.get_status().await;
        
        let response = HealthCheckResponse {
            status: status.into(),
        };
        
        Ok(Response::new(response))
    }

    type WatchStream = tokio_stream::wrappers::ReceiverStream<Result<HealthCheckResponse, Status>>;

    async fn watch(
        &self,
        request: Request<HealthCheckRequest>,
    ) -> Result<Response<Self::WatchStream>, Status> {
        let service = request.into_inner().service;
        debug!("Health watch requested for service: {}", service);
        
        let (tx, rx) = tokio::sync::mpsc::channel(10);
        let status = self.status.clone();
        
        // Send initial status
        let initial_status = *status.read().await;
        let _ = tx.send(Ok(HealthCheckResponse {
            status: initial_status.into(),
        })).await;
        
        // Start watching for changes
        tokio::spawn(async move {
            let mut last_status = initial_status;
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                
                let current_status = *status.read().await;
                if current_status != last_status {
                    last_status = current_status;
                    if let Err(_) = tx.send(Ok(HealthCheckResponse {
                        status: current_status.into(),
                    })).await {
                        break;
                    }
                }
            }
        });
        
        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }
}

/// Create a health server for the gRPC service
pub fn create_health_service() -> (HealthServer<HealthService>, Arc<HealthService>) {
    let health_service = Arc::new(HealthService::new());
    let server = HealthServer::new(health_service.clone());
    (server, health_service)
}

/// Health checker for dependencies
pub struct HealthChecker {
    br_client: Arc<crate::bottlerocket::client::BottlerocketClient>,
    state_manager: Arc<crate::persistence::StateManager>,
}

impl HealthChecker {
    pub fn new(
        br_client: Arc<crate::bottlerocket::client::BottlerocketClient>,
        state_manager: Arc<crate::persistence::StateManager>,
    ) -> Self {
        Self {
            br_client,
            state_manager,
        }
    }

    /// Perform comprehensive health check
    pub async fn check_health(&self) -> (HealthStatus, Vec<String>) {
        let mut issues = Vec::new();
        let mut overall_status = HealthStatus::Serving;

        // Check Bottlerocket API connectivity
        match self.br_client.get_settings().await {
            Ok(_) => {
                debug!("Bottlerocket API: healthy");
            }
            Err(e) => {
                issues.push(format!("Bottlerocket API unavailable: {}", e));
                overall_status = HealthStatus::NotServing;
            }
        }

        // Check state persistence
        if let Err(e) = self.state_manager.health_check().await {
            issues.push(format!("State persistence unhealthy: {}", e));
            if overall_status == HealthStatus::Serving {
                overall_status = HealthStatus::Unknown;
            }
        }

        // Check event system
        if crate::events::EventSystem::get().is_none() {
            issues.push("Event system not initialized".to_string());
            if overall_status == HealthStatus::Serving {
                overall_status = HealthStatus::Unknown;
            }
        }

        (overall_status, issues)
    }

    /// Start periodic health checks
    pub async fn start_health_monitoring(
        self: Arc<Self>,
        health_service: Arc<HealthService>,
        interval_secs: u64,
    ) {
        info!("Starting health monitoring with {}s interval", interval_secs);
        
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval_secs));
        
        loop {
            interval.tick().await;
            
            let (status, issues) = self.check_health().await;
            
            if !issues.is_empty() {
                info!("Health check issues: {:?}", issues);
            }
            
            let current_status = health_service.get_status().await;
            if current_status != status {
                health_service.set_status(status).await;
                
                // Publish health status change event
                crate::events::publish_event(
                    crate::events::EventType::HealthStatusChanged,
                    crate::events::EventData::Health {
                        status: format!("{:?}", status),
                        checks: issues.iter().map(|issue| {
                            crate::events::HealthCheck {
                                name: "dependency".to_string(),
                                passed: false,
                                message: issue.clone(),
                            }
                        }).collect(),
                    },
                );
            }
        }
    }
}

/// Perform a simple health check against a gRPC endpoint
pub async fn check_grpc_health(endpoint: &str) -> anyhow::Result<bool> {
    let channel = Channel::from_shared(endpoint.to_string())?
        .connect()
        .await?;
    
    let mut client = HealthClient::new(channel);
    
    let request = Request::new(HealthCheckRequest {
        service: String::new(),
    });
    
    match client.check(request).await {
        Ok(response) => {
            let status = response.into_inner().status;
            Ok(status == 1) // Serving = 1
        }
        Err(e) => {
            debug!("Health check failed: {}", e);
            Ok(false)
        }
    }
}