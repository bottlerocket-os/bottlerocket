use tonic::{Code, Status};
use tracing::error;

/// Convert anyhow errors to gRPC status codes with proper mapping
pub trait IntoStatus {
    fn into_status(self) -> Status;
}

impl IntoStatus for anyhow::Error {
    fn into_status(self) -> Status {
        // Log the full error chain for debugging
        error!("Error: {:?}", self);
        
        // Try to downcast to known error types
        if let Some(io_err) = self.downcast_ref::<std::io::Error>() {
            return match io_err.kind() {
                std::io::ErrorKind::NotFound => {
                    Status::not_found(format!("Resource not found: {}", io_err))
                }
                std::io::ErrorKind::PermissionDenied => {
                    Status::permission_denied(format!("Permission denied: {}", io_err))
                }
                std::io::ErrorKind::ConnectionRefused => {
                    Status::unavailable("Service unavailable: connection refused")
                }
                std::io::ErrorKind::TimedOut => {
                    Status::deadline_exceeded("Operation timed out")
                }
                std::io::ErrorKind::AlreadyExists => {
                    Status::already_exists(format!("Resource already exists: {}", io_err))
                }
                std::io::ErrorKind::InvalidData => {
                    Status::invalid_argument(format!("Invalid data: {}", io_err))
                }
                _ => Status::internal(format!("IO error: {}", io_err)),
            };
        }
        
        // Check for specific error patterns in the message
        let error_string = self.to_string();
        let error_lower = error_string.to_lowercase();
        
        if error_lower.contains("not found") {
            Status::not_found(error_string)
        } else if error_lower.contains("already exists") {
            Status::already_exists(error_string)
        } else if error_lower.contains("invalid") || error_lower.contains("validation") {
            Status::invalid_argument(error_string)
        } else if error_lower.contains("unauthorized") || error_lower.contains("authentication") {
            Status::unauthenticated(error_string)
        } else if error_lower.contains("forbidden") || error_lower.contains("permission") {
            Status::permission_denied(error_string)
        } else if error_lower.contains("timeout") || error_lower.contains("deadline") {
            Status::deadline_exceeded(error_string)
        } else if error_lower.contains("not implemented") {
            Status::unimplemented(error_string)
        } else if error_lower.contains("unavailable") || error_lower.contains("connection") {
            Status::unavailable(error_string)
        } else if error_lower.contains("conflict") {
            Status::failed_precondition(error_string)
        } else if error_lower.contains("too many") || error_lower.contains("rate limit") {
            Status::resource_exhausted(error_string)
        } else {
            // Default to internal error
            Status::internal(format!("Internal error: {}", error_string))
        }
    }
}

/// Helper trait for Result types
pub trait IntoStatusResult<T> {
    fn into_status(self) -> Result<T, Status>;
}

impl<T> IntoStatusResult<T> for anyhow::Result<T> {
    fn into_status(self) -> Result<T, Status> {
        self.map_err(|e| e.into_status())
    }
}

/// Create a Status with a specific code and detailed message
pub fn status_with_details(code: Code, message: impl Into<String>, details: impl Into<String>) -> Status {
    let message = message.into();
    let details = details.into();
    
    // Log the error
    error!("gRPC error - Code: {:?}, Message: {}, Details: {}", code, message, details);
    
    // Create status with message
    Status::new(code, format!("{}: {}", message, details))
}

/// Common error responses
pub struct ErrorResponses;

impl ErrorResponses {
    pub fn configuration_not_found() -> Status {
        Status::not_found("Machine configuration not found. Apply a configuration first.")
    }
    
    pub fn invalid_configuration(reason: &str) -> Status {
        Status::invalid_argument(format!("Invalid configuration: {}", reason))
    }
    
    pub fn operation_in_progress(operation: &str) -> Status {
        Status::unavailable(format!("{} operation already in progress", operation))
    }
    
    pub fn bottlerocket_api_error(error: &str) -> Status {
        Status::unavailable(format!("Bottlerocket API error: {}", error))
    }
    
    pub fn validation_failed(errors: Vec<String>) -> Status {
        Status::invalid_argument(format!("Validation failed: {}", errors.join(", ")))
    }
    
    pub fn precondition_failed(reason: &str) -> Status {
        Status::failed_precondition(reason)
    }
    
    pub fn internal_error(context: &str) -> Status {
        error!("Internal error in {}", context);
        Status::internal(format!("Internal error in {}", context))
    }
}