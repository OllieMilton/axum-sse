// Static asset serving service for embedded SvelteKit build
use axum::{
    response::{Html, Response},
    http::{StatusCode, HeaderMap, HeaderValue},
    body::Body,
};
use include_dir::{include_dir, Dir};
use std::path::Path;
use tracing::{info, warn, debug};

// Embed the frontend build directory at compile time
// Note: The build directory will be created during the frontend build process
static FRONTEND_DIR: Dir<'_> = include_dir!("frontend/build");

/// Static asset serving service for embedded frontend
#[derive(Clone)]
pub struct StaticService {
    /// Default index file name
    index_file: String,
}

impl StaticService {
    /// Create a new static service
    pub fn new() -> Self {
        Self {
            index_file: "index.html".to_string(),
        }
    }

    /// Serve the main page (index.html)
    pub async fn serve_index(&self) -> Result<Html<String>, StatusCode> {
        debug!("Serving index page");
        
        match self.get_file_content(&self.index_file) {
            Some(content) => {
                info!("Successfully served index.html ({} bytes)", content.len());
                Ok(Html(content))
            }
            None => {
                warn!("index.html not found in embedded assets");
                // Fallback to a basic HTML page
                let fallback_html = self.create_fallback_index();
                Ok(Html(fallback_html))
            }
        }
    }

    /// Serve a static asset by path
    pub async fn serve_asset(&self, path: &str) -> Result<Response<Body>, StatusCode> {
        debug!("Serving static asset: {}", path);
        
        // Clean the path to prevent directory traversal
        let clean_path = self.sanitize_path(path);
        
        match self.get_file_content(&clean_path) {
            Some(content) => {
                let mut headers = HeaderMap::new();
                
                // Set content type based on file extension
                if let Some(content_type) = self.get_content_type(&clean_path) {
                    headers.insert("content-type", HeaderValue::from_static(content_type));
                }
                
                // Set cache headers for static assets
                headers.insert("cache-control", HeaderValue::from_static("public, max-age=3600"));
                
                info!("Successfully served asset {} ({} bytes)", clean_path, content.len());
                
                let mut response = Response::new(Body::from(content));
                *response.headers_mut() = headers;
                Ok(response)
            }
            None => {
                warn!("Static asset not found: {}", clean_path);
                Err(StatusCode::NOT_FOUND)
            }
        }
    }

    /// Check if the static service is healthy (has embedded assets)
    pub fn is_healthy(&self) -> bool {
        !FRONTEND_DIR.entries().is_empty()
    }

    /// Get the number of embedded files
    pub fn asset_count(&self) -> usize {
        self.count_files_recursive(&FRONTEND_DIR)
    }

    /// List all available assets (for debugging)
    pub fn list_assets(&self) -> Vec<String> {
        self.list_files_recursive(&FRONTEND_DIR, "")
    }

    // Private helper methods

    fn get_file_content(&self, path: &str) -> Option<String> {
        FRONTEND_DIR.get_file(path)
            .and_then(|file| file.contents_utf8())
            .map(|content| content.to_string())
    }

    fn sanitize_path(&self, path: &str) -> String {
        // Remove leading slash and resolve any relative path components
        let clean = path.trim_start_matches('/');
        
        // Basic sanitization - in production you'd want more robust path validation
        if clean.contains("..") || clean.contains('\0') {
            return "404.html".to_string(); // Safe fallback
        }
        
        clean.to_string()
    }

    fn get_content_type(&self, path: &str) -> Option<&'static str> {
        let extension = Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())?;
        
        match extension {
            "html" => Some("text/html; charset=utf-8"),
            "css" => Some("text/css"),
            "js" => Some("application/javascript"),
            "json" => Some("application/json"),
            "png" => Some("image/png"),
            "jpg" | "jpeg" => Some("image/jpeg"),
            "gif" => Some("image/gif"),
            "svg" => Some("image/svg+xml"),
            "ico" => Some("image/x-icon"),
            "woff" => Some("font/woff"),
            "woff2" => Some("font/woff2"),
            _ => Some("application/octet-stream"),
        }
    }

    fn create_fallback_index(&self) -> String {
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Axum SSE Demo</title>
    <style>
        body { 
            font-family: Arial, sans-serif; 
            max-width: 800px; 
            margin: 0 auto; 
            padding: 20px;
            background-color: #1a1a1a;
            color: #ffffff;
        }
        .time-display { 
            font-size: 2em; 
            text-align: center; 
            margin: 20px 0;
            padding: 20px;
            border: 1px solid #333;
            background-color: #2a2a2a;
        }
        .banner {
            background-color: #dc3545;
            color: white;
            padding: 10px;
            text-align: center;
            display: none;
        }
        .nav { margin: 20px 0; }
        .nav a { color: #6c757d; margin-right: 20px; text-decoration: none; }
        .nav a:hover { color: #ffffff; }
    </style>
</head>
<body>
    <div class="banner" id="connectionBanner">Connection lost - attempting to reconnect...</div>
    <nav class="nav">
        <a href="/">Main</a>
        <a href="/about">About</a>
    </nav>
    <h1>Axum SSE Time Broadcasting</h1>
    <div class="time-display" id="timeDisplay">Connecting...</div>
    <p>This is a fallback page. The full SvelteKit frontend should be embedded here.</p>
    
    <script>
        const timeDisplay = document.getElementById('timeDisplay');
        const banner = document.getElementById('connectionBanner');
        
        function connectSSE() {
            const eventSource = new EventSource('/api/time/stream');
            
            eventSource.onmessage = function(event) {
                const timeData = JSON.parse(event.data);
                timeDisplay.textContent = timeData.formatted_time;
                banner.style.display = 'none';
            };
            
            eventSource.onerror = function(event) {
                banner.style.display = 'block';
                setTimeout(connectSSE, 5000);
            };
        }
        
        connectSSE();
    </script>
</body>
</html>"#.to_string()
    }

    fn count_files_recursive(&self, dir: &Dir) -> usize {
        let mut count = 0;
        for entry in dir.entries() {
            match entry {
                include_dir::DirEntry::File(_) => count += 1,
                include_dir::DirEntry::Dir(subdir) => count += self.count_files_recursive(subdir),
            }
        }
        count
    }

    fn list_files_recursive(&self, dir: &Dir, prefix: &str) -> Vec<String> {
        let mut files = Vec::new();
        for entry in dir.entries() {
            match entry {
                include_dir::DirEntry::File(file) => {
                    let path = if prefix.is_empty() {
                        file.path().to_string_lossy().to_string()
                    } else {
                        format!("{}/{}", prefix, file.path().to_string_lossy())
                    };
                    files.push(path);
                }
                include_dir::DirEntry::Dir(subdir) => {
                    let subprefix = if prefix.is_empty() {
                        subdir.path().to_string_lossy().to_string()
                    } else {
                        format!("{}/{}", prefix, subdir.path().to_string_lossy())
                    };
                    files.extend(self.list_files_recursive(subdir, &subprefix));
                }
            }
        }
        files
    }
}

impl Default for StaticService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_static_service_creation() {
        let service = StaticService::new();
        assert_eq!(service.index_file, "index.html");
    }

    #[tokio::test]
    async fn test_fallback_pages() {
        let service = StaticService::new();
        
        // Test index page (might serve embedded content or fallback)
        let index_result = service.serve_index().await;
        assert!(index_result.is_ok());
        let index_html = index_result.unwrap().0;
        // Just check it contains some expected content
        assert!(index_html.contains("html") || index_html.contains("HTML"));
    }

    #[test]
    fn test_path_sanitization() {
        let service = StaticService::new();
        
        assert_eq!(service.sanitize_path("/normal/path.html"), "normal/path.html");
        assert_eq!(service.sanitize_path("../../../etc/passwd"), "404.html");
        assert_eq!(service.sanitize_path("path/with/../traversal"), "404.html");
    }

    #[test]
    fn test_content_type_detection() {
        let service = StaticService::new();
        
        assert_eq!(service.get_content_type("test.html"), Some("text/html; charset=utf-8"));
        assert_eq!(service.get_content_type("style.css"), Some("text/css"));
        assert_eq!(service.get_content_type("script.js"), Some("application/javascript"));
        assert_eq!(service.get_content_type("data.json"), Some("application/json"));
        assert_eq!(service.get_content_type("unknown.xyz"), Some("application/octet-stream"));
    }

    #[test]
    fn test_health_check() {
        let service = StaticService::new();
        // Note: In a real build, FRONTEND_DIR might be empty during testing
        // The health check should still work
        let _is_healthy = service.is_healthy();
        let _asset_count = service.asset_count();
        let _assets = service.list_assets();
    }
}