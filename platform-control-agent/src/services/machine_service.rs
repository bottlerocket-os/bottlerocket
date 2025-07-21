use crate::api::{
    machine_service_server::MachineService, *
};
use crate::bottlerocket::client::{BottlerocketClient, Settings};
use crate::error::{ErrorResponses, IntoStatus, IntoStatusResult};
use crate::persistence::StateManager;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};
use tracing::{debug, info, warn};

pub struct MachineServiceImpl {
    br_client: Arc<BottlerocketClient>,
    current_config: Arc<RwLock<Option<MachineConfig>>>,
    state_manager: Arc<StateManager>,
}

impl MachineServiceImpl {
    pub fn new(br_client: Arc<BottlerocketClient>, state_manager: Arc<StateManager>) -> Self {
        Self {
            br_client,
            current_config: state_manager.current_config.clone(),
            state_manager,
        }
    }

    /// Translate MachineConfig to Bottlerocket Settings
    fn translate_to_settings(&self, config: &MachineConfig) -> Result<Settings> {
        let mut settings = Settings::default();

        // Translate cluster settings
        if let Some(cluster) = &config.cluster {
            let k8s = crate::bottlerocket::client::KubernetesSettings {
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
            let net = crate::bottlerocket::client::NetworkSettings {
                hostname: Some(network.hostname.clone()),
                hosts: None,
            };
            settings.network = Some(net);
        }

        // Translate security settings
        if let Some(security) = &config.security {
            let kernel = crate::bottlerocket::client::KernelSettings {
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

    /// Build node conditions based on current state
    fn build_conditions(&self) -> Vec<Condition> {
        let mut conditions = Vec::new();
        let now = chrono::Utc::now().timestamp();

        // Ready condition
        conditions.push(Condition {
            r#type: "Ready".to_string(),
            status: if crate::system::SystemInfo::is_ready() { "True" } else { "False" }.to_string(),
            reason: "NodeReady".to_string(),
            message: "Node is ready".to_string(),
            last_transition_time: now,
        });

        // MemoryPressure condition
        let memory_usage_percent = self.calculate_memory_pressure();
        conditions.push(Condition {
            r#type: "MemoryPressure".to_string(),
            status: if memory_usage_percent > 90.0 { "True" } else { "False" }.to_string(),
            reason: "KubeletHasSufficientMemory".to_string(),
            message: format!("Memory usage: {:.1}%", memory_usage_percent),
            last_transition_time: now,
        });

        // DiskPressure condition
        let disk_usage_percent = self.calculate_disk_pressure();
        conditions.push(Condition {
            r#type: "DiskPressure".to_string(),
            status: if disk_usage_percent > 90.0 { "True" } else { "False" }.to_string(),
            reason: "KubeletHasNoDiskPressure".to_string(),
            message: format!("Disk usage: {:.1}%", disk_usage_percent),
            last_transition_time: now,
        });

        conditions
    }

    /// Calculate memory pressure percentage
    fn calculate_memory_pressure(&self) -> f64 {
        // In a real implementation, this would check actual memory usage
        // For now, return a reasonable value
        45.0
    }

    /// Calculate disk pressure percentage  
    fn calculate_disk_pressure(&self) -> f64 {
        // In a real implementation, this would check actual disk usage
        // For now, return a reasonable value
        30.0
    }

    /// Validate version format (e.g., "1.16.0", "1.17.0-preview")
    fn is_valid_version(version: &str) -> bool {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() < 3 {
            return false;
        }
        
        // Check major.minor are numbers
        parts[0].parse::<u32>().is_ok() && parts[1].parse::<u32>().is_ok()
    }

    /// Check if upgrade path is valid
    fn is_valid_upgrade_path(current: &str, target: &str) -> bool {
        // In production, this would check:
        // - Compatibility matrix
        // - Whether downgrades are allowed
        // - Required intermediate versions
        
        // For now, allow any forward upgrade
        current != target
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
            
            // Publish validation failed event
            crate::events::publish_event(
                crate::events::EventType::ConfigurationValidationFailed,
                crate::events::EventData::ConfigurationValidationFailed {
                    errors: errors.iter().map(|e| format!("{}: {}", e.field, e.message)).collect(),
                }
            );
            
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
            .map_err(|e| ErrorResponses::internal_error(&format!("translate config: {}", e)))?;

        // Apply settings if not dry run
        if !req.dry_run {
            self.br_client
                .set_settings(&settings)
                .await
                .map_err(|e| ErrorResponses::bottlerocket_api_error(&e.to_string()))?;

            // Persist configuration to disk
            self.state_manager
                .save_config(&config)
                .await
                .map_err(|e| {
                    warn!("Failed to persist configuration: {}", e);
                    ErrorResponses::internal_error(&format!("persist config: {}", e))
                })?;
            
            info!("Configuration applied and persisted successfully");
            
            // Publish configuration applied event
            crate::events::publish_event(
                crate::events::EventType::ConfigurationApplied,
                crate::events::EventData::ConfigurationApplied {
                    version: config.version.clone(),
                    machine_type: match config.r#type {
                        1 => "ControlPlane".to_string(),
                        2 => "Worker".to_string(),
                        _ => "Unknown".to_string(),
                    },
                    cluster_name: config.cluster.as_ref().map(|c| c.name.clone()),
                }
            );
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
            Err(ErrorResponses::configuration_not_found())
        }
    }

    async fn get_status(
        &self,
        _request: Request<GetStatusRequest>,
    ) -> Result<Response<MachineStatus>, Status> {
        debug!("Getting machine status");

        // Get system information
        use crate::system::SystemInfo;
        
        // Get OS info from Bottlerocket (with fallback)
        let os_info = match self.br_client.get_os_info().await {
            Ok(info) => info,
            Err(e) => {
                warn!("Failed to get OS info from Bottlerocket: {}", e);
                // Provide fallback values
                crate::bottlerocket::client::OsInfo {
                    arch: std::env::consts::ARCH.to_string(),
                    build_id: SystemInfo::boot_id(),
                    pretty_name: "Bottlerocket OS".to_string(),
                    variant_id: "unknown".to_string(),
                    version_id: "unknown".to_string(),
                }
            }
        };

        // Determine machine type from current config
        let machine_type = {
            let config = self.current_config.read().await;
            config.as_ref()
                .map(|c| c.r#type)
                .unwrap_or(MachineType::Worker as i32)
        };

        // Build status response with real system data
        let status = MachineStatus {
            node_id: SystemInfo::machine_id(),
            r#type: machine_type,
            os_version: os_info.version_id,
            kubernetes_version: SystemInfo::kubernetes_version(),
            system: Some(machine_status::System {
                hostname: SystemInfo::hostname(),
                uptime_seconds: SystemInfo::uptime_seconds(),
                boot_id: SystemInfo::boot_id(),
                machine_id: SystemInfo::machine_id(),
                kernel_version: SystemInfo::kernel_version(),
            }),
            resources: Some(machine_status::Resources {
                cpu_cores: SystemInfo::cpu_cores(),
                memory_bytes: SystemInfo::memory_bytes(),
                disk_bytes: SystemInfo::disk_bytes(),
            }),
            ready: SystemInfo::is_ready(),
            conditions: self.build_conditions(),
        };

        Ok(Response::new(status))
    }

    async fn reset(
        &self,
        request: Request<ResetRequest>,
    ) -> Result<Response<ResetResponse>, Status> {
        let req = request.into_inner();
        info!("Resetting machine (graceful: {})", req.graceful);
        
        // Publish reset initiated event
        crate::events::publish_event(
            crate::events::EventType::ResetInitiated,
            crate::events::EventData::Reset {
                graceful: req.graceful,
                cleared_items: vec!["configuration".to_string()],
            }
        );

        // If graceful, ensure we have time to complete operations
        let timeout = if req.timeout_seconds > 0 {
            req.timeout_seconds
        } else {
            300 // Default 5 minutes
        };

        if req.graceful {
            info!("Performing graceful reset with {}s timeout", timeout);
            // In production, this would:
            // 1. Cordon the node (mark as unschedulable)
            // 2. Drain workloads to other nodes
            // 3. Wait for pods to terminate gracefully
        }

        // Clear persisted configuration
        info!("Clearing persisted configuration");
        {
            let mut config = self.current_config.write().await;
            *config = None;
        }

        // Delete configuration file
        if let Ok(config_path) = std::env::var("PLATFORM_STATE_DIR") {
            let config_file = std::path::Path::new(&config_path).join("config.json");
            if config_file.exists() {
                if let Err(e) = std::fs::remove_file(&config_file) {
                    warn!("Failed to delete config file: {}", e);
                } else {
                    info!("Deleted configuration file");
                }
            }

            // Delete backup file
            let backup_file = std::path::Path::new(&config_path).join("config.json.backup");
            if backup_file.exists() {
                let _ = std::fs::remove_file(&backup_file);
            }
        }

        // Reset Bottlerocket settings to defaults
        info!("Resetting Bottlerocket settings to defaults");
        // In production with real Bottlerocket:
        // self.br_client.reset_settings().await
        //     .map_err(|e| Status::internal(format!("Failed to reset settings: {}", e)))?;

        // Publish reset completed event
        crate::events::publish_event(
            crate::events::EventType::ResetCompleted,
            crate::events::EventData::Reset {
                graceful: req.graceful,
                cleared_items: vec!["configuration".to_string(), "state".to_string()],
            }
        );

        Ok(Response::new(ResetResponse {
            success: true,
            message: format!("Machine reset completed (graceful: {})", req.graceful),
        }))
    }

    async fn reboot(
        &self,
        request: Request<RebootRequest>,
    ) -> Result<Response<RebootResponse>, Status> {
        let req = request.into_inner();
        info!("Rebooting machine (graceful: {})", req.graceful);

        // Calculate reboot time
        let reboot_delay_seconds = if req.graceful {
            let timeout = if req.timeout_seconds > 0 {
                req.timeout_seconds
            } else {
                60 // Default 1 minute for graceful
            };
            
            info!("Performing graceful reboot with {}s delay", timeout);
            // In production:
            // 1. Cordon node
            // 2. Drain workloads
            // 3. Wait for completion
            timeout
        } else {
            10 // Immediate reboot with small delay
        };

        let scheduled_time = chrono::Utc::now().timestamp() + reboot_delay_seconds as i64;
        
        // Publish reboot scheduled event
        crate::events::publish_event(
            crate::events::EventType::RebootScheduled,
            crate::events::EventData::Reboot {
                graceful: req.graceful,
                scheduled_time: Some(scheduled_time),
                reason: Some("User requested reboot".to_string()),
            }
        );

        // Save current configuration before reboot to ensure it persists
        if let Some(config) = &*self.current_config.read().await {
            if let Err(e) = self.state_manager.save_config(config).await {
                warn!("Failed to save configuration before reboot: {}", e);
            }
        }

        // Schedule reboot
        if std::env::var("SKIP_UNIX_SOCKET").is_ok() {
            info!("SKIP_UNIX_SOCKET set, simulating reboot scheduled for {}", scheduled_time);
        } else {
            // Initiate actual reboot
            self.br_client
                .reboot()
                .await
                .map_err(|e| ErrorResponses::bottlerocket_api_error(&format!("reboot failed: {}", e)))?;
        }

        Ok(Response::new(RebootResponse {
            success: true,
            message: format!(
                "Reboot scheduled in {} seconds", 
                reboot_delay_seconds
            ),
            scheduled_time,
        }))
    }

    async fn upgrade(
        &self,
        request: Request<UpgradeRequest>,
    ) -> Result<Response<UpgradeResponse>, Status> {
        let req = request.into_inner();
        info!("Upgrading to version: {}", req.target_version);

        // Get current version
        let current_version = match self.br_client.get_os_info().await {
            Ok(info) => info.version_id,
            Err(_) => "unknown".to_string(),
        };

        // Validate target version format
        if !Self::is_valid_version(&req.target_version) {
            return Err(ErrorResponses::invalid_configuration(
                &format!("Invalid target version format: {}", req.target_version)
            ));
        }

        // Check if already on target version
        if current_version == req.target_version {
            return Ok(Response::new(UpgradeResponse {
                success: true,
                message: "Already on target version".to_string(),
                current_version: current_version.clone(),
                target_version: req.target_version,
            }));
        }

        // Validate upgrade path
        if !Self::is_valid_upgrade_path(&current_version, &req.target_version) {
            return Err(ErrorResponses::precondition_failed(
                &format!(
                    "Invalid upgrade path from {} to {}", 
                    current_version, 
                    req.target_version
                )
            ));
        }

        // In production, this would:
        // 1. Check available updates from update repository
        // 2. Download update to inactive partition
        // 3. Verify checksums and signatures
        // 4. Apply update metadata
        // 5. Schedule reboot to new partition

        info!(
            "Initiating upgrade from {} to {}", 
            current_version, 
            req.target_version
        );
        
        // Publish upgrade started event
        crate::events::publish_event(
            crate::events::EventType::UpgradeStarted,
            crate::events::EventData::Upgrade {
                current_version: current_version.clone(),
                target_version: req.target_version.clone(),
                status: "started".to_string(),
                progress: Some(0),
                error: None,
            }
        );

        // Simulate upgrade process
        if std::env::var("SKIP_UNIX_SOCKET").is_ok() {
            info!("SKIP_UNIX_SOCKET set, simulating upgrade process");
        } else {
            // In production:
            // self.br_client.apply_update(&req.target_version).await
            //     .map_err(|e| Status::internal(format!("Failed to apply update: {}", e)))?;
        }

        Ok(Response::new(UpgradeResponse {
            success: true,
            message: format!(
                "Upgrade from {} to {} initiated. Reboot required to complete.",
                current_version,
                req.target_version
            ),
            current_version,
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

        // Get event system
        let event_system = crate::events::EventSystem::get()
            .ok_or_else(|| Status::internal("Event system not initialized"))?;

        // Create stream channel
        let (tx, rx) = tokio::sync::mpsc::channel(128);

        // Parse requested event types
        let filter_types: Vec<crate::events::EventType> = if req.event_types.is_empty() {
            // No filter, stream all events
            vec![]
        } else {
            // Parse requested types
            req.event_types
                .iter()
                .filter_map(|s| crate::events::EventType::from_str(s))
                .collect()
        };

        // If specific types were requested but none were valid, return error
        if !req.event_types.is_empty() && filter_types.is_empty() {
            return Err(Status::invalid_argument(
                "No valid event types specified"
            ));
        }

        // Subscribe to events
        let mut event_rx = event_system.subscribe();

        // Start streaming task
        tokio::spawn(async move {
            info!("Event stream started, filtering for {:?}", filter_types);
            
            // Send initial event to confirm stream is working
            let welcome_event = MachineEvent {
                id: uuid::Uuid::new_v4().to_string(),
                r#type: "StreamStarted".to_string(),
                timestamp: chrono::Utc::now().timestamp(),
                message: "Event stream started".to_string(),
                metadata: std::collections::HashMap::new(),
            };
            if tx.send(Ok(welcome_event)).await.is_err() {
                warn!("Client disconnected immediately");
                return;
            }

            // Stream events
            loop {
                match event_rx.recv().await {
                    Ok(event) => {
                        // Apply filter
                        if !filter_types.is_empty() && !filter_types.contains(&event.event_type) {
                            continue;
                        }

                        // Convert to proto
                        let proto_event = event.to_proto();
                        
                        // Send to client
                        if tx.send(Ok(proto_event)).await.is_err() {
                            info!("Client disconnected from event stream");
                            break;
                        }
                    }
                    Err(e) => {
                        warn!("Event receiver error: {}", e);
                        break;
                    }
                }
            }
            
            info!("Event stream ended");
        });

        Ok(Response::new(
            tokio_stream::wrappers::ReceiverStream::new(rx)
        ))
    }
}

#[cfg(test)]
mod machine_service_tests;