// Status data model
// Complete data structure for API consumption

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{ServerMetrics, OsInfoValidationError, OsInfo};

/// Complete data structure for API consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusData {
    /// Current system metrics
    pub server_metrics: ServerMetrics,
    /// Update frequency in seconds (5 seconds)
    pub collection_interval_seconds: u32,
    /// Static server information
    pub server_info: ServerInfo,
}

/// Static server identification and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Server hostname
    pub hostname: String,
    /// Application version
    pub version: String,
    /// When server started
    pub start_time: DateTime<Utc>,
    /// Deployment environment (dev/staging/prod)
    pub environment: String,
    /// Operating system information
    pub os_info: OsInfo,
}

/// Validation errors for status data
#[derive(Debug, thiserror::Error)]
pub enum StatusValidationError {
    #[error("Invalid collection interval: {interval} (must be >= 1 second)")]
    InvalidCollectionInterval { interval: u32 },
    #[error("Invalid hostname: {hostname} (must be valid DNS hostname)")]
    InvalidHostname { hostname: String },
    #[error("Invalid version: {version} (must follow semantic versioning)")]
    InvalidVersion { version: String },
    #[error("Invalid start time: {start_time} (must be in the past)")]
    InvalidStartTime { start_time: DateTime<Utc> },
    #[error("Invalid environment: {environment} (must be development, staging, or production)")]
    InvalidEnvironment { environment: String },
    #[error("Invalid OS name: {name} (must be non-empty)")]
    InvalidOsName { name: String },
    #[error("Invalid OS version: {version} (must be non-empty)")]
    InvalidOsVersion { version: String },
    #[error("Invalid OS architecture: {architecture} (must be non-empty)")]
    InvalidOsArchitecture { architecture: String },
    #[error("Invalid kernel version: {kernel_version} (must be non-empty)")]
    InvalidKernelVersion { kernel_version: String },
    #[error("Invalid OS distribution: {distribution} (must be non-empty if present)")]
    InvalidOsDistribution { distribution: String },
    #[error("Invalid OS description: {description} (must be non-empty)")]
    InvalidOsDescription { description: String },
    #[error("Server metrics validation failed: {source}")]
    ServerMetricsError {
        #[from]
        source: super::MetricsValidationError,
    },
}

/// Convert OS info validation errors to status validation errors
impl From<OsInfoValidationError> for StatusValidationError {
    fn from(error: OsInfoValidationError) -> Self {
        match error {
            OsInfoValidationError::InvalidName { name } => {
                StatusValidationError::InvalidOsName { name }
            }
            OsInfoValidationError::InvalidVersion { version } => {
                StatusValidationError::InvalidOsVersion { version }
            }
            OsInfoValidationError::InvalidArchitecture { architecture } => {
                StatusValidationError::InvalidOsArchitecture { architecture }
            }
            OsInfoValidationError::InvalidKernelVersion { kernel_version } => {
                StatusValidationError::InvalidKernelVersion { kernel_version }
            }
            OsInfoValidationError::InvalidDistribution { distribution } => {
                StatusValidationError::InvalidOsDistribution { distribution }
            }
            OsInfoValidationError::InvalidLongDescription { description } => {
                StatusValidationError::InvalidOsDescription { description }
            }
        }
    }
}

#[allow(dead_code)]
impl StatusData {
    /// Create new StatusData with validation
    pub fn new(
        server_metrics: ServerMetrics,
        collection_interval_seconds: u32,
        server_info: ServerInfo,
    ) -> Result<Self, StatusValidationError> {
        let data = StatusData {
            server_metrics,
            collection_interval_seconds,
            server_info,
        };

        data.validate()?;
        Ok(data)
    }

    /// Validate status data
    pub fn validate(&self) -> Result<(), StatusValidationError> {
        // Validate collection interval
        if self.collection_interval_seconds < 1 {
            return Err(StatusValidationError::InvalidCollectionInterval {
                interval: self.collection_interval_seconds,
            });
        }

        // Validate server metrics
        self.server_metrics.validate()?;

        // Validate server info
        self.server_info.validate()?;

        Ok(())
    }

    /// Get the overall health status based on current metrics
    pub fn get_health_status(&self) -> super::HealthStatus {
        super::HealthStatus::from_metrics(
            self.server_metrics.cpu_usage.usage_percentage,
            self.server_metrics.memory_usage.usage_percentage,
        )
    }

    /// Get server uptime in human-readable format
    pub fn format_uptime(&self) -> String {
        let uptime = self.server_metrics.uptime;
        let days = uptime.as_secs() / 86400;
        let hours = (uptime.as_secs() % 86400) / 3600;
        let minutes = (uptime.as_secs() % 3600) / 60;

        if days > 0 {
            format!("{} days, {} hours, {} minutes", days, hours, minutes)
        } else if hours > 0 {
            format!("{} hours, {} minutes", hours, minutes)
        } else {
            format!("{} minutes", minutes)
        }
    }

    /// Get collection interval in human-readable format
    pub fn format_collection_interval(&self) -> String {
        match self.collection_interval_seconds {
            1 => "every second".to_string(),
            n if n < 60 => format!("every {} seconds", n),
            n if n == 60 => "every minute".to_string(),
            n if n < 3600 => format!("every {} minutes", n / 60),
            n if n == 3600 => "every hour".to_string(),
            n => format!("every {} hours", n / 3600),
        }
    }

    /// Check if any metrics are in critical state
    pub fn has_critical_issues(&self) -> bool {
        self.server_metrics.memory_usage.usage_percentage > 90.0
            || self.server_metrics.cpu_usage.usage_percentage > 90.0
            || self.server_metrics.network_metrics.active_connections > 500
    }

    /// Get overall health status
    pub fn health_status(&self) -> &'static str {
        if self.has_critical_issues() {
            "Critical"
        } else if self.server_metrics.memory_usage.usage_percentage > 75.0
            || self.server_metrics.cpu_usage.usage_percentage > 75.0
            || self.server_metrics.network_metrics.active_connections > 100
        {
            "Warning"
        } else {
            "Healthy"
        }
    }
}

#[allow(dead_code)]
impl ServerInfo {
    /// Create new ServerInfo with validation
    pub fn new(
        hostname: String,
        version: String,
        start_time: DateTime<Utc>,
        environment: String,
        os_info: OsInfo,
    ) -> Result<Self, StatusValidationError> {
        let info = ServerInfo {
            hostname,
            version,
            start_time,
            environment,
            os_info,
        };

        info.validate()?;
        Ok(info)
    }

    /// Validate server info
    pub fn validate(&self) -> Result<(), StatusValidationError> {
        // Validate hostname (basic DNS hostname validation)
        if self.hostname.is_empty() 
            || self.hostname.len() > 253
            || !self.hostname.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '.')
            || self.hostname.starts_with('-')
            || self.hostname.ends_with('-')
            || self.hostname.starts_with('.')
            || self.hostname.ends_with('.')
        {
            return Err(StatusValidationError::InvalidHostname {
                hostname: self.hostname.clone(),
            });
        }

        // Validate version (basic semantic versioning check)
        if !self.is_valid_semver(&self.version) {
            return Err(StatusValidationError::InvalidVersion {
                version: self.version.clone(),
            });
        }

        // Validate start time (must be in the past)
        let now = Utc::now();
        if self.start_time > now {
            return Err(StatusValidationError::InvalidStartTime {
                start_time: self.start_time,
            });
        }

        // Validate environment
        match self.environment.as_str() {
            "development" | "staging" | "production" => {},
            _ => return Err(StatusValidationError::InvalidEnvironment {
                environment: self.environment.clone(),
            }),
        }

        // Validate OS info
        self.os_info.validate()?;

        Ok(())
    }

    /// Basic semantic version validation
    fn is_valid_semver(&self, version: &str) -> bool {
        // Basic pattern: X.Y.Z with optional pre-release/build metadata
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() < 3 {
            return false;
        }

        // Check first three parts are numbers
        for part in parts.iter().take(3) {
            if part.parse::<u32>().is_err() {
                return false;
            }
        }

        true
    }

    /// Get server age since start
    pub fn age(&self) -> chrono::Duration {
        Utc::now().signed_duration_since(self.start_time)
    }

    /// Format server age in human-readable format
    pub fn format_age(&self) -> String {
        let age = self.age();
        let days = age.num_days();
        let hours = age.num_hours() % 24;
        let minutes = age.num_minutes() % 60;

        if days > 0 {
            format!("{} days, {} hours, {} minutes", days, hours, minutes)
        } else if hours > 0 {
            format!("{} hours, {} minutes", hours, minutes)
        } else {
            format!("{} minutes", minutes)
        }
    }

    /// Get environment color for UI display
    pub fn environment_color(&self) -> &'static str {
        match self.environment.as_str() {
            "development" => "#4CAF50", // Green
            "staging" => "#FF9800",     // Orange
            "production" => "#F44336",  // Red
            _ => "#9E9E9E",             // Gray
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_status_data_creation() {
        let server_metrics = create_test_metrics();
        let server_info = create_test_server_info();
        
        let status_data = StatusData::new(server_metrics, 5, server_info).unwrap();
        
        assert_eq!(status_data.collection_interval_seconds, 5);
        assert_eq!(status_data.server_info.hostname, "test-server");
    }

    #[test]
    fn test_status_data_invalid_interval() {
        let server_metrics = create_test_metrics();
        let server_info = create_test_server_info();
        
        let result = StatusData::new(server_metrics, 0, server_info);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            StatusValidationError::InvalidCollectionInterval { interval } => {
                assert_eq!(interval, 0);
            }
            _ => panic!("Expected InvalidCollectionInterval error"),
        }
    }

    #[test]
    fn test_server_info_creation() {
        let info = ServerInfo::new(
            "test-server".to_string(),
            "1.0.0".to_string(),
            Utc::now() - chrono::Duration::hours(1),
            "production".to_string(),
            OsInfo::fallback(),
        ).unwrap();

        assert_eq!(info.hostname, "test-server");
        assert_eq!(info.version, "1.0.0");
        assert_eq!(info.environment, "production");
    }

    #[test]
    fn test_server_info_invalid_hostname() {
        let result = ServerInfo::new(
            "".to_string(), // Empty hostname
            "1.0.0".to_string(),
            Utc::now() - chrono::Duration::hours(1),
            "production".to_string(),
            OsInfo::fallback(),
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            StatusValidationError::InvalidHostname { hostname } => {
                assert_eq!(hostname, "");
            }
            _ => panic!("Expected InvalidHostname error"),
        }
    }

    #[test]
    fn test_server_info_invalid_version() {
        let result = ServerInfo::new(
            "test-server".to_string(),
            "invalid-version".to_string(), // Invalid version
            Utc::now() - chrono::Duration::hours(1),
            "production".to_string(),
            OsInfo::fallback(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_server_info_future_start_time() {
        let result = ServerInfo::new(
            "test-server".to_string(),
            "1.0.0".to_string(),
            Utc::now() + chrono::Duration::hours(1), // Future time
            "production".to_string(),
            OsInfo::fallback(),
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            StatusValidationError::InvalidStartTime { .. } => {},
            _ => panic!("Expected InvalidStartTime error"),
        }
    }

    #[test]
    fn test_server_info_invalid_environment() {
        let result = ServerInfo::new(
            "test-server".to_string(),
            "1.0.0".to_string(),
            Utc::now() - chrono::Duration::hours(1),
            "invalid-env".to_string(), // Invalid environment
            OsInfo::fallback(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_health_status() {
        let mut server_metrics = create_test_metrics();
        let server_info = create_test_server_info();

        // Healthy status
        let status_data = StatusData::new(server_metrics.clone(), 5, server_info.clone()).unwrap();
        assert_eq!(status_data.health_status(), "Healthy");
        assert!(!status_data.has_critical_issues());

        // Warning status (high memory)
        server_metrics.memory_usage.usage_percentage = 80.0;
        let status_data = StatusData::new(server_metrics.clone(), 5, server_info.clone()).unwrap();
        assert_eq!(status_data.health_status(), "Warning");

        // Critical status (very high CPU)
        server_metrics.cpu_usage.usage_percentage = 95.0;
        let status_data = StatusData::new(server_metrics, 5, server_info).unwrap();
        assert_eq!(status_data.health_status(), "Critical");
        assert!(status_data.has_critical_issues());
    }

    #[test]
    fn test_formatting() {
        let server_metrics = create_test_metrics();
        let server_info = create_test_server_info();
        let status_data = StatusData::new(server_metrics, 5, server_info).unwrap();

        let uptime_str = status_data.format_uptime();
        assert!(uptime_str.contains("hours") || uptime_str.contains("minutes"));

        let interval_str = status_data.format_collection_interval();
        assert_eq!(interval_str, "every 5 seconds");

        let age_str = status_data.server_info.format_age();
        assert!(age_str.contains("hours") || age_str.contains("minutes"));
    }

    #[test]
    fn test_environment_colors() {
        let dev_info = ServerInfo::new(
            "dev-server".to_string(),
            "1.0.0".to_string(),
            Utc::now() - chrono::Duration::hours(1),
            "development".to_string(),
            OsInfo::fallback(),
        ).unwrap();
        assert_eq!(dev_info.environment_color(), "#4CAF50");

        let prod_info = ServerInfo::new(
            "prod-server".to_string(),
            "1.0.0".to_string(),
            Utc::now() - chrono::Duration::hours(1),
            "production".to_string(),
            OsInfo::fallback(),
        ).unwrap();
        assert_eq!(prod_info.environment_color(), "#F44336");
    }

    #[test]
    fn test_serialization() {
        let server_metrics = create_test_metrics();
        let server_info = create_test_server_info();
        let status_data = StatusData::new(server_metrics, 5, server_info).unwrap();

        let json = serde_json::to_string(&status_data).unwrap();
        let deserialized: StatusData = serde_json::from_str(&json).unwrap();

        assert_eq!(status_data.collection_interval_seconds, deserialized.collection_interval_seconds);
        assert_eq!(status_data.server_info.hostname, deserialized.server_info.hostname);
    }

    // Helper functions
    fn create_test_metrics() -> ServerMetrics {
        use crate::models::{MemoryMetrics, CpuMetrics, NetworkMetrics};
        use crate::models::cpu_metrics::LoadAverage;

        ServerMetrics {
            timestamp: Utc::now(),
            memory_usage: MemoryMetrics {
                total_bytes: 8 * 1024 * 1024 * 1024,
                used_bytes: 4 * 1024 * 1024 * 1024,
                available_bytes: 4 * 1024 * 1024 * 1024,
                usage_percentage: 50.0,
            },
            cpu_usage: CpuMetrics {
                usage_percentage: 25.0,
                core_count: 8,
                load_average: LoadAverage {
                    one_minute: 1.2,
                    five_minute: 1.1,
                    fifteen_minute: 0.9,
                },
            },
            uptime: Duration::from_secs(3600),
            network_metrics: NetworkMetrics {
                bytes_sent: 1024 * 1024,
                bytes_received: 2 * 1024 * 1024,
                packets_sent: 1000,
                packets_received: 1500,
                active_connections: 42,
            },
        }
    }

    fn create_test_server_info() -> ServerInfo {
        ServerInfo::new(
            "test-server".to_string(),
            "1.0.0".to_string(),
            Utc::now() - chrono::Duration::hours(2),
            "development".to_string(),
            OsInfo::fallback(),
        ).unwrap()
    }
}