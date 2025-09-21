// CPU metrics model
// CPU utilization and load average information

use serde::{Deserialize, Serialize};

/// CPU utilization information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CpuMetrics {
    /// Current CPU usage percentage (0-100+% for multi-core)
    pub usage_percentage: f32,
    /// Number of CPU cores
    pub core_count: u32,
    /// System load averages
    pub load_average: LoadAverage,
}

impl Default for CpuMetrics {
    fn default() -> Self {
        Self {
            usage_percentage: 0.0,
            core_count: 1,
            load_average: LoadAverage::default(),
        }
    }
}

/// System load average data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoadAverage {
    /// 1-minute load average
    pub one_minute: f32,
    /// 5-minute load average  
    pub five_minute: f32,
    /// 15-minute load average
    pub fifteen_minute: f32,
}

impl Default for LoadAverage {
    fn default() -> Self {
        Self {
            one_minute: 0.0,
            five_minute: 0.0,
            fifteen_minute: 0.0,
        }
    }
}

/// CPU validation errors
#[derive(Debug, thiserror::Error)]
pub enum CpuValidationError {
    #[error("CPU percentage invalid: {percentage}% (must be >= 0)")]
    InvalidCpuPercentage { percentage: f32 },
    #[error("Core count invalid: {count} (must be > 0)")]
    InvalidCoreCount { count: u32 },
    #[error("Load average invalid: {value} (must be >= 0)")]
    InvalidLoadAverage { value: f32 },
}

#[allow(dead_code)]
impl CpuMetrics {
    /// Create new CpuMetrics with validation
    pub fn new(usage_percentage: f32, core_count: u32, load_average: LoadAverage) -> Result<Self, CpuValidationError> {
        let metrics = CpuMetrics {
            usage_percentage,
            core_count,
            load_average,
        };

        metrics.validate()?;
        Ok(metrics)
    }

    /// Validate CPU metrics according to business rules
    pub fn validate(&self) -> Result<(), CpuValidationError> {
        // CPU usage should be non-negative (can exceed 100% for multi-core)
        if self.usage_percentage < 0.0 {
            return Err(CpuValidationError::InvalidCpuPercentage {
                percentage: self.usage_percentage,
            });
        }

        // Core count must be positive
        if self.core_count == 0 {
            return Err(CpuValidationError::InvalidCoreCount {
                count: self.core_count,
            });
        }

        // Load averages must be non-negative
        self.load_average.validate()?;

        Ok(())
    }

    /// Get CPU usage level description
    pub fn usage_level(&self) -> &'static str {
        match self.usage_percentage {
            x if x < 25.0 => "Low",
            x if x < 50.0 => "Normal", 
            x if x < 75.0 => "High",
            x if x < 90.0 => "Very High",
            _ => "Critical",
        }
    }

    /// Check if CPU usage is critical (>90%)
    pub fn is_critical(&self) -> bool {
        self.usage_percentage > 90.0
    }

    /// Check if CPU usage is high (>75%)
    pub fn is_high(&self) -> bool {
        self.usage_percentage > 75.0
    }

    /// Format CPU usage with core information
    pub fn format_usage(&self) -> String {
        format!(
            "{:.1}% ({} cores) - {}",
            self.usage_percentage,
            self.core_count,
            self.usage_level()
        )
    }

    /// Get per-core usage percentage (for multi-core systems)
    pub fn per_core_usage(&self) -> f32 {
        self.usage_percentage / self.core_count as f32
    }
}

#[allow(dead_code)]
impl LoadAverage {
    /// Create new LoadAverage with validation
    pub fn new(one_minute: f32, five_minute: f32, fifteen_minute: f32) -> Result<Self, CpuValidationError> {
        let load_avg = LoadAverage {
            one_minute,
            five_minute,
            fifteen_minute,
        };

        load_avg.validate()?;
        Ok(load_avg)
    }

    /// Validate load averages
    pub fn validate(&self) -> Result<(), CpuValidationError> {
        for &value in &[self.one_minute, self.five_minute, self.fifteen_minute] {
            if value < 0.0 {
                return Err(CpuValidationError::InvalidLoadAverage { value });
            }
        }
        Ok(())
    }

    /// Get load level description based on 1-minute average
    pub fn load_level(&self) -> &'static str {
        match self.one_minute {
            x if x < 1.0 => "Low",
            x if x < 2.0 => "Normal",
            x if x < 4.0 => "High", 
            x if x < 8.0 => "Very High",
            _ => "Critical",
        }
    }

    /// Check if load is trending up (1min > 5min > 15min)
    pub fn is_trending_up(&self) -> bool {
        self.one_minute > self.five_minute && self.five_minute > self.fifteen_minute
    }

    /// Check if load is trending down (1min < 5min < 15min)
    pub fn is_trending_down(&self) -> bool {
        self.one_minute < self.five_minute && self.five_minute < self.fifteen_minute
    }

    /// Format load averages for display
    pub fn format_load(&self) -> String {
        format!(
            "{:.2}, {:.2}, {:.2} (1m, 5m, 15m) - {}",
            self.one_minute,
            self.five_minute,
            self.fifteen_minute,
            self.load_level()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_metrics_creation_success() {
        let load_avg = LoadAverage::new(1.2, 1.1, 0.9).unwrap();
        let metrics = CpuMetrics::new(45.5, 8, load_avg).unwrap();

        assert_eq!(metrics.usage_percentage, 45.5);
        assert_eq!(metrics.core_count, 8);
        assert_eq!(metrics.load_average.one_minute, 1.2);
    }

    #[test]
    fn test_cpu_metrics_invalid_percentage() {
        let load_avg = LoadAverage::new(1.0, 1.0, 1.0).unwrap();
        let result = CpuMetrics::new(-10.0, 4, load_avg);

        assert!(result.is_err());
        match result.unwrap_err() {
            CpuValidationError::InvalidCpuPercentage { percentage } => {
                assert_eq!(percentage, -10.0);
            }
            _ => panic!("Expected InvalidCpuPercentage error"),
        }
    }

    #[test]
    fn test_cpu_metrics_invalid_core_count() {
        let load_avg = LoadAverage::new(1.0, 1.0, 1.0).unwrap();
        let result = CpuMetrics::new(50.0, 0, load_avg);

        assert!(result.is_err());
        match result.unwrap_err() {
            CpuValidationError::InvalidCoreCount { count } => {
                assert_eq!(count, 0);
            }
            _ => panic!("Expected InvalidCoreCount error"),
        }
    }

    #[test]
    fn test_cpu_usage_over_100_valid() {
        let load_avg = LoadAverage::new(2.0, 1.8, 1.5).unwrap();
        let metrics = CpuMetrics::new(350.0, 4, load_avg).unwrap(); // 350% on 4-core system

        assert_eq!(metrics.usage_percentage, 350.0);
        assert_eq!(metrics.per_core_usage(), 87.5); // 350% / 4 cores
    }

    #[test]
    fn test_cpu_usage_levels() {
        let load_avg = LoadAverage::new(1.0, 1.0, 1.0).unwrap();

        let low_cpu = CpuMetrics::new(15.0, 4, load_avg.clone()).unwrap();
        assert_eq!(low_cpu.usage_level(), "Low");
        assert!(!low_cpu.is_high());
        assert!(!low_cpu.is_critical());

        let normal_cpu = CpuMetrics::new(35.0, 4, load_avg.clone()).unwrap();
        assert_eq!(normal_cpu.usage_level(), "Normal");

        let high_cpu = CpuMetrics::new(80.0, 4, load_avg.clone()).unwrap();
        assert_eq!(high_cpu.usage_level(), "Very High");
        assert!(high_cpu.is_high());
        assert!(!high_cpu.is_critical());

        let critical_cpu = CpuMetrics::new(95.0, 4, load_avg).unwrap();
        assert_eq!(critical_cpu.usage_level(), "Critical");
        assert!(critical_cpu.is_critical());
    }

    #[test]
    fn test_load_average_creation() {
        let load_avg = LoadAverage::new(2.1, 1.8, 1.5).unwrap();
        assert_eq!(load_avg.one_minute, 2.1);
        assert_eq!(load_avg.five_minute, 1.8);
        assert_eq!(load_avg.fifteen_minute, 1.5);
    }

    #[test]
    fn test_load_average_invalid() {
        let result = LoadAverage::new(-0.5, 1.0, 1.0);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            CpuValidationError::InvalidLoadAverage { value } => {
                assert_eq!(value, -0.5);
            }
            _ => panic!("Expected InvalidLoadAverage error"),
        }
    }

    #[test]
    fn test_load_average_trends() {
        let trending_up = LoadAverage::new(3.0, 2.0, 1.0).unwrap();
        assert!(trending_up.is_trending_up());
        assert!(!trending_up.is_trending_down());

        let trending_down = LoadAverage::new(1.0, 2.0, 3.0).unwrap();
        assert!(!trending_down.is_trending_up());
        assert!(trending_down.is_trending_down());

        let stable = LoadAverage::new(2.0, 2.0, 2.0).unwrap();
        assert!(!stable.is_trending_up());
        assert!(!stable.is_trending_down());
    }

    #[test]
    fn test_load_average_levels() {
        let low_load = LoadAverage::new(0.5, 0.6, 0.7).unwrap();
        assert_eq!(low_load.load_level(), "Low");

        let normal_load = LoadAverage::new(1.5, 1.4, 1.3).unwrap();
        assert_eq!(normal_load.load_level(), "Normal");

        let high_load = LoadAverage::new(3.0, 2.8, 2.5).unwrap();
        assert_eq!(high_load.load_level(), "High");

        let critical_load = LoadAverage::new(10.0, 9.0, 8.0).unwrap();
        assert_eq!(critical_load.load_level(), "Critical");
    }

    #[test]
    fn test_cpu_formatting() {
        let load_avg = LoadAverage::new(1.5, 1.4, 1.3).unwrap();
        let metrics = CpuMetrics::new(67.3, 4, load_avg).unwrap();

        let usage_str = metrics.format_usage();
        assert!(usage_str.contains("67.3%"));
        assert!(usage_str.contains("4 cores"));
        assert!(usage_str.contains("High"));

        let load_str = metrics.load_average.format_load();
        assert!(load_str.contains("1.50, 1.40, 1.30"));
        assert!(load_str.contains("Normal"));
    }

    #[test]
    fn test_cpu_serialization() {
        let load_avg = LoadAverage::new(1.2, 1.1, 0.9).unwrap();
        let metrics = CpuMetrics::new(45.5, 8, load_avg).unwrap();

        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: CpuMetrics = serde_json::from_str(&json).unwrap();

        assert_eq!(metrics, deserialized);
    }
}