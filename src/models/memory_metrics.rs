// Memory metrics model
// Memory usage information structure

use serde::{Deserialize, Serialize};

/// Memory usage information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryMetrics {
    /// Total system memory in bytes
    pub total_bytes: u64,
    /// Currently used memory in bytes  
    pub used_bytes: u64,
    /// Available memory in bytes
    pub available_bytes: u64,
    /// Memory usage as percentage (0-100%)
    pub usage_percentage: f32,
}

impl Default for MemoryMetrics {
    fn default() -> Self {
        Self {
            total_bytes: 0,
            used_bytes: 0,
            available_bytes: 0,
            usage_percentage: 0.0,
        }
    }
}

/// Memory validation errors
#[derive(Debug, thiserror::Error)]
pub enum MemoryValidationError {
    #[error("Memory usage inconsistent: used + available ({sum}) > total ({total})")]
    MemoryInconsistent { sum: u64, total: u64 },
    #[error("Memory percentage invalid: {percentage}% (must be 0-100%)")]
    InvalidMemoryPercentage { percentage: f32 },
    #[error("Memory value invalid: {value} (must be >= 0)")]
    #[allow(dead_code)]
    InvalidMemoryValue { value: u64 },
}

#[allow(dead_code)]
impl MemoryMetrics {
    /// Create new MemoryMetrics with validation
    pub fn new(total_bytes: u64, used_bytes: u64, available_bytes: u64) -> Result<Self, MemoryValidationError> {
        // Calculate percentage
        let usage_percentage = if total_bytes > 0 {
            (used_bytes as f32 / total_bytes as f32) * 100.0
        } else {
            0.0
        };

        let metrics = MemoryMetrics {
            total_bytes,
            used_bytes,
            available_bytes,
            usage_percentage,
        };

        metrics.validate()?;
        Ok(metrics)
    }

    /// Validate memory metrics according to business rules
    pub fn validate(&self) -> Result<(), MemoryValidationError> {
        // Memory consistency: used + available ≤ total
        let sum = self.used_bytes + self.available_bytes;
        if sum > self.total_bytes {
            return Err(MemoryValidationError::MemoryInconsistent {
                sum,
                total: self.total_bytes,
            });
        }

        // Memory percentage validation (0-100%)
        if self.usage_percentage < 0.0 || self.usage_percentage > 100.0 {
            return Err(MemoryValidationError::InvalidMemoryPercentage {
                percentage: self.usage_percentage,
            });
        }

        Ok(())
    }

    /// Get memory usage in human-readable format
    pub fn format_usage(&self) -> String {
        format!(
            "{:.1}% ({} / {} GB)",
            self.usage_percentage,
            self.used_bytes / (1024 * 1024 * 1024),
            self.total_bytes / (1024 * 1024 * 1024)
        )
    }

    /// Get available memory in human-readable format
    pub fn format_available(&self) -> String {
        let available_gb = self.available_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        format!("{:.2} GB available", available_gb)
    }

    /// Check if memory usage is critical (>90%)
    pub fn is_critical(&self) -> bool {
        self.usage_percentage > 90.0
    }

    /// Check if memory usage is high (>75%)
    pub fn is_high(&self) -> bool {
        self.usage_percentage > 75.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_metrics_creation_success() {
        let metrics = MemoryMetrics::new(
            8 * 1024 * 1024 * 1024, // 8GB total
            4 * 1024 * 1024 * 1024, // 4GB used
            4 * 1024 * 1024 * 1024, // 4GB available
        ).unwrap();

        assert_eq!(metrics.total_bytes, 8 * 1024 * 1024 * 1024);
        assert_eq!(metrics.used_bytes, 4 * 1024 * 1024 * 1024);
        assert_eq!(metrics.available_bytes, 4 * 1024 * 1024 * 1024);
        assert!((metrics.usage_percentage - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_memory_metrics_inconsistent() {
        let result = MemoryMetrics::new(
            8 * 1024 * 1024 * 1024, // 8GB total
            6 * 1024 * 1024 * 1024, // 6GB used
            4 * 1024 * 1024 * 1024, // 4GB available (used + available > total)
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            MemoryValidationError::MemoryInconsistent { sum, total } => {
                assert_eq!(sum, 10 * 1024 * 1024 * 1024);
                assert_eq!(total, 8 * 1024 * 1024 * 1024);
            }
            _ => panic!("Expected MemoryInconsistent error"),
        }
    }

    #[test]
    fn test_memory_metrics_zero_total() {
        let metrics = MemoryMetrics::new(0, 0, 0).unwrap();
        assert_eq!(metrics.usage_percentage, 0.0);
    }

    #[test]
    fn test_memory_formatting() {
        let metrics = MemoryMetrics::new(
            8 * 1024 * 1024 * 1024, // 8GB total
            3 * 1024 * 1024 * 1024, // 3GB used
            5 * 1024 * 1024 * 1024, // 5GB available
        ).unwrap();

        let usage_str = metrics.format_usage();
        assert!(usage_str.contains("37.5%"));
        assert!(usage_str.contains("3 / 8 GB"));

        let available_str = metrics.format_available();
        assert!(available_str.contains("5.00 GB available"));
    }

    #[test]
    fn test_memory_status_checks() {
        let normal_metrics = MemoryMetrics::new(
            8 * 1024 * 1024 * 1024, // 8GB total
            2 * 1024 * 1024 * 1024, // 2GB used (25%)
            6 * 1024 * 1024 * 1024, // 6GB available
        ).unwrap();
        assert!(!normal_metrics.is_high());
        assert!(!normal_metrics.is_critical());

        let high_metrics = MemoryMetrics::new(
            8 * 1024 * 1024 * 1024, // 8GB total
            6 * 1024 * 1024 * 1024 + 512 * 1024 * 1024, // 6.5GB used (80%)
            1 * 1024 * 1024 * 1024 + 512 * 1024 * 1024, // 1.5GB available
        ).unwrap();
        assert!(high_metrics.is_high());
        assert!(!high_metrics.is_critical());

        let critical_metrics = MemoryMetrics::new(
            8 * 1024 * 1024 * 1024, // 8GB total
            7 * 1024 * 1024 * 1024 + 512 * 1024 * 1024, // 7.5GB used (95%)
            512 * 1024 * 1024, // 0.5GB available
        ).unwrap();
        assert!(critical_metrics.is_high());
        assert!(critical_metrics.is_critical());
    }

    #[test]
    fn test_memory_serialization() {
        let metrics = MemoryMetrics::new(
            8 * 1024 * 1024 * 1024,
            4 * 1024 * 1024 * 1024,
            4 * 1024 * 1024 * 1024,
        ).unwrap();

        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: MemoryMetrics = serde_json::from_str(&json).unwrap();

        assert_eq!(metrics, deserialized);
    }
}