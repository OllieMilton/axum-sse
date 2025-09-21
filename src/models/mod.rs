// Re-export all models
pub mod time_event;
pub mod connection_state;

pub use time_event::TimeEvent;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationState {
    pub current_page: String,
}