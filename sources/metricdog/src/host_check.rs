use crate::error::Result;

pub(crate) trait HostCheck {
    /// Checks whether this current boot is the first boot of the host.
    fn is_first_boot(&self) -> Result<bool>;

    /// The time, in milliseconds, it took for the host to reach the 'preconfigured' stage.
    fn preconfigured_time_ms(&self) -> Result<String>;

    /// The time, in milliseconds, it took for the host to reach the 'configured' stage.
    fn configured_time_ms(&self) -> Result<String>;

    /// The time, in milliseconds, it took for the host network to become ready.
    fn network_ready_time_ms(&self) -> Result<String>;

    /// The time, in milliseconds, it took for the filesystems to become ready.
    fn filesystem_ready_time_ms(&self) -> Result<String>;
}
