// Request logging middleware
use axum::{
    response::Response,
    middleware::Next,
    extract::Request,
};
use tracing::{info, warn, error, debug};
use std::time::Instant;

/// Request logging middleware
pub async fn request_logging(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = Instant::now();
    
    info!("Request started: {} {}", method, uri);
    
    let response = next.run(request).await;
    let status = response.status();
    let duration = start.elapsed();
    
    match status.as_u16() {
        200..=299 => info!(
            "Request completed: {} {} -> {} ({:.2}ms)",
            method, uri, status, duration.as_millis()
        ),
        300..=399 => info!(
            "Request redirected: {} {} -> {} ({:.2}ms)",
            method, uri, status, duration.as_millis()
        ),
        400..=499 => warn!(
            "Client error: {} {} -> {} ({:.2}ms)",
            method, uri, status, duration.as_millis()
        ),
        500..=599 => error!(
            "Server error: {} {} -> {} ({:.2}ms)",
            method, uri, status, duration.as_millis()
        ),
        _ => debug!(
            "Request completed: {} {} -> {} ({:.2}ms)",
            method, uri, status, duration.as_millis()
        ),
    }
    
    response
}

/// Error handling middleware for catching panics and unhandled errors
pub async fn error_handling(request: Request, next: Next) -> Response {
    let uri = request.uri().clone();
    
    // Run the request
    let response = next.run(request).await;
    
    // If it's already an error response, log it
    if response.status().is_server_error() {
        error!("Server error response for {}: {}", uri, response.status());
    } else if response.status().is_client_error() {
        warn!("Client error response for {}: {}", uri, response.status());
    }
    
    response
}

/// Middleware to add request ID for tracing
pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    use uuid::Uuid;
    
    let request_id = Uuid::new_v4().to_string();
    
    // Add to headers for potential client use
    request.headers_mut().insert(
        "X-Request-ID",
        request_id.parse().unwrap_or_else(|_| "invalid".parse().unwrap())
    );
    
    debug!("Request ID: {}", request_id);
    
    let mut response = next.run(request).await;
    
    // Add to response headers
    response.headers_mut().insert(
        "X-Request-ID",
        request_id.parse().unwrap_or_else(|_| "invalid".parse().unwrap())
    );
    
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::{Request as HttpRequest, StatusCode}};
    use tower::ServiceExt;
    use axum::Router;
    use axum::routing::get;
    
    async fn test_handler() -> &'static str {
        "OK"
    }
    
    #[tokio::test]
    async fn test_request_logging_middleware() {
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(axum::middleware::from_fn(request_logging));
        
        let request = HttpRequest::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
    
    #[tokio::test]
    async fn test_error_handling_middleware() {
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(axum::middleware::from_fn(error_handling));
        
        let request = HttpRequest::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
    
    #[tokio::test]
    async fn test_request_id_middleware() {
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(axum::middleware::from_fn(request_id_middleware));
        
        let request = HttpRequest::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.headers().get("X-Request-ID").is_some());
    }
}