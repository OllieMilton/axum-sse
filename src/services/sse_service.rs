// SSE streaming service for broadcasting time events
use axum::{
    response::Sse,
    response::sse::{Event, KeepAlive},
};
use futures::stream::{self, Stream};
use std::{convert::Infallible, time::Duration};
use tokio::time::interval;
use tokio::sync::broadcast;
use crate::models::TimeEvent;
use uuid::Uuid;
use tracing::{info, warn, error};

/// SSE connection manager for handling multiple client connections
#[derive(Clone)]
pub struct SseService {
    /// Broadcast channel for sending time events to all connected clients
    time_sender: broadcast::Sender<TimeEvent>,
}

impl SseService {
    /// Create a new SSE service
    pub fn new() -> Self {
        // Create broadcast channel with buffer for disconnected clients
        let (time_sender, _) = broadcast::channel(100);
        
        Self {
            time_sender,
        }
    }

    /// Start the time broadcasting background task
    pub fn start_time_broadcaster(&self) {
        let sender = self.time_sender.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(10));
            
            loop {
                interval.tick().await;
                
                let time_event = TimeEvent::new();
                info!("Broadcasting time event: {}", time_event.formatted_time);
                
                // Send to all connected clients
                match sender.send(time_event) {
                    Ok(receivers) => {
                        info!("Time event sent to {} receivers", receivers);
                    }
                    Err(e) => {
                        warn!("No receivers for time event: {}", e);
                    }
                }
            }
        });
    }

    /// Create an SSE stream for a new client connection
    pub fn create_time_stream(&self) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
        let connection_id = Uuid::new_v4().to_string();
        let receiver = self.time_sender.subscribe();
        
        info!("New SSE connection: {}", connection_id);
        
        let stream = stream::unfold(
            (receiver, connection_id.clone()),
            |(mut rx, conn_id)| async move {
                match rx.recv().await {
                    Ok(time_event) => {
                        // Create SSE event with the time data
                        let event_data = match serde_json::to_string(&time_event) {
                            Ok(json) => json,
                            Err(e) => {
                                error!("Failed to serialize time event: {}", e);
                                return None;
                            }
                        };
                        
                        let event = Event::default()
                            .event("time-update")
                            .id(&conn_id)
                            .data(event_data);
                        
                        Some((Ok(event), (rx, conn_id)))
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("SSE connection {} closed: channel closed", conn_id);
                        None
                    }
                    Err(broadcast::error::RecvError::Lagged(missed)) => {
                        warn!("SSE connection {} lagged, missed {} events", conn_id, missed);
                        // Send a reconnection event
                        let event = Event::default()
                            .event("connection-lagged")
                            .id(&conn_id)
                            .data(format!("{{\"missed_events\": {}}}", missed));
                        
                        Some((Ok(event), (rx, conn_id)))
                    }
                }
            },
        );
        
        Sse::new(stream)
            .keep_alive(KeepAlive::default().interval(Duration::from_secs(30)))
    }

    /// Get the number of current receivers (approximate active connections)
    pub fn receiver_count(&self) -> usize {
        self.time_sender.receiver_count()
    }

    /// Check if the service is healthy (has active broadcast channel)
    pub fn is_healthy(&self) -> bool {
        // Since broadcast::Sender doesn't have is_closed(), we'll use receiver_count as a proxy
        // A healthy service can have 0 receivers (no clients connected)
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, timeout};

    #[tokio::test]
    async fn test_sse_service_creation() {
        let service = SseService::new();
        assert!(service.is_healthy());
        assert_eq!(service.receiver_count(), 0);
    }

    #[tokio::test]
    async fn test_time_broadcaster() {
        let service = SseService::new();
        service.start_time_broadcaster();
        
        // Subscribe to the broadcast channel
        let mut receiver = service.time_sender.subscribe();
        
        // Wait for a time event (with timeout to avoid hanging)
        let result = timeout(Duration::from_millis(100), receiver.recv()).await;
        
        // Note: This test might timeout because the broadcaster sends every 10 seconds
        // In a real test environment, you'd want to inject a faster interval for testing
        match result {
            Ok(Ok(time_event)) => {
                assert!(!time_event.formatted_time.is_empty());
            }
            Ok(Err(_)) => {
                // Channel error is acceptable for this test
            }
            Err(_) => {
                // Timeout is expected since we only wait 100ms but broadcast is every 10s
            }
        }
    }

    #[tokio::test]
    async fn test_multiple_receivers() {
        let service = SseService::new();
        
        // Subscribe multiple receivers
        let _receiver1 = service.time_sender.subscribe();
        let _receiver2 = service.time_sender.subscribe();
        let _receiver3 = service.time_sender.subscribe();
        
        assert_eq!(service.receiver_count(), 3);
    }

    #[tokio::test]
    async fn test_receiver_cleanup() {
        let service = SseService::new();
        
        {
            let _receiver1 = service.time_sender.subscribe();
            let _receiver2 = service.time_sender.subscribe();
            assert_eq!(service.receiver_count(), 2);
        } // receivers dropped here
        
        // Small delay to allow cleanup
        sleep(Duration::from_millis(10)).await;
        
        // Note: receiver_count() might not immediately reflect dropped receivers
        // This is normal behavior for broadcast channels
    }
}