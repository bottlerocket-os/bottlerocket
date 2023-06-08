use crate::error::Result;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ServiceHealth {
    /// Whether or not the service is healthy.
    pub(crate) is_healthy: bool,
    /// In the event of an unhealthy service, the service's exit code (if found).
    pub(crate) exit_code: Option<i32>,
}

pub(crate) trait ServiceCheck {
    /// Checks the given service to see if it is healthy.
    fn check(&self, service_name: &str) -> Result<ServiceHealth>;
}
