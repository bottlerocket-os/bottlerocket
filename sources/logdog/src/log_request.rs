//! Provides the list of commands that `logdog` will run.

/// `LogRequest` holds a command to run to retrieve log information and the filename for its output.
pub(crate) struct LogRequest<'a> {
    pub(crate) filename: &'a str,
    pub(crate) command: &'a str,
}

cfg_if::cfg_if! {
    if #[cfg(variant_group = "aws-k8s")] {
        /// Returns the list of `logdog` commands for aws-k8s variants.
        pub(crate) fn log_requests<'a>() -> impl Iterator<Item = LogRequest<'static>> {
        [
            ("containerd-config", "containerd config dump"),
            ("df", "df -h"),
            ("df-inodes", "df -hi"),
            ("dmesg", "dmesg --color=never --nopager"),
            ("etc-mtab", "cat /etc/mtab"),
            ("iptables-filter", "iptables -nvL -t filter"),
            ("iptables-nat", "iptables -nvL -t nat"),
            ("journalctl-boots", "journalctl --list-boots --no-pager"),
            ("journalctl.errors", "journalctl -p err -a --no-pager"),
            ("journalctl.log", "journalctl -a --no-pager"),
            ("kube-status", "systemctl status kube* -l --no-pager"),
            ("os-release", "cat /etc/os-release"),
            ("proc-mounts", "cat /proc/mounts"),
            ("settings.json", "apiclient --method GET --uri /"),
            ("signpost", "signpost status"),
            ("wicked", "wicked show all"),
        ]
        .iter()
        .map(|(filename, command)| LogRequest { filename, command })
        }
    } else if #[cfg(variant_group = "aws-ecs")] {
        /// Returns the list of `logdog` commands for aws-ecs variants.
        pub(crate) fn log_requests<'a>() -> impl Iterator<Item = LogRequest<'static>> {
        [
            ("containerd-config", "containerd config dump"),
            ("df", "df -h"),
            ("df-inodes", "df -hi"),
            ("dmesg", "dmesg --color=never --nopager"),
            ("docker-daemon.json", "cat /etc/docker/daemon.json"),
            ("docker-info", "docker info"),
            ("ecs-agent-state.json", "cat /var/lib/ecs/data/ecs_agent_data.json"),
            ("ecs-config.json", "cat /etc/ecs/ecs.config.json"),
            ("etc-mtab", "cat /etc/mtab"),
            ("iptables-filter", "iptables -nvL -t filter"),
            ("iptables-nat", "iptables -nvL -t nat"),
            ("journalctl-boots", "journalctl --list-boots --no-pager"),
            ("journalctl.errors", "journalctl -p err -a --no-pager"),
            ("journalctl.log", "journalctl -a --no-pager"),
            ("os-release", "cat /etc/os-release"),
            ("proc-mounts", "cat /proc/mounts"),
            ("settings.json", "apiclient --method GET --uri /"),
            ("signpost", "signpost status"),
            ("wicked", "wicked show all"),
            // TODO - https://github.com/bottlerocket-os/bottlerocket/issues/1039
            // ("ecs-tasks", "curl localhost:51678/v1/tasks"),
        ]
        .iter()
        .map(|(filename, command)| LogRequest { filename, command })
        }
    }
    else {
        /// Returns the list of `logdog` commands for dev or unspecified variants.
        pub(crate) fn log_requests<'a>() -> impl Iterator<Item = LogRequest<'static>> {
        [
            ("containerd-config", "containerd config dump"),
            ("df", "df -h"),
            ("df-inodes", "df -hi"),
            ("dmesg", "dmesg --color=never --nopager"),
            ("docker-daemon.json", "cat /etc/docker/daemon.json"),
            ("docker-info", "docker info"),
            ("etc-mtab", "cat /etc/mtab"),
            ("iptables-filter", "iptables -nvL -t filter"),
            ("iptables-nat", "iptables -nvL -t nat"),
            ("journalctl-boots", "journalctl --list-boots --no-pager"),
            ("journalctl.errors", "journalctl -p err -a --no-pager"),
            ("journalctl.log", "journalctl -a --no-pager"),
            ("os-release", "cat /etc/os-release"),
            ("proc-mounts", "cat /proc/mounts"),
            ("settings.json", "apiclient --method GET --uri /"),
            ("signpost", "signpost status"),
            ("wicked", "wicked show all"),
        ]
        .iter()
        .map(|(filename, command)| LogRequest { filename, command })
        }
    }
}
