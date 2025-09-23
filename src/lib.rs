pub mod models;
pub mod routes;
pub mod services;
pub mod middleware;

use axum::Router;
use std::sync::Arc;
use chrono::Utc;

pub use services::{SseService, StaticService, MetricsService, MetricsCache};
pub use models::{ServerInfo, OsInfo};
pub use routes::server_status;

/// Create application router for testing
pub async fn create_app() -> Router {
    // Initialize services for testing
    let sse_service = Arc::new(SseService::new());
    let static_service = Arc::new(StaticService::new());
    let metrics_service = Arc::new(MetricsService::new());
    let metrics_cache = Arc::new(MetricsCache::new(Arc::clone(&metrics_service)));
    
    // Initialize metrics service
    let _ = metrics_service.initialize().await;
    
    // Collect OS information
    let os_info = metrics_service.collect_os_info().await.unwrap_or_else(|_| OsInfo::fallback());
    
    // Create test server info
    let server_info = ServerInfo::new(
        "test-server".to_string(),
        "0.1.0".to_string(),
        Utc::now(),
        "development".to_string(),
        os_info,
    ).unwrap();
    
    build_router(
        sse_service,
        static_service,  
        metrics_cache,
        metrics_service,
        server_info,
    )
}

/// Build the application router - exposed for testing
pub fn build_router(
    sse_service: Arc<SseService>,
    static_service: Arc<StaticService>,
    metrics_cache: Arc<MetricsCache>,
    metrics_service: Arc<MetricsService>,
    server_info: ServerInfo,
) -> Router {
    use axum::routing::{get, post};
    use routes::{pages, api, server_status_stream};
    use tower::ServiceBuilder;
    use tower_http::trace::TraceLayer;
    use middleware::{
        cors_layer, security_headers, cache_control,
        request_logging, error_handling, request_id_middleware
    };
    
    // Create server status state
    let server_status_state = server_status::ServerStatusState::new(
        Arc::clone(&metrics_cache),
        Arc::clone(&metrics_service),
        server_info,
    );
    
    // API routes
    let api_routes = Router::new()
        .route("/time-stream", get(api::time_stream))
        .route("/health", get(api::health_check))
        .route("/status", get(api::service_status))
        .route("/broadcast", post(api::manual_time_broadcast))
        // Merge server status routes
        .merge(server_status::create_router().with_state(server_status_state.clone()))
        // Merge SSE routes
        .merge(server_status_stream::create_sse_router().with_state(server_status_state));
    
    // Page routes for SPA  
    let page_routes = Router::new()
        .route("/", get(pages::serve_main_page))
        // Static assets (CSS, JS, images) - must be before the SPA fallback
        .route("/assets/*path", get(pages::serve_static_asset))
        .route("/_app/*path", get(pages::serve_app_asset))
        .route("/favicon.ico", get(pages::serve_fallback_asset))
        // SPA fallback - catches all other routes and serves index.html for client-side routing
        .fallback(get(pages::serve_spa_fallback));
    
    // Build main application
    Router::new()
        // Mount API routes under /api prefix
        .nest("/api", api_routes)
        // Mount page routes at root
        .merge(page_routes)
        // Add service extensions
        .layer(axum::Extension(sse_service))
        .layer(axum::Extension(static_service))
        // Add middleware stack (order matters - first added runs last)
        .layer(
            ServiceBuilder::new()
                // Request ID and logging first
                .layer(axum::middleware::from_fn(request_id_middleware))
                .layer(axum::middleware::from_fn(request_logging))
                // Error handling
                .layer(axum::middleware::from_fn(error_handling))
                // Security layers
                .layer(cors_layer())
                .layer(axum::middleware::from_fn(security_headers))
                .layer(axum::middleware::from_fn(cache_control))
                // Tracing for detailed request/response logging
                .layer(TraceLayer::new_for_http())
        )
}