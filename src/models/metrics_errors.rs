// Metrics collection error types
// Error handling for system metrics gathering failures

use serde::{Deserialize, Serialize};

/// Represents failures in system metrics gathering
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error, PartialEq)]
pub enum MetricsCollectionError {
    #[error("System information unavailable: {reason}")]
    SystemUnavailable { reason: String },
    
    #[error("Permission denied accessing system metrics: {resource}")]
    PermissionDenied { resource: String },
    
    #[error("Failed to parse system data: {details}")]
    ParseError { details: String },
    
    #[error("Metrics collection timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },
    
    #[error("Insufficient memory to collect metrics")]
    OutOfMemory,
    
    #[error("Network interface error: {interface} - {reason}")]
    NetworkError { interface: String, reason: String },
    
    #[error("CPU metrics unavailable: {reason}")]
    CpuError { reason: String },
    
    #[error("Memory metrics unavailable: {reason}")]
    MemoryError { reason: String },
    
    #[error("Multiple collection errors: {count} errors")]
    MultipleErrors { 
        count: usize,
        #[serde(skip)] // Skip serde to avoid circular issues
        errors: Vec<MetricsCollectionError>,
    },
    
    #[error("Metrics collection service not initialized")]
    ServiceNotInitialized,
    
    #[error("Internal error: {message}")]
    Internal { message: String },
}

/// Result type for metrics API responses  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricsResponse<T> {
    /// Successful metrics collection
    Ok(T),
    /// Some metrics unavailable, partial data returned
    PartialData { 
        data: T, 
        errors: Vec<MetricsCollectionError> 
    },
    /// Complete failure
    Error(MetricsCollectionError),
}

#[allow(dead_code)]
impl MetricsCollectionError {
    /// Create a system unavailable error
    pub fn system_unavailable(reason: impl Into<String>) -> Self {
        Self::SystemUnavailable { 
            reason: reason.into() 
        }
    }

    /// Create a permission denied error
    pub fn permission_denied(resource: impl Into<String>) -> Self {
        Self::PermissionDenied { 
            resource: resource.into() 
        }
    }

    /// Create a parse error
    pub fn parse_error(details: impl Into<String>) -> Self {
        Self::ParseError { 
            details: details.into() 
        }
    }

    /// Create a timeout error
    pub fn timeout(timeout_ms: u64) -> Self {
        Self::Timeout { timeout_ms }
    }

    /// Create a network error
    pub fn network_error(interface: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::NetworkError { 
            interface: interface.into(), 
            reason: reason.into() 
        }
    }

    /// Create a CPU error
    pub fn cpu_error(reason: impl Into<String>) -> Self {
        Self::CpuError { 
            reason: reason.into() 
        }
    }

    /// Create a memory error
    pub fn memory_error(reason: impl Into<String>) -> Self {
        Self::MemoryError { 
            reason: reason.into() 
        }
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal { 
            message: message.into() 
        }
    }

    /// Combine multiple errors into a single error
    pub fn multiple(errors: Vec<MetricsCollectionError>) -> Self {
        let count = errors.len();
        Self::MultipleErrors { count, errors }
    }

    /// Check if error is recoverable (temporary)
    pub fn is_recoverable(&self) -> bool {
        match self {
            // These are likely temporary and may resolve
            Self::Timeout { .. } => true,
            Self::OutOfMemory => true,
            Self::NetworkError { .. } => true,
            Self::Internal { .. } => true,
            
            // These are likely permanent until system changes
            Self::SystemUnavailable { .. } => false,
            Self::PermissionDenied { .. } => false,
            Self::ServiceNotInitialized => false,
            
            // These depend on the specific issue
            Self::ParseError { .. } => false,
            Self::CpuError { .. } => false,
            Self::MemoryError { .. } => false,
            
            // For multiple errors, recoverable if any are recoverable
            Self::MultipleErrors { errors, .. } => {
                errors.iter().any(|e| e.is_recoverable())
            }
        }
    }

    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::Timeout { .. } => ErrorSeverity::Warning,
            Self::NetworkError { .. } => ErrorSeverity::Warning,
            Self::Internal { .. } => ErrorSeverity::Warning,
            
            Self::SystemUnavailable { .. } => ErrorSeverity::Error,
            Self::PermissionDenied { .. } => ErrorSeverity::Error,
            Self::ParseError { .. } => ErrorSeverity::Error,
            Self::OutOfMemory => ErrorSeverity::Critical,
            Self::CpuError { .. } => ErrorSeverity::Error,
            Self::MemoryError { .. } => ErrorSeverity::Error,
            Self::ServiceNotInitialized => ErrorSeverity::Critical,
            
            Self::MultipleErrors { errors, .. } => {
                // Return highest severity from contained errors
                errors.iter()
                    .map(|e| e.severity())
                    .max()
                    .unwrap_or(ErrorSeverity::Error)
            }
        }
    }

    /// Get suggested retry delay in milliseconds
    pub fn retry_delay_ms(&self) -> Option<u64> {
        match self {
            Self::Timeout { .. } => Some(1000),      // Retry after 1 second
            Self::OutOfMemory => Some(5000),         // Retry after 5 seconds
            Self::NetworkError { .. } => Some(2000), // Retry after 2 seconds
            Self::Internal { .. } => Some(1000),     // Retry after 1 second
            
            // No retry for these errors
            Self::SystemUnavailable { .. } => None,
            Self::PermissionDenied { .. } => None,
            Self::ParseError { .. } => None,
            Self::CpuError { .. } => None,
            Self::MemoryError { .. } => None,
            Self::ServiceNotInitialized => None,
            
            Self::MultipleErrors { errors, .. } => {
                // Return shortest retry delay from recoverable errors
                errors.iter()
                    .filter_map(|e| e.retry_delay_ms())
                    .min()
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Warning,
    Error,
    Critical,
}

#[allow(dead_code)]
impl<T> MetricsResponse<T> {
    /// Check if response contains any data
    pub fn has_data(&self) -> bool {
        matches!(self, Self::Ok(_) | Self::PartialData { .. })
    }

    /// Get data if available
    pub fn data(self) -> Option<T> {
        match self {
            Self::Ok(data) => Some(data),
            Self::PartialData { data, .. } => Some(data),
            Self::Error(_) => None,
        }
    }

    /// Get errors if any
    pub fn errors(&self) -> Vec<&MetricsCollectionError> {
        match self {
            Self::Ok(_) => vec![],
            Self::PartialData { errors, .. } => errors.iter().collect(),
            Self::Error(error) => vec![error],
        }
    }

    /// Convert to standard Result
    pub fn into_result(self) -> Result<T, MetricsCollectionError> {
        match self {
            Self::Ok(data) => Ok(data),
            Self::PartialData { data, .. } => Ok(data), // Ignore errors, return data
            Self::Error(error) => Err(error),
        }
    }

    /// Map the data type while preserving errors
    pub fn map<U, F>(self, f: F) -> MetricsResponse<U> 
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Self::Ok(data) => MetricsResponse::Ok(f(data)),
            Self::PartialData { data, errors } => MetricsResponse::PartialData { 
                data: f(data), 
                errors 
            },
            Self::Error(error) => MetricsResponse::Error(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = MetricsCollectionError::system_unavailable("sysinfo crate failed");
        match error {
            MetricsCollectionError::SystemUnavailable { reason } => {
                assert_eq!(reason, "sysinfo crate failed");
            }
            _ => panic!("Expected SystemUnavailable error"),
        }
    }

    #[test]
    fn test_error_recoverability() {
        let timeout_error = MetricsCollectionError::timeout(5000);
        assert!(timeout_error.is_recoverable());

        let permission_error = MetricsCollectionError::permission_denied("/proc/stat");
        assert!(!permission_error.is_recoverable());

        let multiple_error = MetricsCollectionError::multiple(vec![
            timeout_error.clone(),
            permission_error,
        ]);
        assert!(multiple_error.is_recoverable()); // At least one is recoverable
    }

    #[test]
    fn test_error_severity() {
        let warning_error = MetricsCollectionError::timeout(1000);
        assert_eq!(warning_error.severity(), ErrorSeverity::Warning);

        let critical_error = MetricsCollectionError::OutOfMemory;
        assert_eq!(critical_error.severity(), ErrorSeverity::Critical);

        let multiple_error = MetricsCollectionError::multiple(vec![
            warning_error,
            critical_error,
        ]);
        assert_eq!(multiple_error.severity(), ErrorSeverity::Critical); // Highest severity
    }

    #[test]
    fn test_retry_delay() {
        let timeout_error = MetricsCollectionError::timeout(1000);
        assert_eq!(timeout_error.retry_delay_ms(), Some(1000));

        let permission_error = MetricsCollectionError::permission_denied("resource");
        assert_eq!(permission_error.retry_delay_ms(), None);

        let multiple_error = MetricsCollectionError::multiple(vec![
            MetricsCollectionError::timeout(2000),
            MetricsCollectionError::network_error("eth0", "timeout"),
            permission_error,
        ]);
        assert_eq!(multiple_error.retry_delay_ms(), Some(1000)); // Shortest delay from recoverable errors
    }

    #[test]
    fn test_metrics_response_success() {
        let response: MetricsResponse<String> = MetricsResponse::Ok("test data".to_string());
        
        assert!(response.has_data());
        assert_eq!(response.errors().len(), 0);
        assert_eq!(response.data(), Some("test data".to_string()));
    }

    #[test]
    fn test_metrics_response_partial() {
        let errors = vec![
            MetricsCollectionError::network_error("eth0", "timeout"),
        ];
        let response: MetricsResponse<String> = MetricsResponse::PartialData {
            data: "partial data".to_string(),
            errors,
        };
        
        assert!(response.has_data());
        assert_eq!(response.errors().len(), 1);
        assert_eq!(response.data(), Some("partial data".to_string()));
    }

    #[test]
    fn test_metrics_response_error() {
        let error = MetricsCollectionError::system_unavailable("system down");
        let response: MetricsResponse<String> = MetricsResponse::Error(error);
        
        assert!(!response.has_data());
        assert_eq!(response.errors().len(), 1);
        assert_eq!(response.data(), None);
    }

    #[test]
    fn test_metrics_response_into_result() {
        let success_response: MetricsResponse<i32> = MetricsResponse::Ok(42);
        assert_eq!(success_response.into_result(), Ok(42));

        let partial_response: MetricsResponse<i32> = MetricsResponse::PartialData {
            data: 42,
            errors: vec![MetricsCollectionError::timeout(1000)],
        };
        assert_eq!(partial_response.into_result(), Ok(42)); // Should return data despite errors

        let error_response: MetricsResponse<i32> = MetricsResponse::Error(
            MetricsCollectionError::system_unavailable("test")
        );
        assert!(error_response.into_result().is_err());
    }

    #[test]
    fn test_metrics_response_map() {
        let response: MetricsResponse<i32> = MetricsResponse::Ok(42);
        let mapped = response.map(|x| x.to_string());
        
        match mapped {
            MetricsResponse::Ok(data) => assert_eq!(data, "42"),
            _ => panic!("Expected Ok response"),
        }
    }

    #[test]
    fn test_error_serialization() {
        let error = MetricsCollectionError::timeout(5000);
        
        let json = serde_json::to_string(&error).unwrap();
        let deserialized: MetricsCollectionError = serde_json::from_str(&json).unwrap();
        
        match deserialized {
            MetricsCollectionError::Timeout { timeout_ms } => {
                assert_eq!(timeout_ms, 5000);
            }
            _ => panic!("Expected Timeout error after deserialization"),
        }
    }

    #[test]
    fn test_multiple_errors_skip_serialization() {
        // Test that MultipleErrors.errors field is skipped during serialization
        let multiple_error = MetricsCollectionError::multiple(vec![
            MetricsCollectionError::timeout(1000),
            MetricsCollectionError::parse_error("test"),
        ]);
        
        let json = serde_json::to_string(&multiple_error).unwrap();
        
        // Should serialize successfully despite Vec<MetricsCollectionError> in errors field
        assert!(json.contains("MultipleErrors"));
        assert!(json.contains("\"count\":2"));
        
        // The errors field should be skipped
        assert!(!json.contains("\"errors\""));
    }
}