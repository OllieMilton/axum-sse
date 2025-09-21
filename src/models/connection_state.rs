// Connection State types for managing SSE connection status
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConnectionState {
    pub connected: bool,
    pub last_ping: Option<DateTime<Utc>>,
    pub connection_id: Option<String>,
    pub failed_attempts: u32,
}

impl ConnectionState {
    /// Create a new disconnected state
    pub fn new() -> Self {
        Self {
            connected: false,
            last_ping: None,
            connection_id: None,
            failed_attempts: 0,
        }
    }
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self::new()
    }
}