use crate::api::{
    machine_service_server::MachineService, *
};
use crate::bottlerocket::client::{BottlerocketClient, Settings};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};
use tracing::{debug, error, info, warn};

pub struct MachineServiceImpl {
    br_client: Arc<BottlerocketClient>,
    current_config: Arc<RwLock<Option<MachineConfig>>>,
}

impl MachineServiceImpl {
    pub fn new(br_client: BottlerocketClient) -> Self {
        Self {
            br_client: Arc::new(br_client),
            current_config: Arc::new(RwLock::new(None)),
        }
    }

    /// Translate MachineConfig to Bottlerocket Settings
    fn translate_to_settings(&self, config: &MachineConfig) -> Result<Settings> {
        let mut settings = Settings::default();

        // Translate cluster settings
        if let Some(cluster) = &config.cluster {
            let mut k8s = crate::bottlerocket::client::KubernetesSettings {
                api_server: Some(cluster.endpoint.clone()),
                cluster_certificate: Some(cluster.ca_certificate.clone()),
                cluster_dns_ip: Some(cluster.dns_ip.clone()),
                cluster_domain: Some(cluster.dns_domain.clone()),
                node_labels: None,
                node_taints: None,
            };
            settings.kubernetes = Some(k8s);
        }

        // Translate network settings
        if let Some(network) = &config.network {
            let mut net = crate::bottlerocket::client::NetworkSettings {
                hostname: Some(network.hostname.clone()),
                hosts: None,
            };
            settings.network = Some(net);
        }

        // Translate security settings
        if let Some(security) = &config.security {
            let mut kernel = crate::bottlerocket::client::KernelSettings {
                lockdown: Some(security.lockdown_mode.clone()),
                sysctl: Some(security.kernel_parameters.clone()),
            };
            settings.kernel = Some(kernel);
        }

        // Translate host containers
        if !config.host_containers.is_empty() {
            let mut containers = std::collections::HashMap::new();
            for (name, container) in &config.host_containers {
                let hc = crate::bottlerocket::client::HostContainer {
                    enabled: container.enabled,
                    source: Some(container.source.clone()),
                    superpowered: Some(container.superpowered),
                    user_data: if container.user_data.is_empty() { 
                        None 
                    } else { 
                        Some(container.user_data.clone()) 
                    },
                };
                containers.insert(name.clone(), hc);
            }
            settings.host_containers = Some(containers);
        }

        Ok(settings)
    }

    /// Validate machine configuration
    fn validate_config(&self, config: &MachineConfig) -> Vec<ConfigValidationError> {
        let mut errors = Vec::new();

        // Validate required fields
        if config.version.is_empty() {
            errors.push(ConfigValidationError {
                field: "version".to_string(),
                message: "Version is required".to_string(),
            });
        }

        // Validate cluster configuration
        if let Some(cluster) = &config.cluster {
            if cluster.endpoint.is_empty() {
                errors.push(ConfigValidationError {
                    field: "cluster.endpoint".to_string(),
                    message: "Cluster endpoint is required".to_string(),
                });
            }
            if cluster.ca_certificate.is_empty() {
                errors.push(ConfigValidationError {
                    field: "cluster.ca_certificate".to_string(),
                    message: "Cluster CA certificate is required".to_string(),
                });
            }
        }

        // Validate FIPS mode if enabled
        if let Some(security) = &config.security {
            if security.fips_enabled && security.lockdown_mode != "integrity" {
                errors.push(ConfigValidationError {
                    field: "security.lockdown_mode".to_string(),
                    message: "Lockdown mode must be 'integrity' when FIPS is enabled".to_string(),
                });
            }
        }

        errors
    }
}

#[tonic::async_trait]
impl MachineService for MachineServiceImpl {
    async fn apply_configuration(
        &self,
        request: Request<MachineConfigRequest>,
    ) -> Result<Response<MachineConfigResponse>, Status> {
        let req = request.into_inner();
        info!("Applying machine configuration");

        // Validate configuration
        let errors = self.validate_config(&req.config.as_ref().unwrap());
        if !errors.is_empty() {
            warn!("Configuration validation failed: {:?}", errors);
            return Ok(Response::new(MachineConfigResponse {
                success: false,
                message: "Configuration validation failed".to_string(),
                errors,
            }));
        }

        let config = req.config.unwrap();

        // Translate to Bottlerocket settings
        let settings = self
            .translate_to_settings(&config)
            .map_err(|e| Status::internal(format!("Failed to translate config: {}", e)))?;

        // Apply settings if not dry run
        if !req.dry_run {
            self.br_client
                .set_settings(&settings)
                .await
                .map_err(|e| Status::internal(format!("Failed to apply settings: {}", e)))?;

            // Update stored configuration
            let mut stored = self.current_config.write().await;
            *stored = Some(config);
        }

        Ok(Response::new(MachineConfigResponse {
            success: true,
            message: if req.dry_run {
                "Configuration validated successfully (dry run)".to_string()
            } else {
                "Configuration applied successfully".to_string()
            },
            errors: vec![],
        }))
    }

    async fn get_configuration(
        &self,
        _request: Request<GetConfigRequest>,
    ) -> Result<Response<MachineConfig>, Status> {
        debug!("Getting current machine configuration");

        let stored = self.current_config.read().await;
        if let Some(config) = stored.as_ref() {
            Ok(Response::new(config.clone()))
        } else {
            Err(Status::not_found("No configuration has been applied yet"))
        }
    }

    async fn get_status(
        &self,
        _request: Request<GetStatusRequest>,
    ) -> Result<Response<MachineStatus>, Status> {
        debug!("Getting machine status");

        // Get OS info from Bottlerocket
        let os_info = self.br_client
            .get_os_info()
            .await
            .map_err(|e| Status::internal(format!("Failed to get OS info: {}", e)))?;

        // Build status response
        let status = MachineStatus {
            node_id: "platform-node-001".to_string(), // TODO: Get actual node ID
            r#type: MachineType::ControlPlane as i32,
            os_version: os_info.version_id,
            kubernetes_version: "1.28.5".to_string(), // TODO: Get from settings
            system: Some(machine_status::System {
                hostname: "platform-node".to_string(), // TODO: Get actual hostname
                uptime_seconds: 3600, // TODO: Get actual uptime
                boot_id: os_info.build_id.clone(),
                machine_id: "".to_string(), // TODO: Get machine ID
                kernel_version: "".to_string(), // TODO: Get kernel version
            }),
            resources: Some(machine_status::Resources {
                cpu_cores: 4, // TODO: Get actual resources
                memory_bytes: 8 * 1024 * 1024 * 1024,
                disk_bytes: 100 * 1024 * 1024 * 1024,
            }),
            ready: true,
            conditions: vec![
                Condition {
                    r#type: "Ready".to_string(),
                    status: "True".to_string(),
                    reason: "NodeReady".to_string(),
                    message: "Node is ready".to_string(),
                    last_transition_time: 0,
                },
            ],
        };

        Ok(Response::new(status))
    }

    async fn reset(
        &self,
        request: Request<ResetRequest>,
    ) -> Result<Response<ResetResponse>, Status> {
        let req = request.into_inner();
        info!("Resetting machine (graceful: {})", req.graceful);

        // TODO: Implement actual reset logic
        // This would involve:
        // 1. Drain node if graceful
        // 2. Reset Bottlerocket settings to defaults
        // 3. Clear stored configuration

        Ok(Response::new(ResetResponse {
            success: true,
            message: "Machine reset initiated".to_string(),
        }))
    }

    async fn reboot(
        &self,
        request: Request<RebootRequest>,
    ) -> Result<Response<RebootResponse>, Status> {
        let req = request.into_inner();
        info!("Rebooting machine (graceful: {})", req.graceful);

        // TODO: If graceful, drain node first

        // Initiate reboot
        self.br_client
            .reboot()
            .await
            .map_err(|e| Status::internal(format!("Failed to reboot: {}", e)))?;

        Ok(Response::new(RebootResponse {
            success: true,
            message: "Reboot initiated".to_string(),
            scheduled_time: chrono::Utc::now().timestamp(),
        }))
    }

    async fn upgrade(
        &self,
        request: Request<UpgradeRequest>,
    ) -> Result<Response<UpgradeResponse>, Status> {
        let req = request.into_inner();
        info!("Upgrading to version: {}", req.target_version);

        // TODO: Implement actual upgrade logic
        // This would involve:
        // 1. Validate target version
        // 2. Download new image
        // 3. Apply update
        // 4. Reboot into new version

        Ok(Response::new(UpgradeResponse {
            success: true,
            message: "Upgrade initiated".to_string(),
            current_version: "1.16.0".to_string(), // TODO: Get actual version
            target_version: req.target_version,
        }))
    }

    type StreamEventsStream = tokio_stream::wrappers::ReceiverStream<Result<MachineEvent, Status>>;

    async fn stream_events(
        &self,
        request: Request<StreamEventsRequest>,
    ) -> Result<Response<Self::StreamEventsStream>, Status> {
        let req = request.into_inner();
        info!("Starting event stream for types: {:?}", req.event_types);

        let (tx, rx) = tokio::sync::mpsc::channel(128);

        // TODO: Implement actual event streaming
        // For now, send a test event
        tokio::spawn(async move {
            let event = MachineEvent {
                id: uuid::Uuid::new_v4().to_string(),
                r#type: "ConfigurationApplied".to_string(),
                timestamp: chrono::Utc::now().timestamp(),
                message: "Configuration applied successfully".to_string(),
                metadata: std::collections::HashMap::new(),
            };
            let _ = tx.send(Ok(event)).await;
        });

        Ok(Response::new(
            tokio_stream::wrappers::ReceiverStream::new(rx)
        ))
    }
}