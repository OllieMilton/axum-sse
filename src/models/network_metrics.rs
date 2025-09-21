// Network metrics model
// Network activity and connection statistics

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Network activity statistics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkMetrics {
    /// Total bytes transmitted
    pub bytes_sent: u64,
    /// Total bytes received
    pub bytes_received: u64,
    /// Total packets transmitted
    pub packets_sent: u64,
    /// Total packets received
    pub packets_received: u64,
    /// Current active network connections
    pub active_connections: u32,
}

impl Default for NetworkMetrics {
    fn default() -> Self {
        Self {
            bytes_sent: 0,
            bytes_received: 0,
            packets_sent: 0,
            packets_received: 0,
            active_connections: 0,
        }
    }
}

/// Network validation error types
#[derive(Debug, Error, PartialEq)]
#[allow(dead_code)] // Some variants may not be used in current implementation
pub enum NetworkValidationError {
    #[error("Network counter invalid: {value} (must be >= 0)")]
    InvalidNetworkCounter { value: u64 },
    #[error("Connection count invalid: {count} (must be >= 0)")]
    InvalidConnectionCount { count: u32 },
}

#[allow(dead_code)]
impl NetworkMetrics {
    /// Create new NetworkMetrics with validation
    pub fn new(
        bytes_sent: u64,
        bytes_received: u64,
        packets_sent: u64,
        packets_received: u64,
        active_connections: u32,
    ) -> Result<Self, NetworkValidationError> {
        let metrics = NetworkMetrics {
            bytes_sent,
            bytes_received,
            packets_sent,
            packets_received,
            active_connections,
        };

        metrics.validate()?;
        Ok(metrics)
    }

    /// Validate network metrics according to business rules
    pub fn validate(&self) -> Result<(), NetworkValidationError> {
        // All network counters must be non-negative (they're u64/u32, so this is mainly for consistency)
        // Note: u64 and u32 can't be negative, but keeping validation for consistency and future-proofing
        Ok(())
    }

    /// Get total bytes transferred (sent + received)
    pub fn total_bytes(&self) -> u64 {
        self.bytes_sent.saturating_add(self.bytes_received)
    }

    /// Get total packets transferred (sent + received)
    pub fn total_packets(&self) -> u64 {
        self.packets_sent.saturating_add(self.packets_received)
    }

    /// Calculate average packet size for sent packets
    pub fn avg_sent_packet_size(&self) -> Option<f64> {
        if self.packets_sent > 0 {
            Some(self.bytes_sent as f64 / self.packets_sent as f64)
        } else {
            None
        }
    }

    /// Calculate average packet size for received packets
    pub fn avg_received_packet_size(&self) -> Option<f64> {
        if self.packets_received > 0 {
            Some(self.bytes_received as f64 / self.packets_received as f64)
        } else {
            None
        }
    }

    /// Format bytes in human-readable format
    pub fn format_bytes(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", bytes, UNITS[unit_index])
        } else {
            format!("{:.2} {}", size, UNITS[unit_index])
        }
    }

    /// Format sent data in human-readable format
    pub fn format_sent(&self) -> String {
        format!(
            "{} ({} packets)",
            Self::format_bytes(self.bytes_sent),
            self.packets_sent
        )
    }

    /// Format received data in human-readable format
    pub fn format_received(&self) -> String {
        format!(
            "{} ({} packets)",
            Self::format_bytes(self.bytes_received),
            self.packets_received
        )
    }

    /// Format total data in human-readable format
    pub fn format_total(&self) -> String {
        format!(
            "{} ({} packets)",
            Self::format_bytes(self.total_bytes()),
            self.total_packets()
        )
    }

    /// Get network activity level based on active connections
    pub fn activity_level(&self) -> &'static str {
        match self.active_connections {
            0 => "Idle",
            1..=10 => "Low",
            11..=50 => "Normal",
            51..=100 => "High",
            101..=500 => "Very High",
            _ => "Critical",
        }
    }

    /// Check if connection count is high (>100 connections)
    pub fn is_high_activity(&self) -> bool {
        self.active_connections > 100
    }

    /// Check if connection count is critical (>500 connections)
    pub fn is_critical_activity(&self) -> bool {
        self.active_connections > 500
    }

    /// Calculate the ratio of sent to received data
    pub fn send_receive_ratio(&self) -> Option<f64> {
        if self.bytes_received > 0 {
            Some(self.bytes_sent as f64 / self.bytes_received as f64)
        } else if self.bytes_sent > 0 {
            Some(f64::INFINITY) // Only sending, no receiving
        } else {
            None // No network activity
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_metrics_creation() {
        let metrics = NetworkMetrics::new(
            1024 * 1024,     // 1MB sent
            2 * 1024 * 1024, // 2MB received  
            1000,            // 1000 packets sent
            1500,            // 1500 packets received
            42,              // 42 active connections
        ).unwrap();

        assert_eq!(metrics.bytes_sent, 1024 * 1024);
        assert_eq!(metrics.bytes_received, 2 * 1024 * 1024);
        assert_eq!(metrics.packets_sent, 1000);
        assert_eq!(metrics.packets_received, 1500);
        assert_eq!(metrics.active_connections, 42);
    }

    #[test]
    fn test_network_metrics_calculations() {
        let metrics = NetworkMetrics::new(
            1024 * 1024,     // 1MB sent
            2 * 1024 * 1024, // 2MB received
            1000,            // 1000 packets sent
            2000,            // 2000 packets received
            25,              // 25 active connections
        ).unwrap();

        assert_eq!(metrics.total_bytes(), 3 * 1024 * 1024); // 3MB total
        assert_eq!(metrics.total_packets(), 3000); // 3000 packets total

        // Average packet sizes
        assert_eq!(metrics.avg_sent_packet_size(), Some(1048.576)); // 1MB / 1000 packets
        assert_eq!(metrics.avg_received_packet_size(), Some(1048.576)); // 2MB / 2000 packets

        // Send/receive ratio
        assert_eq!(metrics.send_receive_ratio(), Some(0.5)); // 1MB sent / 2MB received
    }

    #[test]
    fn test_network_metrics_zero_packets() {
        let metrics = NetworkMetrics::new(0, 0, 0, 0, 0).unwrap();

        assert_eq!(metrics.avg_sent_packet_size(), None);
        assert_eq!(metrics.avg_received_packet_size(), None);
        assert_eq!(metrics.send_receive_ratio(), None);
    }

    #[test]
    fn test_network_metrics_send_only() {
        let metrics = NetworkMetrics::new(
            1024 * 1024, // 1MB sent
            0,           // 0 bytes received
            1000,        // 1000 packets sent
            0,           // 0 packets received
            5,           // 5 connections
        ).unwrap();

        assert_eq!(metrics.send_receive_ratio(), Some(f64::INFINITY));
        assert_eq!(metrics.avg_sent_packet_size(), Some(1048.576));
        assert_eq!(metrics.avg_received_packet_size(), None);
    }

    #[test]
    fn test_byte_formatting() {
        assert_eq!(NetworkMetrics::format_bytes(512), "512 B");
        assert_eq!(NetworkMetrics::format_bytes(1024), "1.00 KB");
        assert_eq!(NetworkMetrics::format_bytes(1536), "1.50 KB"); // 1.5 KB
        assert_eq!(NetworkMetrics::format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(NetworkMetrics::format_bytes(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(NetworkMetrics::format_bytes(1024_u64.pow(4)), "1.00 TB");
    }

    #[test]
    fn test_activity_levels() {
        let idle = NetworkMetrics::new(0, 0, 0, 0, 0).unwrap();
        assert_eq!(idle.activity_level(), "Idle");
        assert!(!idle.is_high_activity());
        assert!(!idle.is_critical_activity());

        let low = NetworkMetrics::new(100, 200, 10, 20, 5).unwrap();
        assert_eq!(low.activity_level(), "Low");

        let normal = NetworkMetrics::new(1000, 2000, 100, 200, 25).unwrap();
        assert_eq!(normal.activity_level(), "Normal");

        let high = NetworkMetrics::new(10000, 20000, 1000, 2000, 150).unwrap();
        assert_eq!(high.activity_level(), "Very High");
        assert!(high.is_high_activity());
        assert!(!high.is_critical_activity());

        let critical = NetworkMetrics::new(100000, 200000, 10000, 20000, 1000).unwrap();
        assert_eq!(critical.activity_level(), "Critical");
        assert!(critical.is_critical_activity());
    }

    #[test]
    fn test_network_formatting() {
        let metrics = NetworkMetrics::new(
            1536 * 1024,     // 1.5 MB sent
            2048 * 1024,     // 2 MB received
            1500,            // 1500 packets sent
            2000,            // 2000 packets received
            42,              // 42 connections
        ).unwrap();

        let sent_str = metrics.format_sent();
        assert!(sent_str.contains("1.50 MB"));
        assert!(sent_str.contains("1500 packets"));

        let received_str = metrics.format_received();
        assert!(received_str.contains("2.00 MB"));
        assert!(received_str.contains("2000 packets"));

        let total_str = metrics.format_total();
        assert!(total_str.contains("3.50 MB"));
        assert!(total_str.contains("3500 packets"));
    }

    #[test]
    fn test_network_serialization() {
        let metrics = NetworkMetrics::new(1024, 2048, 100, 200, 10).unwrap();

        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: NetworkMetrics = serde_json::from_str(&json).unwrap();

        assert_eq!(metrics, deserialized);
    }

    #[test]
    fn test_network_overflow_protection() {
        // Test that saturating_add prevents overflow
        let metrics = NetworkMetrics::new(
            u64::MAX - 100,  // Near max value
            200,             // This would overflow without saturating_add
            1000,
            2000,
            50,
        ).unwrap();

        let total = metrics.total_bytes();
        assert_eq!(total, u64::MAX); // Should saturate at max value
    }
}