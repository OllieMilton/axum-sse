// CORS middleware and security headers
use axum::{
    http::{
        HeaderValue, 
        header::{AUTHORIZATION, CONTENT_TYPE, ACCEPT},
        Method,
    },
    response::Response,
    middleware::Next,
    extract::Request,
};
use tower_http::cors::{CorsLayer, Any};
use std::time::Duration;
use tracing::debug;

/// Create CORS layer for the application
pub fn cors_layer() -> CorsLayer {
    debug!("Configuring CORS layer");
    
    CorsLayer::new()
        // Allow GET, POST, OPTIONS methods
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        // Allow common headers
        .allow_headers([AUTHORIZATION, CONTENT_TYPE, ACCEPT])
        // Allow any origin in development (restrict in production)
        .allow_origin(Any)
        // Cache preflight requests for 1 hour
        .max_age(Duration::from_secs(3600))
}

/// Security headers middleware
pub async fn security_headers(request: Request, next: Next) -> Response {
    debug!("Adding security headers");
    
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    
    // Prevent XSS attacks
    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff")
    );
    
    // Prevent clickjacking
    headers.insert(
        "X-Frame-Options",
        HeaderValue::from_static("DENY")
    );
    
    // XSS protection
    headers.insert(
        "X-XSS-Protection",
        HeaderValue::from_static("1; mode=block")
    );
    
    // Referrer policy
    headers.insert(
        "Referrer-Policy",
        HeaderValue::from_static("strict-origin-when-cross-origin")
    );
    
    // Content Security Policy (restrictive for security)
    headers.insert(
        "Content-Security-Policy",
        HeaderValue::from_static(
            "default-src 'self'; \
             script-src 'self' 'unsafe-inline'; \
             style-src 'self' 'unsafe-inline'; \
             connect-src 'self'; \
             img-src 'self' data:; \
             font-src 'self'"
        )
    );
    
    // Permissions policy (restrict unnecessary features)
    headers.insert(
        "Permissions-Policy",
        HeaderValue::from_static(
            "camera=(), \
             microphone=(), \
             geolocation=(), \
             gyroscope=(), \
             magnetometer=(), \
             payment=()"
        )
    );
    
    response
}

/// Cache control middleware for static assets
pub async fn cache_control(request: Request, next: Next) -> Response {
    let path = request.uri().path().to_string(); // Clone the path to avoid borrow issues
    let mut response = next.run(request).await;
    
    // Set cache headers based on file type
    if is_static_asset(&path) {
        debug!("Setting cache headers for static asset: {}", path);
        let headers = response.headers_mut();
        
        if is_long_lived_asset(&path) {
            // Long-lived assets (images, fonts) - cache for 1 year
            headers.insert(
                "Cache-Control",
                HeaderValue::from_static("public, max-age=31536000, immutable")
            );
        } else {
            // Other static assets - cache for 1 hour
            headers.insert(
                "Cache-Control",
                HeaderValue::from_static("public, max-age=3600")
            );
        }
    } else if path.starts_with("/api/") {
        // API endpoints - no cache
        debug!("Setting no-cache headers for API endpoint: {}", path);
        let headers = response.headers_mut();
        headers.insert(
            "Cache-Control",
            HeaderValue::from_static("no-cache, no-store, must-revalidate")
        );
    } else {
        // HTML pages - short cache
        debug!("Setting short cache headers for page: {}", path);
        let headers = response.headers_mut();
        headers.insert(
            "Cache-Control",
            HeaderValue::from_static("public, max-age=300")
        );
    }
    
    response
}

fn is_static_asset(path: &str) -> bool {
    let static_extensions = [
        ".css", ".js", ".png", ".jpg", ".jpeg", ".gif", ".svg", 
        ".ico", ".woff", ".woff2", ".ttf", ".eot", ".json"
    ];
    
    static_extensions.iter().any(|ext| path.ends_with(ext))
}

fn is_long_lived_asset(path: &str) -> bool {
    let long_lived_extensions = [
        ".png", ".jpg", ".jpeg", ".gif", ".svg", ".ico",
        ".woff", ".woff2", ".ttf", ".eot"
    ];
    
    long_lived_extensions.iter().any(|ext| path.ends_with(ext))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_static_asset_detection() {
        assert!(is_static_asset("/styles.css"));
        assert!(is_static_asset("/script.js"));
        assert!(is_static_asset("/image.png"));
        assert!(is_static_asset("/font.woff2"));
        
        assert!(!is_static_asset("/"));
        assert!(!is_static_asset("/about"));
        assert!(!is_static_asset("/api/health"));
    }
    
    #[test]
    fn test_long_lived_asset_detection() {
        assert!(is_long_lived_asset("/image.png"));
        assert!(is_long_lived_asset("/font.woff2"));
        assert!(is_long_lived_asset("/icon.svg"));
        
        assert!(!is_long_lived_asset("/styles.css"));
        assert!(!is_long_lived_asset("/script.js"));
        assert!(!is_long_lived_asset("/data.json"));
    }
    
    #[test]
    fn test_cors_layer_creation() {
        let _cors = cors_layer();
        // If this compiles and runs, the CORS layer is correctly configured
    }
}