// API endpoint implementations
use axum::{
    extract::Extension,
    response::{
        sse::Event,
        Sse,
    },
    http::StatusCode,
    Json,
};
use std::{sync::Arc, convert::Infallible};
use futures::stream::Stream;
use crate::services::{SseService, StaticService};
use serde_json::{json, Value};
use tracing::{info, error};

/// SSE endpoint for time stream (/api/time/stream)
pub async fn time_stream(
    Extension(sse_service): Extension<Arc<SseService>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    info!("New SSE time stream connection requested");
    
    // Create a new SSE stream for this client
    sse_service.create_time_stream()
}

/// Health check endpoint (/health)
pub async fn health_check(
    Extension(sse_service): Extension<Arc<SseService>>,
    Extension(static_service): Extension<Arc<StaticService>>,
) -> Result<Json<Value>, StatusCode> {
    info!("Health check requested");
    
    let sse_healthy = sse_service.is_healthy();
    let static_healthy = static_service.is_healthy();
    let overall_healthy = sse_healthy && static_healthy;
    
    let status_code = if overall_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    
    let response = json!({
        "status": if overall_healthy { "ok" } else { "unhealthy" },
        "service": "axum-sse",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "components": {
            "sse_service": {
                "healthy": sse_healthy,
                "active_connections": sse_service.receiver_count()
            },
            "static_service": {
                "healthy": static_healthy,
                "asset_count": static_service.asset_count()
            }
        }
    });
    
    if overall_healthy {
        info!("Health check passed - all services healthy");
        Ok(Json(response))
    } else {
        error!("Health check failed - some services unhealthy");
        Err(status_code)
    }
}

/// Service status endpoint (/api/status) - detailed service information
pub async fn service_status(
    Extension(sse_service): Extension<Arc<SseService>>,
    Extension(static_service): Extension<Arc<StaticService>>,
) -> Json<Value> {
    info!("Service status requested");
    
    let response = json!({
        "service": "axum-sse",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "uptime_seconds": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        "sse": {
            "healthy": sse_service.is_healthy(),
            "active_connections": sse_service.receiver_count(),
            "broadcast_interval_seconds": 10
        },
        "static_assets": {
            "healthy": static_service.is_healthy(),
            "embedded_asset_count": static_service.asset_count(),
            "available_assets": static_service.list_assets().len()
        }
    });
    
    Json(response)
}

/// Endpoint to trigger a manual time broadcast (/api/time/broadcast) - for testing
pub async fn manual_time_broadcast(
    Extension(sse_service): Extension<Arc<SseService>>,
) -> Result<Json<Value>, StatusCode> {
    info!("Manual time broadcast requested");
    
    // For now, we'll just return success since our SSE service 
    // broadcasts automatically every 10 seconds
    let response = json!({
        "message": "Time is broadcast automatically every 10 seconds",
        "active_connections": sse_service.receiver_count(),
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::{SseService, StaticService};
    use axum::{body::Body, http::Request, Router, routing::get};
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn test_health_check_endpoint() {
        let sse_service = Arc::new(SseService::new());
        let static_service = Arc::new(StaticService::new());
        
        let app = Router::new()
            .route("/health", get(health_check))
            .layer(Extension(sse_service))
            .layer(Extension(static_service));

        let response = app
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_service_status_endpoint() {
        let sse_service = Arc::new(SseService::new());
        let static_service = Arc::new(StaticService::new());
        
        let app = Router::new()
            .route("/api/status", get(service_status))
            .layer(Extension(sse_service))
            .layer(Extension(static_service));

        let response = app
            .oneshot(Request::builder().uri("/api/status").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_manual_broadcast_endpoint() {
        let sse_service = Arc::new(SseService::new());
        
        let app = Router::new()
            .route("/api/time/broadcast", get(manual_time_broadcast))
            .layer(Extension(sse_service));

        let response = app
            .oneshot(Request::builder().uri("/api/time/broadcast").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_sse_time_stream_endpoint() {
        let sse_service = Arc::new(SseService::new());
        
        let app = Router::new()
            .route("/api/time/stream", get(time_stream))
            .layer(Extension(sse_service));

        let response = app
            .oneshot(Request::builder().uri("/api/time/stream").body(Body::empty()).unwrap())
            .await
            .unwrap();

        // SSE endpoints should return 200 OK
        assert_eq!(response.status(), StatusCode::OK);
        
        // Check for SSE headers
        let headers = response.headers();
        assert_eq!(headers.get("content-type").unwrap(), "text/event-stream");
    }
}