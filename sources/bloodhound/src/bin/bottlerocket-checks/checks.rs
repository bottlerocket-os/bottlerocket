use bloodhound::results::{CheckStatus, Checker, CheckerMetadata, CheckerResult, Mode};
use bloodhound::*;
use std::process::Command;

const PROC_MODULES_FILE: &str = "/proc/modules";
const PROC_CMDLINE_FILE: &str = "/proc/cmdline";
const LOCKDOWN_FILE: &str = "/sys/kernel/security/lockdown";
const CHRONY_CONF_FILE: &str = "/etc/chrony.conf";
const SYSCTL_CMD: &str = "/usr/sbin/sysctl";
const SYSTEMCTL_CMD: &str = "/usr/bin/systemctl";
const MODPROBE_CMD: &str = "/bin/modprobe";
const SESTATUS_CMD: &str = "/usr/bin/sestatus";

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct BR01010101Checker {}

impl Checker for BR01010101Checker {
    fn execute(&self) -> CheckerResult {
        let module_result = check_file_contains!(
            PROC_MODULES_FILE,
            &[" udf,"],
            "unable to parse modules to check for udf",
            "udf is currently loaded"
        );
        if module_result.status != CheckStatus::PASS {
            return module_result;
        }
        check_output_contains!(
            MODPROBE_CMD,
            ["-n", "-v", "udf"],
            &["install /bin/true"],
            "unable to parse modprobe output to check if udf is enabled",
            "modprobe for udf is not disabled"
        )
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure mounting of udf filesystems is disabled".to_string(),
            id: "1.1.1.1".to_string(),
            level: 2,
            name: "br01010101".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct BR01030100Checker {}

impl Checker for BR01030100Checker {
    fn execute(&self) -> CheckerResult {
        check_file_contains!(
            PROC_CMDLINE_FILE,
            &[
                "dm-mod.create=root,,,ro,0",
                "root=/dev/dm-0",
                "restart_on_corruption",
            ],
            "unable to verify cmdline includes dm-verity settings",
            "unable to verify dm-verity enforcement, settings not found"
        )
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure dm-verity is configured".to_string(),
            id: "1.3.1".to_string(),
            level: 1,
            name: "br01030100".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct BR01040100Checker {}

impl Checker for BR01040100Checker {
    fn execute(&self) -> CheckerResult {
        check_output_contains!(
            SYSCTL_CMD,
            ["fs.suid_dumpable"],
            &["fs.suid_dumpable = 0"],
            "unable to verify fs.suid_dumpable setting",
            "setuid core dumps are not disabled"
        )
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure setuid programs do not create core dumps".to_string(),
            id: "1.4.1".to_string(),
            level: 1,
            name: "br01040100".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct BR01040200Checker {}

impl Checker for BR01040200Checker {
    fn execute(&self) -> CheckerResult {
        check_output_contains!(
            SYSCTL_CMD,
            ["kernel.randomize_va_space"],
            &["kernel.randomize_va_space = 2"],
            "unable to verify kernel.randomize_va_space setting",
            "Address space layout randomization is not enabled"
        )
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure address space layout randomization (ASLR) is enabled".to_string(),
            id: "1.4.2".to_string(),
            level: 1,
            name: "br01040200".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct BR01040300Checker {}

impl Checker for BR01040300Checker {
    fn execute(&self) -> CheckerResult {
        check_output_contains!(
            SYSCTL_CMD,
            ["kernel.unprivileged_bpf_disabled"],
            &["kernel.unprivileged_bpf_disabled = 1"],
            "unable to verify kernel.unprivileged_bpf_disabled setting",
            "unprivileged eBPF is not disabled"
        )
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure unprivileged eBPF is disabled".to_string(),
            id: "1.4.3".to_string(),
            level: 1,
            name: "br01040300".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct BR01040400Checker {}

impl Checker for BR01040400Checker {
    fn execute(&self) -> CheckerResult {
        check_output_contains!(
            SYSCTL_CMD,
            ["user.max_user_namespaces"],
            &["user.max_user_namespaces = 0"],
            "unable to verify user.max_user_namespaces setting",
            "user namespaces are not disabled"
        )
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure user namespaces are disabled".to_string(),
            id: "1.4.4".to_string(),
            level: 2,
            name: "br01040400".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct BR01050100Checker {}

impl Checker for BR01050100Checker {
    fn execute(&self) -> CheckerResult {
        let mut result = CheckerResult::default();

        // Trying to avoid bringing in regex for now
        let to_match = &[
            ("SELinux status: ", " enabled"),
            ("Loaded policy name: ", " fortified"),
            ("Current mode: ", " enforcing"),
            ("Mode from config file: ", " enforcing"),
            ("Policy MLS status: ", " enabled"),
            ("Policy deny_unknown status: ", " denied"),
            ("Memory protection checking: ", " actual (secure)"),
        ];

        if let Ok(output) = Command::new(SESTATUS_CMD).output() {
            let mut matched = 0;

            if output.status.success() {
                let mp_output = String::from_utf8_lossy(&output.stdout).to_string();
                for line in mp_output.lines() {
                    for match_line in to_match {
                        if line.contains(match_line.0) && line.contains(match_line.1) {
                            matched += 1;
                            break;
                        }
                    }
                }

                if to_match.len() == matched {
                    result.status = CheckStatus::PASS;
                } else {
                    result.error = "Unable to find expected SELinux values".to_string();
                    result.status = CheckStatus::FAIL;
                }
            }
        } else {
            result.error = "unable to verify SELinux settings".to_string();
        }

        result
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure SELinux is configured".to_string(),
            id: "1.5.1".to_string(),
            level: 1,
            name: "br01050100".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct BR01050200Checker {}

impl Checker for BR01050200Checker {
    fn execute(&self) -> CheckerResult {
        check_file_contains!(
            LOCKDOWN_FILE,
            &["[integrity]"],
            "unable to verify lockdown mode",
            "lockdown integrity mode is not enabled"
        )
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure Lockdown is configured".to_string(),
            id: "1.5.2".to_string(),
            level: 2,
            name: "br01050200".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct BR02010101Checker {}

impl Checker for BR02010101Checker {
    fn execute(&self) -> CheckerResult {
        let result = check_file_contains!(
            CHRONY_CONF_FILE,
            &["pool"],
            "unable to verify time-servers setting",
            "no ntp servers are configured"
        );

        // Check if we need to continue
        if result.status == CheckStatus::FAIL {
            return result;
        }

        check_output_contains!(
            SYSTEMCTL_CMD,
            ["is-active", "chronyd"],
            &["active"],
            "unable to verify chronyd service enabled",
            "chronyd NTP service is not enabled"
        )
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure chrony is configured".to_string(),
            id: "2.1.1.1".to_string(),
            level: 1,
            name: "br02010101".to_string(),
            mode: Mode::Automatic,
        }
    }
}
