// Server metrics model with validation
// This will initially fail unit tests until implementation is complete

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use super::{MemoryMetrics, CpuMetrics, NetworkMetrics};

/// Represents real-time system performance data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMetrics {
    /// When metrics were collected
    pub timestamp: DateTime<Utc>,
    /// RAM consumption data
    pub memory_usage: MemoryMetrics,
    /// Processor load data
    pub cpu_usage: CpuMetrics,
    /// Time since system/service start
    #[serde(with = "duration_serde")]
    pub uptime: Duration,
    /// Network activity data
    pub network_metrics: NetworkMetrics,
}

/// Validation errors for metrics
#[derive(Debug, thiserror::Error)]
pub enum MetricsValidationError {
    #[error("Memory validation failed: {0}")]
    Memory(#[from] super::memory_metrics::MemoryValidationError),
    #[error("CPU validation failed: {0}")]
    Cpu(#[from] super::cpu_metrics::CpuValidationError),
    #[error("Network validation failed: {0}")]
    Network(#[from] super::network_metrics::NetworkValidationError),
}

impl ServerMetrics {
    /// Validate server metrics according to business rules
    pub fn validate(&self) -> Result<(), MetricsValidationError> {
        // Validate sub-components using their own validation
        self.memory_usage.validate()?;
        self.cpu_usage.validate()?;
        self.network_metrics.validate()?;

        Ok(())
    }

    /// Check if timestamp is stale (for warnings, not blocking validation)
    pub fn is_timestamp_stale(&self) -> Option<i64> {
        let now = Utc::now();
        let age = now.signed_duration_since(self.timestamp);
        if age.num_seconds() > 10 {
            Some(age.num_seconds())
        } else {
            None
        }
    }
}

// Duration serialization module
mod duration_serde {
    use serde::{Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Duration", 2)?;
        state.serialize_field("secs", &duration.as_secs())?;
        state.serialize_field("nanos", &duration.subsec_nanos())?;
        state.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct DurationVisitor;

        impl<'de> Visitor<'de> for DurationVisitor {
            type Value = Duration;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a duration with secs and nanos fields")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Duration, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut secs = None;
                let mut nanos = None;
                
                while let Some(key) = map.next_key()? {
                    match key {
                        "secs" => {
                            if secs.is_some() {
                                return Err(de::Error::duplicate_field("secs"));
                            }
                            secs = Some(map.next_value()?);
                        }
                        "nanos" => {
                            if nanos.is_some() {
                                return Err(de::Error::duplicate_field("nanos"));
                            }
                            nanos = Some(map.next_value()?);
                        }
                        _ => {
                            let _: serde_json::Value = map.next_value()?;
                        }
                    }
                }
                
                let secs = secs.ok_or_else(|| de::Error::missing_field("secs"))?;
                let nanos = nanos.ok_or_else(|| de::Error::missing_field("nanos"))?;
                Ok(Duration::new(secs, nanos))
            }
        }

        deserializer.deserialize_struct("Duration", &["secs", "nanos"], DurationVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::time::Duration;

    fn create_test_memory_metrics() -> MemoryMetrics {
        MemoryMetrics {
            total_bytes: 8_000_000_000,
            used_bytes: 4_000_000_000,
            available_bytes: 4_000_000_000,
            usage_percentage: 50.0,
        }
    }

    fn create_test_cpu_metrics() -> CpuMetrics {
        use super::super::cpu_metrics::LoadAverage;
        CpuMetrics {
            usage_percentage: 25.0,
            core_count: 4,
            load_average: LoadAverage {
                one_minute: 1.0,
                five_minute: 1.2,
                fifteen_minute: 1.1,
            },
        }
    }

    fn create_test_network_metrics() -> NetworkMetrics {
        NetworkMetrics {
            bytes_sent: 1_000_000,
            bytes_received: 2_000_000,
            packets_sent: 1000,
            packets_received: 2000,
            active_connections: 10,
        }
    }

    #[test]
    fn test_server_metrics_creation() {
        let metrics = ServerMetrics {
            timestamp: Utc::now(),
            memory_usage: create_test_memory_metrics(),
            cpu_usage: create_test_cpu_metrics(),
            uptime: Duration::from_secs(3600),
            network_metrics: create_test_network_metrics(),
        };

        assert!(metrics.timestamp <= Utc::now());
        assert_eq!(metrics.memory_usage.usage_percentage, 50.0);
        assert_eq!(metrics.cpu_usage.core_count, 4);
        assert_eq!(metrics.uptime, Duration::from_secs(3600));
    }

    #[test]
    fn test_server_metrics_validation_success() {
        let metrics = ServerMetrics {
            timestamp: Utc::now(),
            memory_usage: create_test_memory_metrics(),
            cpu_usage: create_test_cpu_metrics(),
            uptime: Duration::from_secs(3600),
            network_metrics: create_test_network_metrics(),
        };

        assert!(metrics.validate().is_ok());
    }

    #[test]
    fn test_server_metrics_validation_stale_timestamp() {
        let old_timestamp = Utc::now() - chrono::Duration::seconds(15);
        let metrics = ServerMetrics {
            timestamp: old_timestamp,
            memory_usage: create_test_memory_metrics(),
            cpu_usage: create_test_cpu_metrics(),
            uptime: Duration::from_secs(3600),
            network_metrics: create_test_network_metrics(),
        };

        // Validation should pass (no timestamp check in main validation)
        let result = metrics.validate();
        assert!(result.is_ok());

        // But timestamp staleness should be detected
        let stale_age = metrics.is_timestamp_stale();
        assert!(stale_age.is_some());
        assert!(stale_age.unwrap() >= 15);
    }

    #[test]
    fn test_duration_serialization() {
        let metrics = ServerMetrics {
            timestamp: Utc::now(),
            memory_usage: create_test_memory_metrics(),
            cpu_usage: create_test_cpu_metrics(),
            uptime: Duration::from_secs(3661), // 1 hour, 1 minute, 1 second
            network_metrics: create_test_network_metrics(),
        };

        let json = serde_json::to_string(&metrics).unwrap();
        assert!(json.contains("3661")); // Duration should be serialized as seconds

        let deserialized: ServerMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.uptime, Duration::from_secs(3661));
    }

    #[test]
    fn test_server_metrics_debug() {
        let metrics = ServerMetrics {
            timestamp: Utc::now(),
            memory_usage: create_test_memory_metrics(),
            cpu_usage: create_test_cpu_metrics(),
            uptime: Duration::from_secs(3600),
            network_metrics: create_test_network_metrics(),
        };

        let debug_str = format!("{:?}", metrics);
        assert!(debug_str.contains("ServerMetrics"));
        assert!(debug_str.contains("memory_usage"));
        assert!(debug_str.contains("cpu_usage"));
    }

    #[test]
    fn test_server_metrics_clone() {
        let metrics = ServerMetrics {
            timestamp: Utc::now(),
            memory_usage: create_test_memory_metrics(),
            cpu_usage: create_test_cpu_metrics(),
            uptime: Duration::from_secs(3600),
            network_metrics: create_test_network_metrics(),
        };

        let cloned = metrics.clone();
        assert_eq!(cloned.timestamp, metrics.timestamp);
        assert_eq!(cloned.memory_usage.usage_percentage, metrics.memory_usage.usage_percentage);
        assert_eq!(cloned.cpu_usage.core_count, metrics.cpu_usage.core_count);
        assert_eq!(cloned.uptime, metrics.uptime);
    }
}