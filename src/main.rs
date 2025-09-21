use axum::{
    Router,
    Extension,
    routing::{get, post},
};
use std::{sync::Arc, net::SocketAddr};
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
    fmt,
    EnvFilter,
};

mod models;
mod services;
mod routes;
mod middleware;

use services::{SseService, StaticService, MetricsService, MetricsCache};
use routes::{pages, api, server_status, server_status_stream};
use models::ServerInfo;
use middleware::{
    cors_layer, security_headers, cache_control,
    request_logging, error_handling, request_id_middleware
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    init_logging();
    
    info!("🚀 Starting axum-sse server...");
    
    // Initialize services
    let sse_service = Arc::new(SseService::new());
    let static_service = Arc::new(StaticService::new());
    
    // Initialize metrics services
    let metrics_service = Arc::new(MetricsService::new());
    let metrics_cache = Arc::new(MetricsCache::new(Arc::clone(&metrics_service)));
    
    // Initialize metrics service
    if let Err(e) = metrics_service.initialize().await {
        warn!("Failed to initialize metrics service: {}", e);
    } else {
        info!("📊 Metrics service initialized");
    }
    
    // Start metrics cache background refresh
    if let Err(e) = metrics_cache.start_background_refresh().await {
        warn!("Failed to start metrics cache background refresh: {}", e);
    } else {
        info!("🔄 Metrics cache background refresh started");
    }
    
    // Create server info
    let server_info = ServerInfo::new(
        hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string()),
        env!("CARGO_PKG_VERSION").to_string(),
        chrono::Utc::now(),
        std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
    ).unwrap_or_else(|e| {
        warn!("Failed to create server info: {}, using defaults", e);
        ServerInfo {
            hostname: "unknown".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            start_time: chrono::Utc::now(),
            environment: "development".to_string(),
        }
    });
    
    // Start the SSE time broadcaster
    SseService::start_time_broadcaster(&sse_service);
    info!("📡 SSE time broadcaster started");
    
    // Build the application router
    let app = build_router(
        sse_service, 
        static_service, 
        metrics_cache, 
        metrics_service, 
        server_info
    );
    
    // Configure server address
    let addr = get_server_address();
    info!("🌐 Server will listen on http://{}", addr);
    
    // Create listener
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("✅ Server listening on http://{}", addr);
    
    // Start server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    
    info!("👋 Server shutdown complete");
    Ok(())
}

fn build_router(
    sse_service: Arc<SseService>,
    static_service: Arc<StaticService>,
    metrics_cache: Arc<MetricsCache>,
    metrics_service: Arc<MetricsService>,
    server_info: ServerInfo,
) -> Router {
    info!("🔧 Building application router...");
    
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
        .layer(Extension(sse_service))
        .layer(Extension(static_service))
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

fn init_logging() {
    // Configure logging based on environment
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            // Default to info level, with debug for our crate
            "axum_sse=debug,tower_http=debug,info".into()
        });
    
    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_line_number(true)
                .compact()
        )
        .init();
    
    info!("📋 Logging initialized");
}

fn get_server_address() -> SocketAddr {
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);
    
    format!("{}:{}", host, port)
        .parse()
        .unwrap_or_else(|_| "127.0.0.1:3000".parse().unwrap())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            warn!("🛑 Received Ctrl+C, shutting down gracefully...");
        },
        _ = terminate => {
            warn!("🛑 Received SIGTERM, shutting down gracefully...");
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{Request, StatusCode};
    use axum::body::Body;
    use tower::ServiceExt;
    
    fn create_test_services() -> (Arc<SseService>, Arc<StaticService>, Arc<MetricsCache>, Arc<MetricsService>, ServerInfo) {
        let sse_service = Arc::new(SseService::new());
        let static_service = Arc::new(StaticService::new());
        let metrics_service = Arc::new(MetricsService::new());
        let metrics_cache = Arc::new(MetricsCache::new(Arc::clone(&metrics_service)));
        let server_info = ServerInfo {
            hostname: "test-server".to_string(),
            version: "1.0.0".to_string(),
            start_time: chrono::Utc::now(),
            environment: "test".to_string(),
        };
        
        (sse_service, static_service, metrics_cache, metrics_service, server_info)
    }
    
    #[tokio::test]
    async fn test_router_creation() {
        let (sse_service, static_service, metrics_cache, metrics_service, server_info) = create_test_services();
        
        let app = build_router(sse_service, static_service, metrics_cache, metrics_service, server_info);
        
        // Test that the router can handle requests
        let request = Request::builder()
            .uri("/api/health")
            .body(Body::empty())
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
    
    #[tokio::test]
    async fn test_api_routes() {
        let (sse_service, static_service, metrics_cache, metrics_service, server_info) = create_test_services();
        
        let app = build_router(sse_service, static_service, metrics_cache, metrics_service, server_info);
        
        // Test health endpoint
        let request = Request::builder()
            .uri("/api/health")
            .body(Body::empty())
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
    
    #[tokio::test]
    async fn test_page_routes() {
        let (sse_service, static_service, metrics_cache, metrics_service, server_info) = create_test_services();
        
        let app = build_router(sse_service, static_service, metrics_cache, metrics_service, server_info);
        
        // Test index page
        let request = Request::builder()
            .uri("/")
            .body(Body::empty())
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
    
    #[test]
    fn test_server_address_parsing() {
        // Test default address
        std::env::remove_var("HOST");
        std::env::remove_var("PORT");
        let addr = get_server_address();
        assert_eq!(addr.to_string(), "127.0.0.1:3000");
        
        // Test custom address
        std::env::set_var("HOST", "0.0.0.0");
        std::env::set_var("PORT", "8080");
        let addr = get_server_address();
        assert_eq!(addr.to_string(), "0.0.0.0:8080");
        
        // Clean up
        std::env::remove_var("HOST");
        std::env::remove_var("PORT");
    }
}