//! Provides the list of commands that `logdog` will run.

/// `LogRequest` holds a command to run to retrieve log information and the filename for its output.
pub(crate) struct LogRequest<'a> {
    pub(crate) filename: &'a str,
    pub(crate) command: &'a str,
}

/// Returns the standard list of `logdog` commands.
pub(crate) fn log_requests<'a>() -> impl Iterator<Item = LogRequest<'static>> {
    [
        ("os-release", "cat /etc/os-release"),
        ("journalctl-boots", "journalctl --list-boots --no-pager"),
        ("journalctl.errors", "journalctl -p err -a --no-pager"),
        ("journalctl.log", "journalctl -a --no-pager"),
        ("signpost", "signpost status"),
        ("settings.json", "apiclient --method GET --uri /"),
        ("wicked", "wicked show all"),
        ("containerd-config", "containerd config dump"),
        ("kube-status", "systemctl status kube* -l --no-pager"),
        ("dmesg", "dmesg --color=never --nopager"),
        ("iptables-filter", "iptables -nvL -t filter"),
        ("iptables-nat", "iptables -nvL -t nat"),
        ("df", "df -h"),
        ("df-inodes", "df -hi"),
    ]
    .iter()
    .map(|(filename, command)| LogRequest { filename, command })
}
