// Health status enumeration for system status reporting

use serde::{Deserialize, Serialize};

/// Represents the overall health status of the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// All systems operating normally
    Healthy,
    /// Some issues detected but system is operational
    Warning,
    /// Critical issues detected requiring attention
    Critical,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self::Healthy
    }
}

impl HealthStatus {
    /// Get the string representation for API responses
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Warning => "warning",
            Self::Critical => "critical",
        }
    }

    /// Get the status based on system metrics
    pub fn from_metrics(cpu_usage: f32, memory_usage: f32) -> Self {
        if cpu_usage > 90.0 || memory_usage > 95.0 {
            Self::Critical
        } else if cpu_usage > 70.0 || memory_usage > 80.0 {
            Self::Warning
        } else {
            Self::Healthy
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_default() {
        assert_eq!(HealthStatus::default(), HealthStatus::Healthy);
    }

    #[test]
    fn test_health_status_as_str() {
        assert_eq!(HealthStatus::Healthy.as_str(), "healthy");
        assert_eq!(HealthStatus::Warning.as_str(), "warning");
        assert_eq!(HealthStatus::Critical.as_str(), "critical");
    }

    #[test]
    fn test_health_status_from_metrics_healthy() {
        let status = HealthStatus::from_metrics(50.0, 60.0);
        assert_eq!(status, HealthStatus::Healthy);
    }

    #[test]
    fn test_health_status_from_metrics_warning() {
        let status = HealthStatus::from_metrics(75.0, 85.0);
        assert_eq!(status, HealthStatus::Warning);
    }

    #[test]
    fn test_health_status_from_metrics_critical() {
        let status = HealthStatus::from_metrics(95.0, 97.0);
        assert_eq!(status, HealthStatus::Critical);
    }

    #[test]
    fn test_health_status_serialization() {
        let status = HealthStatus::Warning;
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: HealthStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);
    }
}