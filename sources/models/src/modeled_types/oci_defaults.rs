use scalar_derive::Scalar;
use serde::{Deserialize, Serialize};

/// OciDefaultsCapability specifies which process capabilities are
/// allowed to be set in the default OCI spec.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Scalar, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OciDefaultsCapability {
    AuditControl,
    AuditRead,
    AuditWrite,
    BlockSuspend,
    Bpf,
    CheckpointRestore,
    Chown,
    DacOverride,
    DacReadSearch,
    Fowner,
    Fsetid,
    IpcLock,
    IpcOwner,
    Kill,
    Lease,
    LinuxImmutable,
    MacAdmin,
    MacOverride,
    Mknod,
    NetAdmin,
    NetBindService,
    NetBroadcast,
    NetRaw,
    Perfmon,
    Setgid,
    Setfcap,
    Setpcap,
    Setuid,
    SysAdmin,
    SysBoot,
    SysChroot,
    SysModule,
    SysNice,
    SysPacct,
    SysPtrace,
    SysRawio,
    SysResource,
    SysTime,
    SysTtyConfig,
    Syslog,
    WakeAlarm,
}

impl OciDefaultsCapability {
    /// Converts from Bottlerocket's kabob-case name into the Linux capability name, e.g. turns
    /// `wake-alarm` into `CAP_WAKE_ALARM`.
    pub fn to_linux_string(&self) -> String {
        format!("CAP_{}", self.to_string().to_uppercase().replace('-', "_"))
    }
}

#[cfg(test)]
mod oci_defaults_capabilities {
    use super::*;

    fn check_capability_strings(cap: OciDefaultsCapability, bottlerocket: &str, linux: &str) {
        let actual_bottlerocket = cap.to_string();
        let actual_linux = cap.to_linux_string();
        assert_eq!(bottlerocket, actual_bottlerocket);
        assert_eq!(linux, actual_linux);
    }

    #[test]
    fn linux_capability_strings() {
        check_capability_strings(
            OciDefaultsCapability::AuditControl,
            "audit-control",
            "CAP_AUDIT_CONTROL",
        );

        check_capability_strings(
            OciDefaultsCapability::SysPacct,
            "sys-pacct",
            "CAP_SYS_PACCT",
        );

        check_capability_strings(OciDefaultsCapability::Mknod, "mknod", "CAP_MKNOD");
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// OciDefaultsResourceLimitType specifies which resource limits are
/// allowed to be set in the default OCI spec.

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Scalar, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OciDefaultsResourceLimitType {
    MaxOpenFiles,
}

// We are leaving this implementation open for the easy addition of
// future resource limits.
impl OciDefaultsResourceLimitType {
    pub fn to_linux_string(&self) -> &'static str {
        match self {
            OciDefaultsResourceLimitType::MaxOpenFiles => "RLIMIT_NOFILE",
        }
    }
}

#[cfg(test)]
mod oci_defaults_rlimits {
    use super::*;

    fn check_rlimit_strings(cap: OciDefaultsResourceLimitType, bottlerocket: &str, linux: &str) {
        let actual_bottlerocket = cap.to_string();
        let actual_linux = cap.to_linux_string();
        assert_eq!(bottlerocket, actual_bottlerocket);
        assert_eq!(linux, actual_linux);
    }

    #[test]
    fn linux_rlimit_strings() {
        check_rlimit_strings(
            OciDefaultsResourceLimitType::MaxOpenFiles,
            "max-open-files",
            "RLIMIT_NOFILE",
        );
    }
}
