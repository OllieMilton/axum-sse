// Page route handlers using the static service
use axum::{
    response::Html,
    http::StatusCode,
    Extension,
};
use std::sync::Arc;
use crate::services::StaticService;
use tracing::{info, error, debug};

/// Handler for the main page (/)
pub async fn serve_main_page(
    Extension(static_service): Extension<Arc<StaticService>>,
) -> Result<Html<String>, StatusCode> {
    info!("Serving main page");
    
    match static_service.serve_index().await {
        Ok(html) => {
            info!("Successfully served main page");
            Ok(html)
        }
        Err(status) => {
            error!("Failed to serve main page: {}", status);
            Err(status)
        }
    }
}

/// SPA fallback handler - serves index.html for all unmatched routes
/// This enables client-side routing for the SPA
pub async fn serve_spa_fallback(
    uri: axum::http::Uri,
    Extension(static_service): Extension<Arc<StaticService>>,
) -> Result<Html<String>, StatusCode> {
    let path = uri.path();
    info!("SPA fallback for route: {}", path);
    
    // For SPA routing, always serve index.html for unmatched routes
    // Let the client-side router handle the actual routing
    match static_service.serve_index().await {
        Ok(html) => {
            info!("Successfully served SPA fallback for: {}", path);
            Ok(html)
        }
        Err(status) => {
            error!("Failed to serve SPA fallback for {}: {}", path, status);
            Err(status)
        }
    }
}

/// Handler for static assets (CSS, JS, images, etc.)
pub async fn serve_static_asset(
    axum::extract::Path(path): axum::extract::Path<String>,
    Extension(static_service): Extension<Arc<StaticService>>,
) -> Result<axum::response::Response, StatusCode> {
    info!("Serving static asset: {}", path);
    
    match static_service.serve_asset(&path).await {
        Ok(response) => {
            info!("Successfully served static asset: {}", path);
            Ok(response)
        }
        Err(status) => {
            error!("Failed to serve static asset {}: {}", path, status);
            Err(status)
        }
    }
}

/// Handler for _app assets (SvelteKit specific)
pub async fn serve_app_asset(
    axum::extract::Path(path): axum::extract::Path<String>,
    Extension(static_service): Extension<Arc<StaticService>>,
) -> Result<axum::response::Response, StatusCode> {
    // Reconstruct the full _app path
    let full_path = format!("_app/{}", path);
    info!("Serving static asset: {}", path);
    
    // Debug: List available assets to understand the structure
    let assets = static_service.list_assets();
    let app_assets: Vec<_> = assets.iter().filter(|a| a.starts_with("_app/")).take(5).collect();
    debug!("Available _app assets (first 5): {:?}", app_assets);
    debug!("Looking for asset: {}", full_path);
    
    match static_service.serve_asset(&full_path).await {
        Ok(response) => {
            info!("Successfully served static asset: {}", path);
            Ok(response)
        }
        Err(status) => {
            error!("Failed to serve static asset {}: {}", path, status);
            Err(status)
        }
    }
}

/// Fallback handler for static assets (extracts path from URI)
pub async fn serve_fallback_asset(
    uri: axum::http::Uri,
    Extension(static_service): Extension<Arc<StaticService>>,
) -> Result<axum::response::Response, StatusCode> {
    let path = uri.path();
    info!("Serving fallback static asset: {}", path);
    
    match static_service.serve_asset(path).await {
        Ok(response) => {
            info!("Successfully served fallback static asset: {}", path);
            Ok(response)
        }
        Err(status) => {
            error!("Failed to serve fallback static asset {}: {}", path, status);
            Err(status)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::StaticService;
    use axum::{body::Body, http::Request, Router, routing::get};
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn test_main_page_handler() {
        let static_service = Arc::new(StaticService::new());
        
        let app = Router::new()
            .route("/", get(serve_main_page))
            .layer(Extension(static_service));

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_static_asset_handler() {
        let static_service = Arc::new(StaticService::new());
        
        let app = Router::new()
            .route("/assets/*path", get(serve_static_asset))
            .layer(Extension(static_service));

        // Test with a non-existent asset (should return 404)
        let response = app
            .oneshot(Request::builder().uri("/assets/nonexistent.css").body(Body::empty()).unwrap())
            .await
            .unwrap();

        // This should return 404 since the asset doesn't exist
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}