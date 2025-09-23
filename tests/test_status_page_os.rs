// Integration test for status page OS display
// Tests that OS information appears correctly in the frontend UI
// TEST MUST FAIL until frontend OS info component is implemented

use axum_test::TestServer;

#[tokio::test]
async fn test_status_page_contains_os_section() {
    // Create test server
    let app = axum_sse::create_app().await;
    let server = TestServer::new(app).unwrap();
    
    // Test GET /status page
    let response = server.get("/status").await;
    response.assert_status_ok();
    
    // Check content type header directly
    let content_type = response.header("content-type");
    let content_type_str = content_type.to_str().unwrap_or("");
    assert!(content_type_str.contains("text/html"), "Expected HTML content type, got: {}", content_type_str);
    
    let html = response.text();
    
    // Verify the status page contains OS information section
    // This will fail until the OS info component is added to the status page
    assert!(
        html.contains("Operating System") || html.contains("OS Information") || html.contains("System Information"),
        "Status page should contain OS information section"
    );
    
    // Check for OS-related content that should be displayed
    // These will fail until the component displays actual OS info
    assert!(
        html.contains("Architecture") || html.contains("architecture"),
        "Status page should display system architecture"
    );
    
    assert!(
        html.contains("Kernel") || html.contains("kernel"),
        "Status page should display kernel information"
    );
    
    // Verify the page structure includes OS info alongside other metrics
    assert!(
        html.contains("Memory") || html.contains("memory"),
        "Status page should still contain memory metrics"
    );
    
    assert!(
        html.contains("CPU") || html.contains("cpu"),
        "Status page should still contain CPU metrics"
    );
}

#[tokio::test]
async fn test_status_page_os_info_integration() {
    // Test that status page properly integrates OS info with existing layout
    // Validates FR-004: seamless integration with existing status page layout
    
    let app = axum_sse::create_app().await;
    let server = TestServer::new(app).unwrap();
    
    let response = server.get("/status").await;
    response.assert_status_ok();
    
    let html = response.text();
    
    // Verify page structure maintains existing functionality
    assert!(html.contains("html"), "Should return valid HTML");
    assert!(html.contains("body"), "Should have body element");
    
    // Check for existing status page elements
    let required_elements = [
        "Server Status",
        "server-status", // API endpoint reference
        "metrics",
        "chart" // Chart.js elements
    ];
    
    for element in required_elements {
        assert!(
            html.contains(element),
            "Status page should contain '{}' for proper integration",
            element
        );
    }
    
    // Verify the page loads JavaScript for metrics and OS info
    assert!(
        html.contains("script") || html.contains(".js"),
        "Status page should include JavaScript for functionality"
    );
}

#[tokio::test]
async fn test_status_page_responsive_layout() {
    // Test that OS info integration maintains responsive design
    // Validates that new OS section doesn't break mobile layout
    
    let app = axum_sse::create_app().await;
    let server = TestServer::new(app).unwrap();
    
    let response = server.get("/status").await;
    response.assert_status_ok();
    
    let html = response.text();
    
    // Check for responsive design elements
    let responsive_indicators = [
        "viewport",
        "mobile",
        "responsive",
        "@media",
        "flex",
        "grid"
    ];
    
    let has_responsive_design = responsive_indicators.iter().any(|&indicator| {
        html.to_lowercase().contains(indicator)
    });
    
    assert!(
        has_responsive_design,
        "Status page should maintain responsive design with OS info integration"
    );
    
    // Verify no obvious layout breaking elements
    assert!(
        !html.contains("overflow"),
        "Status page should not have obvious overflow issues"
    );
}

#[tokio::test]
async fn test_status_page_accessibility() {
    // Test that OS info section maintains accessibility standards
    // Validates FR-006: accessible to all users who can view status page
    
    let app = axum_sse::create_app().await;
    let server = TestServer::new(app).unwrap();
    
    let response = server.get("/status").await;
    response.assert_status_ok();
    
    let html = response.text();
    
    // Check for accessibility elements
    let accessibility_indicators = [
        "aria-",
        "role=",
        "alt=",
        "title=",
        "label"
    ];
    
    let has_accessibility_features = accessibility_indicators.iter().any(|&indicator| {
        html.contains(indicator)
    });
    
    assert!(
        has_accessibility_features,
        "Status page should maintain accessibility features with OS info"
    );
    
    // Check for semantic HTML structure
    let semantic_elements = ["main", "section", "article", "header", "nav"];
    let has_semantic_html = semantic_elements.iter().any(|&element| {
        html.contains(&format!("<{}", element))
    });
    
    assert!(
        has_semantic_html,
        "Status page should use semantic HTML elements"
    );
}

#[tokio::test]
async fn test_status_page_error_handling() {
    // Test that status page gracefully handles OS info errors
    // Validates FR-005: graceful handling when OS info unavailable
    
    let app = axum_sse::create_app().await;
    let server = TestServer::new(app).unwrap();
    
    // Even if OS info collection fails, status page should still load
    let response = server.get("/status").await;
    response.assert_status_ok();
    
    let html = response.text();
    
    // Page should still be functional even with OS info errors
    assert!(html.contains("html"), "Page should still render HTML");
    assert!(html.len() > 100, "Page should have substantial content");
    
    // Should not contain obvious error messages in HTML
    let error_indicators = ["Error:", "Failed:", "Exception:", "undefined", "null"];
    let has_obvious_errors = error_indicators.iter().any(|&error| {
        html.contains(error)
    });
    
    assert!(
        !has_obvious_errors,
        "Status page should not display obvious error messages to users"
    );
    
    // Should still contain core functionality
    assert!(
        html.contains("status") || html.contains("metrics"),
        "Core status page functionality should remain intact"
    );
}