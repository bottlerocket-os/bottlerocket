use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

pub mod config;
pub mod diff;

use crate::api::MachineConfig;
use crate::bottlerocket::client::BottlerocketClient;
use crate::events::{publish_event, EventData, EventType};
use crate::persistence::StateManager;

pub use config::ReconcilerConfig;
use diff::{ConfigDiff, DriftSeverity};

/// Configuration reconciler that detects and corrects drift
pub struct ConfigReconciler {
    /// Bottlerocket API client
    br_client: Arc<BottlerocketClient>,
    /// State manager for desired configuration
    state_manager: Arc<StateManager>,
    /// Reconciler configuration
    config: ReconcilerConfig,
    /// Flag to stop reconciliation
    shutdown: Arc<RwLock<bool>>,
    /// Last reconciliation status
    last_status: Arc<RwLock<ReconciliationStatus>>,
}

/// Status of the last reconciliation
#[derive(Debug, Clone)]
pub struct ReconciliationStatus {
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub drift_detected: bool,
    pub drift_corrected: bool,
    pub error: Option<String>,
}

impl Default for ReconciliationStatus {
    fn default() -> Self {
        Self {
            last_check: chrono::Utc::now(),
            drift_detected: false,
            drift_corrected: false,
            error: None,
        }
    }
}

impl ConfigReconciler {
    /// Create a new configuration reconciler
    pub fn new(
        br_client: Arc<BottlerocketClient>,
        state_manager: Arc<StateManager>,
        config: ReconcilerConfig,
    ) -> Self {
        Self {
            br_client,
            state_manager,
            config,
            shutdown: Arc::new(RwLock::new(false)),
            last_status: Arc::new(RwLock::new(ReconciliationStatus::default())),
        }
    }

    /// Start the reconciliation loop
    pub async fn start_reconciliation_loop(self: Arc<Self>) {
        if !self.config.enabled {
            info!("Reconciliation loop is disabled");
            return;
        }

        info!(
            "Starting reconciliation loop with interval: {}s",
            self.config.interval_seconds
        );

        // Publish startup event
        publish_event(
            EventType::ReconciliationStarted,
            EventData::Generic {
                message: format!(
                    "Reconciliation loop started (interval: {}s, auto_correct: {})",
                    self.config.interval_seconds, self.config.auto_correct
                ),
                details: std::collections::HashMap::new(),
            },
        );

        let mut interval = interval(Duration::from_secs(self.config.interval_seconds));
        
        // Add some jitter to avoid thundering herd
        tokio::time::sleep(Duration::from_secs(5)).await;

        loop {
            interval.tick().await;

            // Check if we should shutdown
            if *self.shutdown.read().await {
                info!("Reconciliation loop shutting down");
                break;
            }

            // Perform reconciliation
            match self.reconcile_once().await {
                Ok(()) => {
                    debug!("Reconciliation completed successfully");
                }
                Err(e) => {
                    error!("Reconciliation failed: {}", e);
                    
                    // Update status
                    let mut status = self.last_status.write().await;
                    status.error = Some(e.to_string());
                    
                    // Publish failure event
                    publish_event(
                        EventType::ReconciliationFailed,
                        EventData::Generic {
                            message: format!("Reconciliation failed: {}", e),
                            details: std::collections::HashMap::new(),
                        },
                    );
                }
            }
        }
    }

    /// Perform a single reconciliation
    async fn reconcile_once(&self) -> Result<()> {
        debug!("Starting reconciliation check");

        // Get desired configuration
        let desired_config = {
            let config = self.state_manager.current_config.read().await;
            match config.as_ref() {
                Some(c) => c.clone(),
                None => {
                    debug!("No desired configuration set, skipping reconciliation");
                    return Ok(());
                }
            }
        };

        // Get actual settings from Bottlerocket
        let actual_settings = match self.br_client.get_settings().await {
            Ok(settings) => settings,
            Err(e) => {
                // If we can't get settings (e.g., in dev mode), skip reconciliation
                debug!("Cannot fetch actual settings: {}", e);
                return Ok(());
            }
        };

        // Translate desired config to settings format for comparison
        let desired_settings = self.translate_config_to_settings(&desired_config)?;

        // Compare configurations
        let mut diff = diff::compare_settings(&desired_settings, &actual_settings);
        
        // Filter out ignored fields
        diff.drifts.retain(|drift| !self.config.should_ignore_field(&drift.field_path));
        
        // Mark critical fields with higher severity
        for drift in &mut diff.drifts {
            if self.config.is_critical_field(&drift.field_path) {
                drift.severity = diff::DriftSeverity::Critical;
            }
        }
        
        // Recalculate severity after filtering
        if !diff.drifts.is_empty() {
            diff.severity = diff.drifts
                .iter()
                .map(|d| d.severity)
                .max()
                .unwrap_or(diff::DriftSeverity::Info);
        }

        // Update status
        {
            let mut status = self.last_status.write().await;
            status.last_check = chrono::Utc::now();
            status.drift_detected = diff.has_drift();
            status.drift_corrected = false;
            status.error = None;
        }

        // Handle drift if detected
        if diff.has_drift() {
            self.handle_drift(&desired_config, &diff).await?;
        } else {
            debug!("No configuration drift detected");
        }

        // Publish completion event
        publish_event(
            EventType::ReconciliationCompleted,
            EventData::Generic {
                message: format!(
                    "Reconciliation completed (drift_detected: {}, auto_corrected: {})",
                    diff.has_drift(),
                    diff.has_drift() && self.config.auto_correct
                ),
                details: std::collections::HashMap::new(),
            },
        );

        Ok(())
    }

    /// Handle detected configuration drift
    async fn handle_drift(&self, desired_config: &MachineConfig, diff: &ConfigDiff) -> Result<()> {
        warn!("Configuration drift detected: {}", diff.summary());

        // Publish drift detection event
        let mut details = std::collections::HashMap::new();
        details.insert("severity".to_string(), diff.severity.to_string());
        details.insert("drift_count".to_string(), diff.drifts.len().to_string());
        
        publish_event(
            EventType::ConfigurationDriftDetected,
            EventData::Generic {
                message: diff.summary(),
                details,
            },
        );

        // Check if we should auto-correct
        if !self.config.auto_correct {
            info!("Auto-correction is disabled, drift will not be corrected");
            return Ok(());
        }

        // Check severity threshold
        let threshold_severity = match self.config.correction_threshold {
            config::CorrectionThreshold::Critical => diff::DriftSeverity::Critical,
            config::CorrectionThreshold::Warning => diff::DriftSeverity::Warning,
            config::CorrectionThreshold::Info => diff::DriftSeverity::Info,
        };
        
        if diff.severity < threshold_severity {
            debug!("Drift severity {} is below correction threshold {}", 
                diff.severity, threshold_severity);
            return Ok(());
        }

        // Apply correction
        info!("Applying configuration to correct drift");
        
        let settings = self.translate_config_to_settings(desired_config)?;
        self.br_client.set_settings(&settings).await?;

        // Update status
        {
            let mut status = self.last_status.write().await;
            status.drift_corrected = true;
        }

        // Publish correction event
        publish_event(
            EventType::ConfigurationDriftCorrected,
            EventData::Generic {
                message: "Configuration drift automatically corrected".to_string(),
                details: std::collections::HashMap::new(),
            },
        );

        Ok(())
    }

    /// Translate MachineConfig to Bottlerocket Settings
    fn translate_config_to_settings(
        &self,
        config: &MachineConfig,
    ) -> Result<crate::bottlerocket::client::Settings> {
        let mut settings = crate::bottlerocket::client::Settings::default();

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

        Ok(settings)
    }

    /// Stop the reconciliation loop
    pub async fn stop(&self) {
        info!("Stopping reconciliation loop");
        *self.shutdown.write().await = true;
    }

    /// Get the current reconciliation status
    pub async fn get_status(&self) -> ReconciliationStatus {
        self.last_status.read().await.clone()
    }
}

#[cfg(test)]
mod tests;