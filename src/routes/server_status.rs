// Server status API endpoint handlers
// Provides REST endpoints for server metrics and status information

use crate::models::{
    StatusData, ServerMetrics, MetricsCollectionError, MetricsResponse,
    ServerInfo, MetricsValidationError
};
use crate::services::{MetricsCache, MetricsService};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::get,
    Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, warn, error, instrument};

/// Query parameters for server status endpoint
#[derive(Debug, Deserialize)]
pub struct StatusQuery {
    /// Include detailed metrics (default: true)
    pub detailed: Option<bool>,
    /// Cache key for metrics (optional)
    pub cache_key: Option<String>,
    /// Force fresh collection bypassing cache
    pub force_refresh: Option<bool>,
}

/// Response format for server status endpoint
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerStatusResponse {
    /// Status data with metrics and server info
    pub data: StatusData,
    /// Additional metadata about the response
    pub metadata: ResponseMetadata,
}

/// Metadata included with API responses
#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseMetadata {
    /// Timestamp when response was generated
    pub response_timestamp: DateTime<Utc>,
    /// Whether data came from cache
    pub cached: bool,
    /// Collection time in milliseconds
    pub collection_time_ms: Option<u64>,
    /// API version
    pub api_version: String,
    /// Any warnings or partial data indicators
    pub warnings: Vec<String>,
}

impl Default for ResponseMetadata {
    fn default() -> Self {
        Self {
            response_timestamp: Utc::now(),
            cached: false,
            collection_time_ms: None,
            api_version: "1.0".to_string(),
            warnings: vec![],
        }
    }
}

/// Error response format
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub error_type: String,
    pub timestamp: DateTime<Utc>,
    pub api_version: String,
    pub details: Option<serde_json::Value>,
}

impl ErrorResponse {
    fn new(error: &str, error_type: &str) -> Self {
        Self {
            error: error.to_string(),
            error_type: error_type.to_string(),
            timestamp: Utc::now(),
            api_version: "1.0".to_string(),
            details: None,
        }
    }

    fn with_details(error: &str, error_type: &str, details: serde_json::Value) -> Self {
        Self {
            error: error.to_string(),
            error_type: error_type.to_string(),
            timestamp: Utc::now(),
            api_version: "1.0".to_string(),
            details: Some(details),
        }
    }
}

/// Application state shared with route handlers
#[derive(Clone)]
pub struct ServerStatusState {
    pub metrics_cache: Arc<MetricsCache>,
    pub metrics_service: Arc<MetricsService>,
    pub server_info: ServerInfo,
}

impl ServerStatusState {
    pub fn new(
        metrics_cache: Arc<MetricsCache>,
        metrics_service: Arc<MetricsService>,
        server_info: ServerInfo,
    ) -> Self {
        Self {
            metrics_cache,
            metrics_service,
            server_info,
        }
    }
}

    /// GET /api/server-status - Get current server status and metrics
#[instrument(skip(state))]
pub async fn get_server_status(
    Query(params): Query<StatusQuery>,
    State(state): State<ServerStatusState>,
) -> Result<Json<ServerStatusResponse>, ServerStatusError> {
    debug!("GET /api/server-status - params: {:?}", params);

    let start_time = std::time::Instant::now();
    let detailed = params.detailed.unwrap_or(true);
    let force_refresh = params.force_refresh.unwrap_or(false);

    // Collect metrics
    let metrics_result = if force_refresh {
        debug!("Force refresh requested, bypassing cache");
        state.metrics_service.collect_fresh_metrics().await
    } else {
        state.metrics_cache.get_metrics(params.cache_key).await
    };

    let collection_time = start_time.elapsed().as_millis() as u64;

    // Process metrics result
    let (server_metrics, mut warnings) = match metrics_result {
        MetricsResponse::Ok(metrics) => (metrics, vec![]),
        MetricsResponse::PartialData { data, errors } => {
            let warnings: Vec<String> = errors
                .iter()
                .map(|e| format!("Partial data warning: {}", e))
                .collect();
            
            warn!("Partial metrics data returned with {} warnings", warnings.len());
            (data, warnings)
        }
        MetricsResponse::Error(error) => {
            error!("Failed to collect server metrics: {}", error);
            return Err(ServerStatusError::MetricsCollection(error));
        }
    };

    // Create simplified metrics if detailed=false
    let final_metrics = if detailed {
        server_metrics
    } else {
        create_simplified_metrics(server_metrics)
    };

    // Check for stale timestamps and add warnings
    if let Some(age_seconds) = final_metrics.is_timestamp_stale() {
        warnings.push(format!("Timestamp is stale: {} seconds old", age_seconds));
    }

    // Validate metrics
    if let Err(validation_error) = final_metrics.validate() {
        warn!("Metrics validation failed: {}", validation_error);
        warnings.push(format!("Validation warning: {}", validation_error));
    }

    // Create status data
    let status_data = match StatusData::new(
        final_metrics,
        state.metrics_service.get_config().collection_interval_seconds,
        state.server_info.clone(),
    ) {
        Ok(data) => data,
        Err(validation_error) => {
            warn!("StatusData validation failed: {}", validation_error);
            return Err(ServerStatusError::Internal(format!("StatusData validation failed: {}", validation_error)));
        }
    };

    // Check if data came from cache
    let cached = !force_refresh && collection_time < 50; // Heuristic: < 50ms likely cached

    // Create response metadata
    let metadata = ResponseMetadata {
        response_timestamp: Utc::now(),
        cached,
        collection_time_ms: Some(collection_time),
        api_version: "1.0".to_string(),
        warnings,
    };

    let response = ServerStatusResponse {
        data: status_data,
        metadata,
    };

    debug!(
        "Successfully returned server status (cached: {}, collection_time: {}ms)",
        cached, collection_time
    );

    Ok(Json(response))
}

/// Create simplified metrics for non-detailed requests
fn create_simplified_metrics(full_metrics: ServerMetrics) -> ServerMetrics {
    ServerMetrics {
        timestamp: full_metrics.timestamp,
        memory_usage: full_metrics.memory_usage, // Keep full memory metrics
        cpu_usage: crate::models::CpuMetrics {
            usage_percentage: full_metrics.cpu_usage.usage_percentage,
            core_count: full_metrics.cpu_usage.core_count,
            load_average: crate::models::cpu_metrics::LoadAverage {
                one_minute: full_metrics.cpu_usage.load_average.one_minute,
                five_minute: 0.0, // Remove 5min load for simplified view
                fifteen_minute: 0.0, // Remove 15min load for simplified view
            },
        },
        uptime: full_metrics.uptime,
        network_metrics: crate::models::NetworkMetrics {
            bytes_sent: full_metrics.network_metrics.bytes_sent,
            bytes_received: full_metrics.network_metrics.bytes_received,
            packets_sent: 0, // Remove packet details for simplified view
            packets_received: 0, // Remove packet details for simplified view
            active_connections: full_metrics.network_metrics.active_connections,
        },
    }
}

    /// GET /api/server-status/health - Health check endpoint
#[instrument(skip(state))]
pub async fn get_server_health(
    State(state): State<ServerStatusState>,
) -> Result<Json<serde_json::Value>, ServerStatusError> {
    debug!("GET /api/server-status/health");

    // Quick health check - try to get cached metrics
    let metrics_result = state.metrics_cache.get_metrics(Some("health_check".to_string())).await;
    
    let health_status = match metrics_result {
        MetricsResponse::Ok(metrics) => {
            match StatusData::new(
                metrics,
                state.metrics_service.get_config().collection_interval_seconds,
                state.server_info.clone(),
            ) {
                Ok(status_data) => match status_data.get_health_status() {
                    crate::models::HealthStatus::Healthy => "healthy",
                    crate::models::HealthStatus::Warning => "warning", 
                    crate::models::HealthStatus::Critical => "critical",
                },
                Err(_) => "warning", // Validation failed, but we have metrics
            }
        }
        MetricsResponse::PartialData { .. } => "warning",
        MetricsResponse::Error(_) => "critical",
    };

    let cache_stats = state.metrics_cache.get_stats();
    let service_stats = state.metrics_service.get_stats().await;

    let health_response = serde_json::json!({
        "status": health_status,
        "timestamp": Utc::now(),
        "cache": {
            "hit_ratio": cache_stats.hit_ratio,
            "entries": cache_stats.current_entries,
        },
        "metrics_service": {
            "successful_collections": service_stats.successful_collections,
            "failed_collections": service_stats.failed_collections,
            "average_collection_time_ms": service_stats.average_collection_time_ms,
        },
        "api_version": "1.0"
    });

    debug!("Health check returned status: {}", health_status);
    Ok(Json(health_response))
}

/// Custom error type for server status endpoints
#[derive(Debug)]
pub enum ServerStatusError {
    MetricsCollection(MetricsCollectionError),
    Validation(MetricsValidationError),
    Internal(String),
}

impl std::fmt::Display for ServerStatusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MetricsCollection(e) => write!(f, "Metrics collection error: {}", e),
            Self::Validation(e) => write!(f, "Validation error: {}", e),
            Self::Internal(e) => write!(f, "Internal error: {}", e),
        }
    }
}

impl std::error::Error for ServerStatusError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::MetricsCollection(e) => Some(e),
            Self::Validation(e) => Some(e),
            Self::Internal(_) => None,
        }
    }
}

impl IntoResponse for ServerStatusError {
    fn into_response(self) -> Response {
        let (status_code, error_type, error_message, details) = match &self {
            Self::MetricsCollection(e) => {
                let details = serde_json::json!({
                    "recoverable": e.is_recoverable(),
                    "severity": format!("{:?}", e.severity()),
                    "retry_delay_ms": e.retry_delay_ms(),
                });
                
                match e.severity() {
                    crate::models::metrics_errors::ErrorSeverity::Warning => {
                        (StatusCode::OK, "metrics_warning", self.to_string(), Some(details))
                    }
                    crate::models::metrics_errors::ErrorSeverity::Error => {
                        (StatusCode::SERVICE_UNAVAILABLE, "metrics_error", self.to_string(), Some(details))
                    }
                    crate::models::metrics_errors::ErrorSeverity::Critical => {
                        (StatusCode::INTERNAL_SERVER_ERROR, "metrics_critical", self.to_string(), Some(details))
                    }
                }
            }
            Self::Validation(_e) => {
                let details = serde_json::json!({
                    "validation_type": "metrics_validation",
                    "field": "server_metrics",
                });
                (StatusCode::UNPROCESSABLE_ENTITY, "validation_error", self.to_string(), Some(details))
            }
            Self::Internal(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", self.to_string(), None)
            }
        };

        let error_response = match details {
            Some(details) => ErrorResponse::with_details(&error_message, error_type, details),
            None => ErrorResponse::new(&error_message, error_type),
        };

        (status_code, Json(error_response)).into_response()
    }
}

impl From<MetricsCollectionError> for ServerStatusError {
    fn from(error: MetricsCollectionError) -> Self {
        Self::MetricsCollection(error)
    }
}

impl From<MetricsValidationError> for ServerStatusError {
    fn from(error: MetricsValidationError) -> Self {
        Self::Validation(error)
    }
}

/// Create the server status router
pub fn create_router() -> Router<ServerStatusState> {
    Router::new()
        .route("/server-status", get(get_server_status))
        .route("/server-status/health", get(get_server_health))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::{MetricsService, MetricsCache};
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use std::sync::Arc;

    fn create_test_state() -> ServerStatusState {
        let metrics_service = Arc::new(MetricsService::new());
        let metrics_cache = Arc::new(MetricsCache::new(Arc::clone(&metrics_service)));
        let server_info = ServerInfo::new(
            "test-server".to_string(),
            "1.0.0".to_string(),
            Utc::now(),
            "development".to_string(),
        ).expect("Failed to create test ServerInfo");

        ServerStatusState::new(metrics_cache, metrics_service, server_info)
    }

    #[tokio::test]
    async fn test_server_status_endpoint() {
        let state = create_test_state();
        state.metrics_service.initialize().await.unwrap();
        
        let app = create_router().with_state(state);
        let server = TestServer::new(app).unwrap();

        let response = server.get("/api/server-status").await;
        
        // Should succeed or return partial data
        assert!(
            response.status_code() == StatusCode::OK ||
            response.status_code() == StatusCode::SERVICE_UNAVAILABLE
        );

        if response.status_code() == StatusCode::OK {
            let body: ServerStatusResponse = response.json();
            assert!(!body.data.server_info.hostname.is_empty());
            assert!(body.metadata.api_version == "1.0");
        }
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let state = create_test_state();
        state.metrics_service.initialize().await.unwrap();
        
        let app = create_router().with_state(state);
        let server = TestServer::new(app).unwrap();

        let response = server.get("/api/server-status/health").await;
        assert_eq!(response.status_code(), StatusCode::OK);

        let body: serde_json::Value = response.json();
        assert!(body["status"].is_string());
        assert!(body["timestamp"].is_string());
        assert!(body["api_version"] == "1.0");
    }

    #[tokio::test]
    async fn test_detailed_query_parameter() {
        let state = create_test_state();
        state.metrics_service.initialize().await.unwrap();
        
        let app = create_router().with_state(state);
        let server = TestServer::new(app).unwrap();

        // Test detailed=false
        let response = server.get("/api/server-status?detailed=false").await;
        
        if response.status_code() == StatusCode::OK {
            let body: ServerStatusResponse = response.json();
            // Simplified metrics should have basic CPU data
            assert!(body.data.server_metrics.cpu_usage.usage_percentage >= 0.0);
        }
    }

    #[tokio::test]
    async fn test_force_refresh_parameter() {
        let state = create_test_state();
        state.metrics_service.initialize().await.unwrap();
        
        let app = create_router().with_state(state);
        let server = TestServer::new(app).unwrap();

        let response = server.get("/api/server-status?force_refresh=true").await;
        
        if response.status_code() == StatusCode::OK {
            let body: ServerStatusResponse = response.json();
            // Force refresh should indicate not cached
            assert!(!body.metadata.cached);
            assert!(body.metadata.collection_time_ms.unwrap() > 0);
        }
    }

    #[test]
    fn test_error_response_creation() {
        let error = ErrorResponse::new("Test error", "test_error");
        assert_eq!(error.error, "Test error");
        assert_eq!(error.error_type, "test_error");
        assert_eq!(error.api_version, "1.0");
    }

    #[test]
    fn test_simplified_metrics_creation() {
        let full_metrics = ServerMetrics {
            timestamp: Utc::now(),
            memory_usage: crate::models::MemoryMetrics::default(),
            cpu_usage: crate::models::CpuMetrics {
                usage_percentage: 50.0,
                core_count: 4,
                load_average: crate::models::cpu_metrics::LoadAverage {
                    one_minute: 1.5,
                    five_minute: 1.2,
                    fifteen_minute: 1.0,
                },
            },
            uptime: std::time::Duration::from_secs(86400), // 24 hours
            network_metrics: crate::models::NetworkMetrics {
                bytes_sent: 500000,
                bytes_received: 1000000,
                packets_sent: 1000,
                packets_received: 2000,
                active_connections: 10,
            },
        };

        let simplified = create_simplified_metrics(full_metrics);
        
        // Should keep overall CPU usage but remove per-core
        assert_eq!(simplified.cpu_usage.usage_percentage, 50.0);
        
        // Should keep total network stats but remove interface details
        assert_eq!(simplified.network_metrics.bytes_received, 1000000);
    }
}