use std::{sync::Arc, net::SocketAddr};
use tokio::signal;
use tracing::{info, warn};
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
    fmt,
    EnvFilter,
};

use axum_sse::{build_router, SseService, StaticService, MetricsService, MetricsCache, ServerInfo, OsInfo};

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
    
    // Collect OS information
    let os_info = metrics_service.collect_os_info().await.unwrap_or_else(|e| {
        warn!("Failed to collect OS info: {}, using fallback", e);
        OsInfo::fallback()
    });
    
    // Create server info
    let server_info = ServerInfo::new(
        hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string()),
        env!("CARGO_PKG_VERSION").to_string(),
        chrono::Utc::now(),
        std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
        os_info.clone(),
    ).unwrap_or_else(|e| {
        warn!("Failed to create server info: {}, using defaults", e);
        ServerInfo::new(
            "unknown".to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
            chrono::Utc::now(),
            "development".to_string(),
            os_info,
        ).unwrap()
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
    use axum::http::{Request, StatusCode};
    use axum::body::Body;
    use tower::ServiceExt;
    
    fn create_test_services() -> (Arc<SseService>, Arc<StaticService>, Arc<MetricsCache>, Arc<MetricsService>, ServerInfo) {
        let sse_service = Arc::new(SseService::new());
        let static_service = Arc::new(StaticService::new());
        let metrics_service = Arc::new(MetricsService::new());
        let metrics_cache = Arc::new(MetricsCache::new(Arc::clone(&metrics_service)));
        let server_info = ServerInfo::new(
            "test-server".to_string(),
            "1.0.0".to_string(),
            chrono::Utc::now(),
            "development".to_string(),
            OsInfo::fallback(),
        ).unwrap();
        
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