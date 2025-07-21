use serde::{Deserialize, Serialize};

/// Configuration for the reconciliation loop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconcilerConfig {
    /// Whether reconciliation is enabled
    pub enabled: bool,
    
    /// Interval between reconciliation checks (in seconds)
    pub interval_seconds: u64,
    
    /// Whether to automatically correct drift
    pub auto_correct: bool,
    
    /// Minimum severity level to trigger auto-correction
    pub correction_threshold: CorrectionThreshold,
    
    /// Fields to ignore during drift detection
    pub ignored_fields: Vec<String>,
    
    /// Fields that are critical and always trigger correction
    pub critical_fields: Vec<String>,
    
    /// Maximum number of correction attempts before backing off
    pub max_correction_attempts: u32,
    
    /// Backoff multiplier for failed corrections
    pub backoff_multiplier: f64,
}

impl Default for ReconcilerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_seconds: 300, // 5 minutes
            auto_correct: true,
            correction_threshold: CorrectionThreshold::Warning,
            ignored_fields: vec![
                // Fields that change frequently or are managed externally
                "motd".to_string(),
                "host_containers.admin.user_data".to_string(),
            ],
            critical_fields: vec![
                // Fields that must always match
                "kubernetes.api_server".to_string(),
                "kubernetes.cluster_certificate".to_string(),
                "network.hostname".to_string(),
            ],
            max_correction_attempts: 3,
            backoff_multiplier: 2.0,
        }
    }
}

/// Threshold for automatic correction
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CorrectionThreshold {
    /// Only correct critical drift
    Critical,
    /// Correct warning and critical drift
    Warning,
    /// Correct any drift
    Info,
}

impl ReconcilerConfig {
    /// Create a new reconciler config with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Load from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(val) = std::env::var("RECONCILER_ENABLED") {
            config.enabled = val.parse().unwrap_or(true);
        }

        if let Ok(val) = std::env::var("RECONCILER_INTERVAL") {
            if let Ok(seconds) = val.parse() {
                config.interval_seconds = seconds;
            }
        }

        if let Ok(val) = std::env::var("RECONCILER_AUTO_CORRECT") {
            config.auto_correct = val.parse().unwrap_or(true);
        }

        config
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.interval_seconds < 30 {
            return Err("Reconciliation interval must be at least 30 seconds".to_string());
        }

        if self.interval_seconds > 3600 {
            return Err("Reconciliation interval cannot exceed 1 hour".to_string());
        }

        if self.max_correction_attempts == 0 {
            return Err("Max correction attempts must be at least 1".to_string());
        }

        if self.backoff_multiplier < 1.0 {
            return Err("Backoff multiplier must be at least 1.0".to_string());
        }

        Ok(())
    }

    /// Check if a field should be ignored
    pub fn should_ignore_field(&self, field_path: &str) -> bool {
        self.ignored_fields.iter().any(|pattern| {
            field_path == pattern || field_path.starts_with(&format!("{}.", pattern))
        })
    }

    /// Check if a field is critical
    pub fn is_critical_field(&self, field_path: &str) -> bool {
        self.critical_fields.iter().any(|pattern| {
            field_path == pattern || field_path.starts_with(&format!("{}.", pattern))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ReconcilerConfig::default();
        assert!(config.enabled);
        assert_eq!(config.interval_seconds, 300);
        assert!(config.auto_correct);
    }

    #[test]
    fn test_config_validation() {
        let mut config = ReconcilerConfig::default();
        assert!(config.validate().is_ok());

        config.interval_seconds = 10;
        assert!(config.validate().is_err());

        config.interval_seconds = 5000;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_field_patterns() {
        let config = ReconcilerConfig::default();
        
        assert!(config.should_ignore_field("motd"));
        assert!(config.should_ignore_field("host_containers.admin.user_data"));
        assert!(!config.should_ignore_field("kubernetes.api_server"));
        
        assert!(config.is_critical_field("kubernetes.api_server"));
        assert!(config.is_critical_field("network.hostname"));
        assert!(!config.is_critical_field("motd"));
    }
}