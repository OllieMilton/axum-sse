// Time Event model with UK formatting support
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimeEvent {
    pub timestamp: DateTime<Utc>,
    pub formatted_time: String,
}

impl TimeEvent {
    /// Create a new TimeEvent with current time formatted for UK locale
    pub fn new() -> Self {
        let now = Utc::now();
        Self::from_timestamp(now)
    }

    /// Create a TimeEvent from a specific timestamp
    pub fn from_timestamp(timestamp: DateTime<Utc>) -> Self {
        let formatted_time = Self::format_uk_time(&timestamp);
        Self {
            timestamp,
            formatted_time,
        }
    }

    /// Format timestamp as UK date/time: DD/MM/YYYY HH:MM:SS
    fn format_uk_time(timestamp: &DateTime<Utc>) -> String {
        timestamp.format("%d/%m/%Y %H:%M:%S").to_string()
    }
}

impl Default for TimeEvent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_new_time_event_has_current_time() {
        let before = Utc::now();
        let event = TimeEvent::new();
        let after = Utc::now();

        assert!(event.timestamp >= before);
        assert!(event.timestamp <= after);
    }

    #[test]
    fn test_uk_time_formatting() {
        // Test with a known timestamp: 2025-09-20 10:30:45 UTC
        let timestamp = Utc.with_ymd_and_hms(2025, 9, 20, 10, 30, 45).unwrap();
        let event = TimeEvent::from_timestamp(timestamp);

        assert_eq!(event.formatted_time, "20/09/2025 10:30:45");
    }

    #[test]
    fn test_serialization() {
        let event = TimeEvent::new();
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: TimeEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.timestamp, deserialized.timestamp);
        assert_eq!(event.formatted_time, deserialized.formatted_time);
    }
}