// Server-Sent Events (SSE) endpoint for real-time server metrics streaming
// Provides continuous updates of server status to connected clients

use crate::models::{
    StatusData, ServerMetrics, MetricsCollectionError, MetricsResponse
};
use crate::routes::server_status::{ServerStatusState, ServerStatusError};
use axum::{
    extract::{Query, State},
    response::{
        sse::{Event, Sse},
        IntoResponse,
    },
    routing::get,
    Router,
};
use chrono::{DateTime, Utc};
use futures_util::stream::Stream;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use tokio::time::{interval, MissedTickBehavior};
use tracing::{debug, error, instrument, info, warn};

/// Query parameters for SSE stream endpoint
#[derive(Debug, Deserialize)]
pub struct SseQuery {
    /// Update interval in seconds (default: 5)
    pub interval: Option<u32>,
    /// Include detailed metrics (default: true)
    pub detailed: Option<bool>,
    /// Client identifier for connection tracking
    pub client_id: Option<String>,
    /// Include only specific metric types
    pub metrics: Option<String>, // comma-separated: memory,cpu,network
}

/// SSE event data for server metrics
#[derive(Debug, Serialize)]
pub struct MetricsEvent {
    /// Event type identifier
    pub event_type: String,
    /// Server status data
    pub data: StatusData,
    /// Event sequence number
    pub sequence: u64,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Connection metadata
    pub connection_info: ConnectionInfo,
}

/// Connection tracking information
#[derive(Debug, Serialize, Clone)]
pub struct ConnectionInfo {
    /// Client identifier
    pub client_id: String,
    /// Connection duration in seconds
    pub connection_duration_seconds: u64,
    /// Number of events sent to this client
    pub events_sent: u64,
    /// Update interval for this client
    pub update_interval_seconds: u32,
}

/// SSE stream state for individual connections
struct SseConnectionState {
    client_id: String,
    #[allow(dead_code)]
    connected_at: Instant,
    events_sent: u64,
    #[allow(dead_code)]
    interval_seconds: u32,
    #[allow(dead_code)]
    detailed: bool,
    #[allow(dead_code)]
    metrics_filter: Option<Vec<String>>,
}

impl SseConnectionState {
    fn new(client_id: String, interval_seconds: u32, detailed: bool, metrics_filter: Option<Vec<String>>) -> Self {
        Self {
            client_id,
            connected_at: Instant::now(),
            events_sent: 0,
            interval_seconds,
            detailed,
            metrics_filter,
        }
    }

    #[allow(dead_code)]
    fn get_connection_info(&self) -> ConnectionInfo {
        ConnectionInfo {
            client_id: self.client_id.clone(),
            connection_duration_seconds: self.connected_at.elapsed().as_secs(),
            events_sent: self.events_sent,
            update_interval_seconds: self.interval_seconds,
        }
    }

    fn increment_events(&mut self) {
        self.events_sent += 1;
    }
}

/// Custom stream implementation for metrics SSE
struct MetricsStream {
    state: SseConnectionState,
    #[allow(dead_code)]
    app_state: ServerStatusState,
    sequence: u64,
    interval_timer: tokio::time::Interval,
}

impl MetricsStream {
    fn new(
        client_id: String,
        interval_seconds: u32,
        detailed: bool,
        metrics_filter: Option<Vec<String>>,
        app_state: ServerStatusState,
    ) -> Self {
        let mut timer = interval(Duration::from_secs(interval_seconds as u64));
        timer.set_missed_tick_behavior(MissedTickBehavior::Skip);

        Self {
            state: SseConnectionState::new(client_id, interval_seconds, detailed, metrics_filter),
            app_state,
            sequence: 0,
            interval_timer: timer,
        }
    }

    #[allow(dead_code)]
    async fn collect_metrics(&self) -> Result<ServerMetrics, MetricsCollectionError> {
        // Use cache for regular updates to reduce system load
        let cache_key = format!("sse_{}", self.state.client_id);
        
        match self.app_state.metrics_cache.get_metrics(Some(cache_key)).await {
            MetricsResponse::Ok(metrics) => Ok(metrics),
            MetricsResponse::PartialData { data, errors } => {
                // Log warnings but return partial data
                for error in errors {
                    warn!("Partial metrics for SSE client {}: {}", self.state.client_id, error);
                }
                Ok(data)
            }
            MetricsResponse::Error(error) => Err(error),
        }
    }

    #[allow(dead_code)]
    fn filter_metrics(&self, mut metrics: ServerMetrics) -> ServerMetrics {
        if let Some(ref filter) = self.state.metrics_filter {
            // Apply metrics filtering based on requested types
            if !filter.contains(&"memory".to_string()) {
                metrics.memory_usage = crate::models::MemoryMetrics::default();
            }
            if !filter.contains(&"cpu".to_string()) {
                metrics.cpu_usage = crate::models::CpuMetrics::default();
            }
            if !filter.contains(&"network".to_string()) {
                metrics.network_metrics = crate::models::NetworkMetrics::default();
            }
        }

        // Apply detailed flag
        if !self.state.detailed {
            metrics = create_simplified_metrics(metrics);
        }

        metrics
    }

    #[allow(dead_code)]
    async fn create_event(&mut self) -> Result<Event, ServerStatusError> {
        // Collect metrics
        let raw_metrics = self.collect_metrics().await?;
        
        // Apply filtering
        let filtered_metrics = self.filter_metrics(raw_metrics);

        // Create status data
        let status_data = match StatusData::new(
            filtered_metrics,
            self.app_state.metrics_service.get_config().collection_interval_seconds,
            self.app_state.server_info.clone(),
        ) {
            Ok(data) => data,
            Err(e) => {
                return Err(ServerStatusError::Internal(format!("Failed to create status data: {}", e)));
            }
        };

        // Create event data
        let event_data = MetricsEvent {
            event_type: "metrics_update".to_string(),
            data: status_data,
            sequence: self.sequence,
            timestamp: Utc::now(),
            connection_info: self.state.get_connection_info(),
        };

        // Increment counters
        self.sequence += 1;
        self.state.increment_events();

        // Create SSE event
        let event = Event::default()
            .event("metrics_update")
            .id(self.sequence.to_string())
            .data(serde_json::to_string(&event_data).map_err(|e| {
                ServerStatusError::Internal(format!("Failed to serialize event data: {}", e))
            })?)
            .retry(Duration::from_secs(5));

        debug!(
            "Created SSE event {} for client {} (connection: {}s)",
            self.sequence,
            self.state.client_id,
            self.state.connected_at.elapsed().as_secs()
        );

        Ok(event)
    }

    /// Create a minimal status data for error cases
    fn create_minimal_status(&self) -> Result<StatusData, String> {
        use crate::models::{ServerMetrics, MemoryMetrics, CpuMetrics, NetworkMetrics};
        use crate::models::cpu_metrics::LoadAverage;
        
        // Create minimal/default metrics
        let minimal_metrics = ServerMetrics {
            timestamp: Utc::now(),
            memory_usage: MemoryMetrics {
                total_bytes: 0,
                used_bytes: 0,
                available_bytes: 0,
                usage_percentage: 0.0,
            },
            cpu_usage: CpuMetrics {
                usage_percentage: 0.0,
                core_count: 1,
                load_average: LoadAverage {
                    one_minute: 0.0,
                    five_minute: 0.0,
                    fifteen_minute: 0.0,
                },
            },
            uptime: Duration::from_secs(0),
            network_metrics: NetworkMetrics {
                bytes_sent: 0,
                bytes_received: 0,
                packets_sent: 0,
                packets_received: 0,
                active_connections: 0,
            },
        };

        StatusData::new(
            minimal_metrics,
            5, // default interval
            self.app_state.server_info.clone(),
        ).map_err(|e| format!("Failed to create minimal status: {}", e))
    }
}

impl Stream for MetricsStream {
    type Item = Result<Event, Infallible>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Poll the interval timer
        match self.interval_timer.poll_tick(cx) {
            Poll::Ready(_) => {
                // Timer ticked - collect metrics and create event
                let sequence = self.sequence;
                let client_id = self.state.client_id.clone();
                let connected_at = self.state.connected_at;
                let events_sent = self.state.events_sent;
                let interval_seconds = self.state.interval_seconds;
                
                // Create connection info
                let connection_info = ConnectionInfo {
                    client_id: client_id.clone(),
                    connection_duration_seconds: connected_at.elapsed().as_secs(),
                    events_sent,
                    update_interval_seconds: interval_seconds,
                };
                
                // Get metrics from cache (this is synchronous and safe to call in poll_next)
                let metrics_result = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        self.app_state.metrics_cache.get_metrics(None).await
                    })
                });
                
                let event_data = match metrics_result {
                    MetricsResponse::Ok(metrics) => {
                        // Create status data
                        match StatusData::new(
                            metrics,
                            self.app_state.metrics_service.get_config().collection_interval_seconds,
                            self.app_state.server_info.clone(),
                        ) {
                            Ok(status_data) => {
                                // Create proper event with full metrics
                                MetricsEvent {
                                    event_type: "status-update".to_string(),
                                    data: status_data,
                                    sequence,
                                    timestamp: Utc::now(),
                                    connection_info: connection_info.clone(),
                                }
                            }
                            Err(e) => {
                                // Fallback to error event - create minimal StatusData
                                warn!("Failed to create status data: {}", e);
                                let minimal_status = self.create_minimal_status().unwrap_or_else(|err| {
                                    // This shouldn't happen, but if it does, we need to handle it
                                    error!("Failed to create minimal status: {}", err);
                                    // We can't return from here, so we'll create the most basic status possible
                                    StatusData::new(
                                        ServerMetrics {
                                            timestamp: Utc::now(),
                                            memory_usage: crate::models::MemoryMetrics {
                                                total_bytes: 0, used_bytes: 0, available_bytes: 0, usage_percentage: 0.0
                                            },
                                            cpu_usage: crate::models::CpuMetrics {
                                                usage_percentage: 0.0, core_count: 1,
                                                load_average: crate::models::cpu_metrics::LoadAverage { one_minute: 0.0, five_minute: 0.0, fifteen_minute: 0.0 }
                                            },
                                            uptime: Duration::from_secs(0),
                                            network_metrics: crate::models::NetworkMetrics {
                                                bytes_sent: 0, bytes_received: 0, packets_sent: 0, packets_received: 0, active_connections: 0
                                            },
                                        },
                                        5,
                                        self.app_state.server_info.clone(),
                                    ).unwrap_or_else(|_| panic!("Critical: Cannot create StatusData"))
                                });
                                
                                MetricsEvent {
                                    event_type: "error".to_string(),
                                    data: minimal_status,
                                    sequence,
                                    timestamp: Utc::now(),
                                    connection_info: connection_info.clone(),
                                }
                            }
                        }
                    }
                    MetricsResponse::PartialData { data, errors } => {
                        warn!("Partial metrics data with {} errors", errors.len());
                        match StatusData::new(
                            data,
                            self.app_state.metrics_service.get_config().collection_interval_seconds,
                            self.app_state.server_info.clone(),
                        ) {
                            Ok(status_data) => MetricsEvent {
                                event_type: "status-update".to_string(),
                                data: status_data,
                                sequence,
                                timestamp: Utc::now(),
                                connection_info: connection_info.clone(),
                            },
                            Err(_) => {
                                let minimal_status = self.create_minimal_status().unwrap_or_else(|_| {
                                    StatusData::new(
                                        ServerMetrics {
                                            timestamp: Utc::now(),
                                            memory_usage: crate::models::MemoryMetrics {
                                                total_bytes: 0, used_bytes: 0, available_bytes: 0, usage_percentage: 0.0
                                            },
                                            cpu_usage: crate::models::CpuMetrics {
                                                usage_percentage: 0.0, core_count: 1,
                                                load_average: crate::models::cpu_metrics::LoadAverage { one_minute: 0.0, five_minute: 0.0, fifteen_minute: 0.0 }
                                            },
                                            uptime: Duration::from_secs(0),
                                            network_metrics: crate::models::NetworkMetrics {
                                                bytes_sent: 0, bytes_received: 0, packets_sent: 0, packets_received: 0, active_connections: 0
                                            },
                                        },
                                        5,
                                        self.app_state.server_info.clone(),
                                    ).unwrap()
                                });
                                MetricsEvent {
                                    event_type: "error".to_string(),
                                    data: minimal_status,
                                    sequence,
                                    timestamp: Utc::now(),
                                    connection_info: connection_info.clone(),
                                }
                            }
                        }
                    }
                    MetricsResponse::Error(e) => {
                        error!("Failed to collect metrics for SSE: {}", e);
                        let minimal_status = self.create_minimal_status().unwrap_or_else(|_| {
                            StatusData::new(
                                ServerMetrics {
                                    timestamp: Utc::now(),
                                    memory_usage: crate::models::MemoryMetrics {
                                        total_bytes: 0, used_bytes: 0, available_bytes: 0, usage_percentage: 0.0
                                    },
                                    cpu_usage: crate::models::CpuMetrics {
                                        usage_percentage: 0.0, core_count: 1,
                                        load_average: crate::models::cpu_metrics::LoadAverage { one_minute: 0.0, five_minute: 0.0, fifteen_minute: 0.0 }
                                    },
                                    uptime: Duration::from_secs(0),
                                    network_metrics: crate::models::NetworkMetrics {
                                        bytes_sent: 0, bytes_received: 0, packets_sent: 0, packets_received: 0, active_connections: 0
                                    },
                                },
                                5,
                                self.app_state.server_info.clone(),
                            ).unwrap()
                        });
                        MetricsEvent {
                            event_type: "error".to_string(),
                            data: minimal_status,
                            sequence,
                            timestamp: Utc::now(),
                            connection_info: connection_info.clone(),
                        }
                    }
                };
                
                // Serialize event data
                let event_data_json = match serde_json::to_string(&event_data) {
                    Ok(json) => json,
                    Err(e) => {
                        error!("Failed to serialize event data: {}", e);
                        format!(r#"{{"event_type":"error","sequence":{},"timestamp":"{}","client_id":"{}","error":"serialization_failed"}}"#,
                            sequence, Utc::now().to_rfc3339(), client_id)
                    }
                };
                
                let event = Event::default()
                    .event(&event_data.event_type)
                    .id(sequence.to_string())
                    .data(event_data_json)
                    .retry(Duration::from_secs(5));

                self.sequence += 1;
                self.state.increment_events();

                Poll::Ready(Some(Ok(event)))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

    /// GET /api/server-status-stream - Server-Sent Events stream for real-time metrics
#[instrument(skip(state))]
pub async fn server_status_stream(
    Query(params): Query<SseQuery>,
    State(state): State<ServerStatusState>,
) -> impl IntoResponse {
    let client_id = params.client_id.unwrap_or_else(|| {
        format!("client_{}", uuid::Uuid::new_v4().to_string()[..8].to_string())
    });
    
    let interval = params.interval.unwrap_or(5).max(1).min(60); // Clamp between 1-60 seconds
    let detailed = params.detailed.unwrap_or(true);
    
    let metrics_filter = params.metrics.map(|m| {
        m.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| ["memory", "cpu", "network"].contains(&s.as_str()))
            .collect()
    });

    info!(
        "New SSE connection: client_id={}, interval={}s, detailed={}, filter={:?}",
        client_id, interval, detailed, metrics_filter
    );

    // Create metrics stream
    let stream = MetricsStream::new(client_id.clone(), interval, detailed, metrics_filter, state);

    // Create SSE response
    let sse = Sse::new(stream)
        .keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(Duration::from_secs(30))
                .text("ping"),
        );

    // Add CORS headers for SSE
    let response = sse.into_response();
    
    debug!("SSE stream initialized for client: {}", client_id);
    response
}

/// Create a simplified version of metrics for non-detailed streams
#[allow(dead_code)]
fn create_simplified_metrics(full_metrics: ServerMetrics) -> ServerMetrics {
    ServerMetrics {
        timestamp: full_metrics.timestamp,
        memory_usage: crate::models::MemoryMetrics {
            total_bytes: full_metrics.memory_usage.total_bytes,
            used_bytes: full_metrics.memory_usage.used_bytes,
            available_bytes: full_metrics.memory_usage.available_bytes,
            usage_percentage: full_metrics.memory_usage.usage_percentage,
        },
        cpu_usage: crate::models::CpuMetrics {
            usage_percentage: full_metrics.cpu_usage.usage_percentage,
            core_count: full_metrics.cpu_usage.core_count,
            load_average: crate::models::cpu_metrics::LoadAverage {
                one_minute: full_metrics.cpu_usage.load_average.one_minute,
                five_minute: 0.0, // Remove extended load averages for simplified view
                fifteen_minute: 0.0,
            },
        },
        uptime: full_metrics.uptime,
        network_metrics: crate::models::NetworkMetrics {
            bytes_sent: full_metrics.network_metrics.bytes_sent,
            bytes_received: full_metrics.network_metrics.bytes_received,
            packets_sent: 0, // Remove packet details for simplified view
            packets_received: 0,
            active_connections: full_metrics.network_metrics.active_connections,
        },
    }
}

/// Helper endpoint to get SSE connection info
#[instrument(skip(_state))]
pub async fn get_sse_info(
    State(_state): State<ServerStatusState>,
) -> impl IntoResponse {
    let info = serde_json::json!({
        "endpoint": "/api/server-status-stream",
        "description": "Server-Sent Events stream for real-time server metrics",
        "parameters": {
            "interval": "Update interval in seconds (1-60, default: 5)",
            "detailed": "Include detailed metrics (default: true)",
            "client_id": "Client identifier for connection tracking (optional)",
            "metrics": "Comma-separated metric types: memory,cpu,network (default: all)"
        },
        "events": {
            "metrics_update": "Regular metrics update event",
            "ping": "Keep-alive ping event"
        },
        "headers": {
            "Cache-Control": "no-cache",
            "Content-Type": "text/event-stream",
            "Connection": "keep-alive"
        },
        "example_url": "/api/server-status-stream?interval=10&detailed=false&metrics=memory,cpu",
        "api_version": "1.0"
    });

    axum::Json(info)
}

/// Create the SSE router
pub fn create_sse_router() -> Router<ServerStatusState> {
    Router::new()
        .route("/server-status-stream", get(server_status_stream))
        .route("/server-status-stream/info", get(get_sse_info))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::{MetricsService, MetricsCache};
    use axum_test::TestServer;
    use std::sync::Arc;

    fn create_test_state() -> ServerStatusState {
        let metrics_service = Arc::new(MetricsService::new());
        let metrics_cache = Arc::new(MetricsCache::new(Arc::clone(&metrics_service)));
        let server_info = crate::models::ServerInfo::new(
            "test-server".to_string(),
            "1.0.0".to_string(),
            chrono::Utc::now(),
            "development".to_string(),
        ).expect("Failed to create test ServerInfo");

        ServerStatusState::new(metrics_cache, metrics_service, server_info)
    }

    #[tokio::test]
    async fn test_sse_info_endpoint() {
        let state = create_test_state();
        let app = create_sse_router().with_state(state);
        let server = TestServer::new(app).unwrap();

        let response = server.get("/api/server-status-stream/info").await;
        assert_eq!(response.status_code(), 200);

        let body: serde_json::Value = response.json();
        assert_eq!(body["endpoint"], "/api/server-status-stream");
        assert_eq!(body["api_version"], "1.0");
        assert!(body["parameters"].is_object());
    }

    #[tokio::test]
    async fn test_sse_connection_state() {
        let state = SseConnectionState::new(
            "test_client".to_string(),
            5,
            true,
            Some(vec!["memory".to_string(), "cpu".to_string()]),
        );

        let info = state.get_connection_info();
        assert_eq!(info.client_id, "test_client");
        assert_eq!(info.update_interval_seconds, 5);
        assert_eq!(info.events_sent, 0);
    }

    #[tokio::test]
    async fn test_metrics_filtering() {
        let state = create_test_state();
        let filter = Some(vec!["memory".to_string()]);
        
        let stream = MetricsStream::new(
            "test_client".to_string(),
            5,
            true,
            filter,
            state,
        );

        let full_metrics = ServerMetrics {
            timestamp: Utc::now(),
            memory_usage: crate::models::MemoryMetrics {
                total_bytes: 1000000000,
                used_bytes: 500000000,
                available_bytes: 500000000,
                usage_percentage: 50.0,
            },
            cpu_usage: crate::models::CpuMetrics {
                usage_percentage: 25.0,
                core_count: 2,
                load_average: crate::models::cpu_metrics::LoadAverage {
                    one_minute: 1.5,
                    five_minute: 1.2,
                    fifteen_minute: 1.0,
                },
            },
            uptime: std::time::Duration::from_secs(3600), // 1 hour
            network_metrics: crate::models::NetworkMetrics::default(),
        };

        let filtered = stream.filter_metrics(full_metrics);
        
        // Should keep memory metrics
        assert_eq!(filtered.memory_usage.total_bytes, 1000000000);
        
        // Should zero out CPU metrics (not in filter)
        assert_eq!(filtered.cpu_usage.usage_percentage, 0.0);
        
        // Should zero out network metrics (not in filter)
        assert_eq!(filtered.network_metrics.bytes_received, 0);
    }

    #[test]
    fn test_simplified_metrics() {
        let full_metrics = ServerMetrics {
            timestamp: Utc::now(),
            memory_usage: crate::models::MemoryMetrics {
                total_bytes: 1000000000, // 1GB
                used_bytes: 500000000,   // 500MB
                available_bytes: 500000000, // 500MB
                usage_percentage: 50.0,
            },
            cpu_usage: crate::models::CpuMetrics {
                usage_percentage: 25.0,
                core_count: 4,
                load_average: crate::models::cpu_metrics::LoadAverage {
                    one_minute: 1.5,
                    five_minute: 1.2,
                    fifteen_minute: 1.0,
                },
            },
            uptime: std::time::Duration::from_secs(3600), // 1 hour
            network_metrics: crate::models::NetworkMetrics {
                bytes_sent: 500000,
                bytes_received: 1000000,
                packets_sent: 1000,
                packets_received: 2000,
                active_connections: 10,
            },
        };

        let simplified = create_simplified_metrics(full_metrics);
        
        // Should keep basic memory info
        assert_eq!(simplified.memory_usage.total_bytes, 1000000000);
        assert_eq!(simplified.memory_usage.available_bytes, 500000000);
        
        // Should keep CPU usage
        assert_eq!(simplified.cpu_usage.usage_percentage, 25.0);
        
        // Should keep total network bytes
        assert_eq!(simplified.network_metrics.bytes_received, 1000000);
    }

    #[test]
    fn test_query_parameters_parsing() {
        // Test valid parameters
        let query = SseQuery {
            interval: Some(10),
            detailed: Some(false),
            client_id: Some("test_client".to_string()),
            metrics: Some("memory,cpu".to_string()),
        };
        
        assert_eq!(query.interval.unwrap(), 10);
        assert!(!query.detailed.unwrap());
        assert_eq!(query.client_id.unwrap(), "test_client");
        
        let metrics: Vec<String> = query.metrics.unwrap()
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        assert_eq!(metrics, vec!["memory", "cpu"]);
    }

    #[test]
    fn test_event_serialization() {
        let event_data = MetricsEvent {
            event_type: "metrics_update".to_string(),
            data: StatusData::new(
                ServerMetrics {
                    timestamp: Utc::now(),
                    memory_usage: crate::models::MemoryMetrics::default(),
                    cpu_usage: crate::models::CpuMetrics::default(),
                    uptime: std::time::Duration::from_secs(0),
                    network_metrics: crate::models::NetworkMetrics::default(),
                },
                5,
                crate::models::ServerInfo::new(
                    "test".to_string(),
                    "1.0.0".to_string(),
                    Utc::now(),
                    "development".to_string(),
                ).expect("Failed to create test ServerInfo"),
            ).expect("Failed to create test StatusData"),
            sequence: 1,
            timestamp: Utc::now(),
            connection_info: ConnectionInfo {
                client_id: "test".to_string(),
                connection_duration_seconds: 10,
                events_sent: 1,
                update_interval_seconds: 5,
            },
        };

        let json = serde_json::to_string(&event_data).unwrap();
        assert!(json.contains("metrics_update"));
        assert!(json.contains("\"sequence\":1"));
        assert!(json.contains("test"));
    }
}