use std::fs;
use tracing::warn;

/// System information collector
pub struct SystemInfo;

impl SystemInfo {
    /// Get hostname
    pub fn hostname() -> String {
        match hostname::get() {
            Ok(name) => name.to_string_lossy().to_string(),
            Err(e) => {
                warn!("Failed to get hostname: {}", e);
                "unknown".to_string()
            }
        }
    }

    /// Get system uptime in seconds
    pub fn uptime_seconds() -> i64 {
        #[cfg(target_os = "linux")]
        {
            if let Ok(contents) = fs::read_to_string("/proc/uptime") {
                if let Some(uptime_str) = contents.split_whitespace().next() {
                    if let Ok(uptime) = uptime_str.parse::<f64>() {
                        return uptime as i64;
                    }
                }
            }
        }
        
        // Fallback for non-Linux or if reading fails
        match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(duration) => duration.as_secs() as i64,
            Err(_) => 0,
        }
    }

    /// Get boot ID (Linux only)
    pub fn boot_id() -> String {
        #[cfg(target_os = "linux")]
        {
            if let Ok(boot_id) = fs::read_to_string("/proc/sys/kernel/random/boot_id") {
                return boot_id.trim().to_string();
            }
        }
        
        // Fallback: use a generated UUID
        uuid::Uuid::new_v4().to_string()
    }

    /// Get machine ID
    pub fn machine_id() -> String {
        // Try standard locations
        let paths = [
            "/etc/machine-id",
            "/var/lib/dbus/machine-id",
        ];
        
        for path in &paths {
            if let Ok(id) = fs::read_to_string(path) {
                let trimmed = id.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }
        
        // Fallback: generate a stable ID based on hostname
        let hostname = Self::hostname();
        format!("{:x}", md5::compute(hostname.as_bytes()))
    }

    /// Get kernel version
    pub fn kernel_version() -> String {
        #[cfg(target_os = "linux")]
        {
            if let Ok(sys_info) = nix::sys::utsname::uname() {
                return format!(
                    "{} {}",
                    sys_info.sysname().to_string_lossy(),
                    sys_info.release().to_string_lossy()
                );
            }
        }
        
        // Fallback
        format!("{} {}", std::env::consts::OS, std::env::consts::ARCH)
    }

    /// Get CPU core count
    pub fn cpu_cores() -> u32 {
        num_cpus::get() as u32
    }

    /// Get total memory in bytes
    pub fn memory_bytes() -> u64 {
        #[cfg(target_os = "linux")]
        {
            if let Ok(contents) = fs::read_to_string("/proc/meminfo") {
                for line in contents.lines() {
                    if line.starts_with("MemTotal:") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            if let Ok(kb) = parts[1].parse::<u64>() {
                                return kb * 1024; // Convert KB to bytes
                            }
                        }
                    }
                }
            }
        }
        
        // Fallback: use sysinfo crate
        use sysinfo::System;
        let mut sys = System::new_all();
        sys.refresh_memory();
        sys.total_memory()
    }

    /// Get total disk space in bytes
    pub fn disk_bytes() -> u64 {
        #[cfg(target_os = "linux")]
        {
            // Get root filesystem size
            if let Ok(metadata) = fs::metadata("/") {
                if let Ok(stat) = nix::sys::statvfs::statvfs("/") {
                    return stat.blocks() * stat.block_size();
                }
            }
        }
        
        // Fallback
        100 * 1024 * 1024 * 1024 // 100GB default
    }

    /// Check if the system is ready
    pub fn is_ready() -> bool {
        // Basic readiness checks
        // In production, this would check:
        // - Critical services are running
        // - Network connectivity
        // - Configuration is applied
        // - No critical errors
        
        true // For now, always ready
    }

    /// Get Kubernetes version (if installed)
    pub fn kubernetes_version() -> String {
        // Try to get kubelet version
        if let Ok(output) = std::process::Command::new("kubelet")
            .arg("--version")
            .output()
        {
            let version_str = String::from_utf8_lossy(&output.stdout);
            if let Some(version) = version_str.split_whitespace().nth(1) {
                return version.trim_start_matches('v').to_string();
            }
        }
        
        // Default version
        "1.28.5".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hostname() {
        let hostname = SystemInfo::hostname();
        assert!(!hostname.is_empty());
    }

    #[test]
    fn test_cpu_cores() {
        let cores = SystemInfo::cpu_cores();
        assert!(cores > 0);
    }

    #[test]
    fn test_memory_bytes() {
        let memory = SystemInfo::memory_bytes();
        assert!(memory > 0);
    }

    #[test]
    fn test_kernel_version() {
        let kernel = SystemInfo::kernel_version();
        assert!(!kernel.is_empty());
    }
}