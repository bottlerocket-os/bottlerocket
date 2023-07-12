use bloodhound::results::{CheckStatus, Checker, CheckerMetadata, CheckerResult, Mode};
use bloodhound::*;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use walkdir::WalkDir;

const PROC_MODULES_FILE: &str = "/proc/modules";
const PROC_CMDLINE_FILE: &str = "/proc/cmdline";
const LOCKDOWN_FILE: &str = "/sys/kernel/security/lockdown";
const CHRONY_CONF_FILE: &str = "/etc/chrony.conf";
const JOURNALD_CONF_FILE: &str = "/usr/lib/systemd/journald.conf.d/journald.conf";
const SYSCTL_CMD: &str = "/usr/sbin/sysctl";
const SYSTEMCTL_CMD: &str = "/usr/bin/systemctl";
const MODPROBE_CMD: &str = "/bin/modprobe";
const SESTATUS_CMD: &str = "/usr/bin/sestatus";
const IPTABLES_CMD: &str = "/usr/sbin/iptables";
const IP6TABLES_CMD: &str = "/usr/sbin/ip6tables";

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

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct BR03010100Checker {}

impl Checker for BR03010100Checker {
    fn execute(&self) -> CheckerResult {
        let settings = [
            "net.ipv4.conf.all.send_redirects",
            "net.ipv4.conf.default.send_redirects",
        ];

        let output = [
            "net.ipv4.conf.all.send_redirects = 0",
            "net.ipv4.conf.default.send_redirects = 0",
        ];

        check_output_contains!(
            SYSCTL_CMD,
            settings,
            &output,
            "unable to verify redirect settings",
            "redirects not disabled"
        )
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure packet redirect sending is disabled".to_string(),
            id: "3.1.1".to_string(),
            level: 2,
            name: "br03010100".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct BR03020100Checker {}

impl Checker for BR03020100Checker {
    fn execute(&self) -> CheckerResult {
        let settings = [
            "net.ipv4.conf.all.accept_source_route",
            "net.ipv4.conf.default.accept_source_route",
            "net.ipv6.conf.all.accept_source_route",
            "net.ipv6.conf.default.accept_source_route",
        ];

        let output = [
            "net.ipv4.conf.all.accept_source_route = 0",
            "net.ipv4.conf.default.accept_source_route = 0",
            "net.ipv6.conf.all.accept_source_route = 0",
            "net.ipv6.conf.default.accept_source_route = 0",
        ];

        check_output_contains!(
            SYSCTL_CMD,
            settings,
            &output,
            "unable to verify source route settings",
            "accept source route not disabled"
        )
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure source routed packets are not accepted".to_string(),
            id: "3.2.1".to_string(),
            level: 2,
            name: "br03020100".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct BR03020200Checker {}

impl Checker for BR03020200Checker {
    fn execute(&self) -> CheckerResult {
        let settings = [
            "net.ipv4.conf.all.accept_redirects",
            "net.ipv4.conf.default.accept_redirects",
            "net.ipv6.conf.all.accept_redirects",
            "net.ipv6.conf.default.accept_redirects",
        ];

        let output = [
            "net.ipv4.conf.all.accept_redirects = 0",
            "net.ipv4.conf.default.accept_redirects = 0",
            "net.ipv6.conf.all.accept_redirects = 0",
            "net.ipv6.conf.default.accept_redirects = 0",
        ];

        check_output_contains!(
            SYSCTL_CMD,
            settings,
            &output,
            "unable to verify redirect settings",
            "accept redirects not disabled"
        )
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure ICMP redirects are not accepted".to_string(),
            id: "3.2.2".to_string(),
            level: 2,
            name: "br03020200".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct BR03020300Checker {}

impl Checker for BR03020300Checker {
    fn execute(&self) -> CheckerResult {
        let settings = [
            "net.ipv4.conf.all.secure_redirects",
            "net.ipv4.conf.default.secure_redirects",
        ];

        let output = [
            "net.ipv4.conf.all.secure_redirects = 0",
            "net.ipv4.conf.default.secure_redirects = 0",
        ];

        check_output_contains!(
            SYSCTL_CMD,
            settings,
            &output,
            "unable to verify secure redirect settings",
            "secure redirects not disabled"
        )
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure secure ICMP redirects are not accepted".to_string(),
            id: "3.2.3".to_string(),
            level: 2,
            name: "br03020300".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct BR03020400Checker {}

impl Checker for BR03020400Checker {
    fn execute(&self) -> CheckerResult {
        let settings = [
            "net.ipv4.conf.all.log_martians",
            "net.ipv4.conf.default.log_martians",
        ];

        let output = [
            "net.ipv4.conf.all.log_martians = 1",
            "net.ipv4.conf.default.log_martians = 1",
        ];

        check_output_contains!(
            SYSCTL_CMD,
            settings,
            &output,
            "unable to verify martian packet logging settings",
            "martian packet logging not enabled"
        )
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure suspicious packets are logged".to_string(),
            id: "3.2.4".to_string(),
            level: 2,
            name: "br03020400".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct BR03020500Checker {}

impl Checker for BR03020500Checker {
    fn execute(&self) -> CheckerResult {
        check_output_contains!(
            SYSCTL_CMD,
            ["net.ipv4.icmp_echo_ignore_broadcasts"],
            &["net.ipv4.icmp_echo_ignore_broadcasts = 1"],
            "unable to verify broadcast ICMP requests setting",
            "broadcast ICMP requests not ignored"
        )
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure broadcast ICMP requests are ignored".to_string(),
            id: "3.2.5".to_string(),
            level: 1,
            name: "br03020500".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct BR03020600Checker {}

impl Checker for BR03020600Checker {
    fn execute(&self) -> CheckerResult {
        check_output_contains!(
            SYSCTL_CMD,
            ["net.ipv4.icmp_ignore_bogus_error_responses"],
            &["net.ipv4.icmp_ignore_bogus_error_responses = 1"],
            "unable to verify bogus ICMP bogus requests setting",
            "ignore bogus ICMP requests not ignored"
        )
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure bogus ICMP responses are ignored".to_string(),
            id: "3.2.6".to_string(),
            level: 1,
            name: "br03020600".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct BR03020700Checker {}

impl Checker for BR03020700Checker {
    fn execute(&self) -> CheckerResult {
        check_output_contains!(
            SYSCTL_CMD,
            ["net.ipv4.tcp_syncookies"],
            &["net.ipv4.tcp_syncookies = 1"],
            "unable to verify SYN flood cookie protection setting",
            "SYN flood cookie protection not enabled"
        )
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure TCP SYN Cookies is enabled".to_string(),
            id: "3.2.7".to_string(),
            level: 1,
            name: "br03020700".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct BR03030100Checker {}

impl Checker for BR03030100Checker {
    fn execute(&self) -> CheckerResult {
        let result = check_file_contains!(
            PROC_MODULES_FILE,
            &["sctp"],
            "unable to parse modules to check for sctp",
            "sctp is currently loaded"
        );

        // Check if we need to continue
        if result.status == CheckStatus::FAIL {
            return result;
        }

        check_output_contains!(
            MODPROBE_CMD,
            ["-n", "-v", "sctp"],
            &["install /bin/true"],
            "unable to parse modprobe output to check if sctp is enabled",
            "modprobe for sctp is not disabled"
        )
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure SCTP is disabled".to_string(),
            id: "3.3.1".to_string(),
            level: 2,
            name: "br03030100".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct BR03040101Checker {}

impl Checker for BR03040101Checker {
    fn execute(&self) -> CheckerResult {
        let output = &[
            "Chain INPUT (policy DROP)",
            "Chain FORWARD (policy DROP)",
            "Chain OUTPUT (policy DROP)",
        ];

        check_output_contains!(
            IPTABLES_CMD,
            ["-L"],
            output,
            "unable to verify iptables settings",
            "unable to find expected iptables values"
        )
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure IPv4 default deny firewall policy".to_string(),
            id: "3.4.1.1".to_string(),
            level: 2,
            name: "br03040101".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct BR03040102Checker {}

impl Checker for BR03040102Checker {
    fn execute(&self) -> CheckerResult {
        let mut result = CheckerResult::default();

        // Order matters here, so need to find the first one, then look for the second one
        let first = (
            "ACCEPT",
            "--  lo     *       0.0.0.0/0            0.0.0.0/0",
        );
        let second = ("DROP", "--  *      *       127.0.0.0/8          0.0.0.0/0");

        if let Ok(output) = Command::new(IPTABLES_CMD)
            .args(["-L", "INPUT", "-v", "-n"])
            .output()
        {
            let mut first_found = false;
            let mut second_found = false;

            if output.status.success() {
                let std_output = String::from_utf8_lossy(&output.stdout).to_string();
                for line in std_output.lines() {
                    if !first_found && line.contains(first.0) && line.contains(first.1) {
                        first_found = true;
                        continue;
                    }

                    if first_found && line.contains(second.0) && line.contains(second.1) {
                        second_found = true;
                        break;
                    }
                }
            }

            if first_found && second_found {
                result.status = CheckStatus::PASS;
            } else {
                result.error = "Unable to find expected iptables INPUT values".to_string();
                result.status = CheckStatus::FAIL;
                return result;
            }
        } else {
            result.error = "unable to verify iptables INPUT settings".to_string();
        }

        if let Some(found) = look_for_string_in_output(
            IPTABLES_CMD,
            ["-L", "OUTPUT", "-v", "-n"],
            "ACCEPT     0    --  *      lo      0.0.0.0/0            0.0.0.0/0",
        ) {
            if !found {
                result.error = "iptables OUTPUT rule not found".to_string();
                result.status = CheckStatus::FAIL;
            } else {
                result.status = CheckStatus::PASS;
            }
        } else {
            result.error =
                "unable to parse iptables OUTPUT rules to verify loopback policy".to_string();
        }

        result
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure IPv4 loopback traffic is configured".to_string(),
            id: "3.4.1.2".to_string(),
            level: 2,
            name: "br03040102".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct BR03040201Checker {}

impl Checker for BR03040201Checker {
    fn execute(&self) -> CheckerResult {
        let output = &[
            "Chain INPUT (policy DROP)",
            "Chain FORWARD (policy DROP)",
            "Chain OUTPUT (policy DROP)",
        ];

        check_output_contains!(
            IP6TABLES_CMD,
            ["-L"],
            output,
            "unable to verify ip6tables settings",
            "unable to find expected ip6tables values"
        )
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure IPv6 default deny firewall policy".to_string(),
            id: "3.4.2.1".to_string(),
            level: 2,
            name: "br03040201".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct BR03040202Checker {}

impl Checker for BR03040202Checker {
    fn execute(&self) -> CheckerResult {
        let mut result = CheckerResult::default();

        // Order matters here, so need to find the first one, then look for the second one
        let first = ("ACCEPT", "--  lo     *       ::/0                 ::/0");
        let second = ("DROP", "--  *      *       ::1                  ::/0");

        if let Ok(output) = Command::new(IP6TABLES_CMD)
            .args(["-L", "INPUT", "-v", "-n"])
            .output()
        {
            let mut first_found = false;
            let mut second_found = false;

            if output.status.success() {
                let std_output = String::from_utf8_lossy(&output.stdout).to_string();
                for line in std_output.lines() {
                    if !first_found && line.contains(first.0) && line.contains(first.1) {
                        first_found = true;
                        continue;
                    }

                    if first_found && line.contains(second.0) && line.contains(second.1) {
                        second_found = true;
                        break;
                    }
                }
            }

            if first_found && second_found {
                result.status = CheckStatus::PASS;
            } else {
                result.error = "Unable to find expected iptables INPUT values".to_string();
                result.status = CheckStatus::FAIL;
                return result;
            }
        } else {
            result.error = "unable to verify iptables INPUT settings".to_string();
        }

        if let Some(found) = look_for_string_in_output(
            IP6TABLES_CMD,
            ["-L", "OUTPUT", "-v", "-n"],
            "ACCEPT     0    --  *      lo      ::/0                 ::/0",
        ) {
            if !found {
                result.error = "iptables OUTPUT rule not found".to_string();
                result.status = CheckStatus::FAIL;
            } else {
                result.status = CheckStatus::PASS;
            }
        } else {
            result.error =
                "unable to parse iptables OUTPUT rules to verify loopback policy".to_string();
        }

        result
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure IPv6 loopback traffic is configured".to_string(),
            id: "3.4.2.2".to_string(),
            level: 2,
            name: "br03040202".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct BR04010101Checker {}

impl Checker for BR04010101Checker {
    fn execute(&self) -> CheckerResult {
        check_file_contains!(
            JOURNALD_CONF_FILE,
            &["Storage=persistent"],
            "unable to verify journald settings",
            "journald is not configured"
        )
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure journald is configured to write logs to persistent disk".to_string(),
            id: "4.1.1.1".to_string(),
            level: 1,
            name: "br04010101".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct BR04010200Checker {}

impl Checker for BR04010200Checker {
    fn execute(&self) -> CheckerResult {
        let mut result = CheckerResult::default();

        // Recursively walk over all files in /var/log/journal and check perms
        for file in WalkDir::new("/var/log/journal")
            .into_iter()
            .filter_map(|file| file.ok())
        {
            if let Ok(metadata) = file.metadata() {
                if !metadata.is_file() {
                    continue;
                }

                if (metadata.permissions().mode() & 0b111) > 0 {
                    result.error = format!("file {:?} has permissions for 'other'", file.path());
                    result.status = CheckStatus::FAIL;
                    break;
                }
            }
        }

        result
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure permissions on journal files are configured".to_string(),
            id: "4.1.2".to_string(),
            level: 1,
            name: "br04010200".to_string(),
            mode: Mode::Automatic,
        }
    }
}
