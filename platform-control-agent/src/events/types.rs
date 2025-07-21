use serde::{Deserialize, Serialize};
use std::fmt;

/// Event types supported by the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    // Configuration events
    ConfigurationApplied,
    ConfigurationValidationFailed,
    ConfigurationReset,
    
    // System lifecycle events
    SystemStartup,
    SystemShutdown,
    SystemReady,
    
    // Reboot events
    RebootScheduled,
    RebootInitiated,
    RebootCancelled,
    
    // Reset events
    ResetInitiated,
    ResetCompleted,
    
    // Upgrade events
    UpgradeStarted,
    UpgradeDownloading,
    UpgradeApplying,
    UpgradeCompleted,
    UpgradeFailed,
    UpgradeRollback,
    
    // Health events
    HealthCheckPassed,
    HealthCheckFailed,
    HealthStatusChanged,
    
    // Resource events
    ResourceThresholdWarning,
    ResourceThresholdCritical,
    ResourceThresholdRecovered,
    
    // Network events
    NetworkConnected,
    NetworkDisconnected,
    NetworkConfigChanged,
    
    // Service events
    ServiceStarted,
    ServiceStopped,
    ServiceFailed,
    ServiceRecovered,
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl EventType {
    /// Get all event types
    pub fn all() -> Vec<EventType> {
        vec![
            EventType::ConfigurationApplied,
            EventType::ConfigurationValidationFailed,
            EventType::ConfigurationReset,
            EventType::SystemStartup,
            EventType::SystemShutdown,
            EventType::SystemReady,
            EventType::RebootScheduled,
            EventType::RebootInitiated,
            EventType::RebootCancelled,
            EventType::ResetInitiated,
            EventType::ResetCompleted,
            EventType::UpgradeStarted,
            EventType::UpgradeDownloading,
            EventType::UpgradeApplying,
            EventType::UpgradeCompleted,
            EventType::UpgradeFailed,
            EventType::UpgradeRollback,
            EventType::HealthCheckPassed,
            EventType::HealthCheckFailed,
            EventType::HealthStatusChanged,
            EventType::ResourceThresholdWarning,
            EventType::ResourceThresholdCritical,
            EventType::ResourceThresholdRecovered,
            EventType::NetworkConnected,
            EventType::NetworkDisconnected,
            EventType::NetworkConfigChanged,
            EventType::ServiceStarted,
            EventType::ServiceStopped,
            EventType::ServiceFailed,
            EventType::ServiceRecovered,
        ]
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Option<EventType> {
        match s {
            "ConfigurationApplied" => Some(EventType::ConfigurationApplied),
            "ConfigurationValidationFailed" => Some(EventType::ConfigurationValidationFailed),
            "ConfigurationReset" => Some(EventType::ConfigurationReset),
            "SystemStartup" => Some(EventType::SystemStartup),
            "SystemShutdown" => Some(EventType::SystemShutdown),
            "SystemReady" => Some(EventType::SystemReady),
            "RebootScheduled" => Some(EventType::RebootScheduled),
            "RebootInitiated" => Some(EventType::RebootInitiated),
            "RebootCancelled" => Some(EventType::RebootCancelled),
            "ResetInitiated" => Some(EventType::ResetInitiated),
            "ResetCompleted" => Some(EventType::ResetCompleted),
            "UpgradeStarted" => Some(EventType::UpgradeStarted),
            "UpgradeDownloading" => Some(EventType::UpgradeDownloading),
            "UpgradeApplying" => Some(EventType::UpgradeApplying),
            "UpgradeCompleted" => Some(EventType::UpgradeCompleted),
            "UpgradeFailed" => Some(EventType::UpgradeFailed),
            "UpgradeRollback" => Some(EventType::UpgradeRollback),
            "HealthCheckPassed" => Some(EventType::HealthCheckPassed),
            "HealthCheckFailed" => Some(EventType::HealthCheckFailed),
            "HealthStatusChanged" => Some(EventType::HealthStatusChanged),
            "ResourceThresholdWarning" => Some(EventType::ResourceThresholdWarning),
            "ResourceThresholdCritical" => Some(EventType::ResourceThresholdCritical),
            "ResourceThresholdRecovered" => Some(EventType::ResourceThresholdRecovered),
            "NetworkConnected" => Some(EventType::NetworkConnected),
            "NetworkDisconnected" => Some(EventType::NetworkDisconnected),
            "NetworkConfigChanged" => Some(EventType::NetworkConfigChanged),
            "ServiceStarted" => Some(EventType::ServiceStarted),
            "ServiceStopped" => Some(EventType::ServiceStopped),
            "ServiceFailed" => Some(EventType::ServiceFailed),
            "ServiceRecovered" => Some(EventType::ServiceRecovered),
            _ => None,
        }
    }
}

/// Event-specific data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventData {
    /// Configuration was applied
    ConfigurationApplied {
        version: String,
        machine_type: String,
        cluster_name: Option<String>,
    },
    
    /// Configuration validation failed
    ConfigurationValidationFailed {
        errors: Vec<String>,
    },
    
    /// System lifecycle event
    SystemLifecycle {
        action: String,
        reason: Option<String>,
    },
    
    /// Reboot event
    Reboot {
        graceful: bool,
        scheduled_time: Option<i64>,
        reason: Option<String>,
    },
    
    /// Reset event
    Reset {
        graceful: bool,
        cleared_items: Vec<String>,
    },
    
    /// Upgrade event
    Upgrade {
        current_version: String,
        target_version: String,
        status: String,
        progress: Option<u8>,
        error: Option<String>,
    },
    
    /// Health check event
    Health {
        status: String,
        checks: Vec<HealthCheck>,
    },
    
    /// Resource threshold event
    ResourceThreshold {
        resource_type: String,
        current_value: f64,
        threshold_value: f64,
        unit: String,
    },
    
    /// Network event
    Network {
        interface: Option<String>,
        state: String,
        details: Option<String>,
    },
    
    /// Service event
    Service {
        name: String,
        state: String,
        error: Option<String>,
    },
    
    /// Generic event data
    Generic {
        message: String,
        details: HashMap<String, String>,
    },
}

use std::collections::HashMap;

impl EventData {
    /// Get a human-readable message for the event
    pub fn message(&self) -> String {
        match self {
            EventData::ConfigurationApplied { version, machine_type, cluster_name } => {
                if let Some(cluster) = cluster_name {
                    format!("Configuration {} applied for {} node in cluster {}", version, machine_type, cluster)
                } else {
                    format!("Configuration {} applied for {} node", version, machine_type)
                }
            }
            EventData::ConfigurationValidationFailed { errors } => {
                format!("Configuration validation failed: {}", errors.join(", "))
            }
            EventData::SystemLifecycle { action, reason } => {
                if let Some(r) = reason {
                    format!("System {}: {}", action, r)
                } else {
                    format!("System {}", action)
                }
            }
            EventData::Reboot { graceful, scheduled_time, reason } => {
                let mut msg = if *graceful { "Graceful reboot" } else { "Immediate reboot" }.to_string();
                if let Some(time) = scheduled_time {
                    msg.push_str(&format!(" scheduled for {}", time));
                }
                if let Some(r) = reason {
                    msg.push_str(&format!(": {}", r));
                }
                msg
            }
            EventData::Reset { graceful, cleared_items } => {
                format!(
                    "{} reset completed, cleared: {}", 
                    if *graceful { "Graceful" } else { "Immediate" },
                    cleared_items.join(", ")
                )
            }
            EventData::Upgrade { current_version, target_version, status, progress, error } => {
                let mut msg = format!("Upgrade from {} to {}: {}", current_version, target_version, status);
                if let Some(p) = progress {
                    msg.push_str(&format!(" ({}%)", p));
                }
                if let Some(e) = error {
                    msg.push_str(&format!(" - Error: {}", e));
                }
                msg
            }
            EventData::Health { status, checks } => {
                let failed = checks.iter().filter(|c| !c.passed).count();
                if failed > 0 {
                    format!("Health check {}: {} of {} checks failed", status, failed, checks.len())
                } else {
                    format!("Health check {}: all {} checks passed", status, checks.len())
                }
            }
            EventData::ResourceThreshold { resource_type, current_value, threshold_value, unit } => {
                format!(
                    "{} threshold exceeded: {:.1}{} (threshold: {:.1}{})", 
                    resource_type, current_value, unit, threshold_value, unit
                )
            }
            EventData::Network { interface, state, details } => {
                let mut msg = if let Some(iface) = interface {
                    format!("Network interface {} is {}", iface, state)
                } else {
                    format!("Network is {}", state)
                };
                if let Some(d) = details {
                    msg.push_str(&format!(": {}", d));
                }
                msg
            }
            EventData::Service { name, state, error } => {
                let mut msg = format!("Service {} is {}", name, state);
                if let Some(e) = error {
                    msg.push_str(&format!(": {}", e));
                }
                msg
            }
            EventData::Generic { message, .. } => message.clone(),
        }
    }
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub name: String,
    pub passed: bool,
    pub message: String,
}