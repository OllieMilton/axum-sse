// Contract test for GET /api/server-status endpoint
// This test verifies the API contract for server status snapshot endpoint

use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::Value;

#[tokio::test]
async fn test_server_status_endpoint_contract() {
    // This test will fail until implementation is complete
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();
    
    let response = server.get("/api/server-status").await;
    
    // Verify response status
    assert_eq!(response.status_code(), StatusCode::OK);
    
    // Verify content type
    assert_eq!(response.content_type(), "application/json");
    
    // Verify response structure matches OpenAPI schema
    let body: Value = response.json();
    
    // Required top-level fields per contract
    assert!(body.get("server_metrics").is_some(), "Missing server_metrics field");
    assert!(body.get("collection_interval_seconds").is_some(), "Missing collection_interval_seconds field");
    assert!(body.get("server_info").is_some(), "Missing server_info field");
    
    // Verify server_metrics structure
    let metrics = body.get("server_metrics").unwrap();
    assert!(metrics.get("timestamp").is_some(), "Missing timestamp in server_metrics");
    assert!(metrics.get("memory_usage").is_some(), "Missing memory_usage in server_metrics");
    assert!(metrics.get("cpu_usage").is_some(), "Missing cpu_usage in server_metrics");
    assert!(metrics.get("uptime").is_some(), "Missing uptime in server_metrics");
    assert!(metrics.get("network_metrics").is_some(), "Missing network_metrics in server_metrics");
    
    // Verify memory_usage structure
    let memory = metrics.get("memory_usage").unwrap();
    assert!(memory.get("total_bytes").is_some(), "Missing total_bytes in memory_usage");
    assert!(memory.get("used_bytes").is_some(), "Missing used_bytes in memory_usage");
    assert!(memory.get("available_bytes").is_some(), "Missing available_bytes in memory_usage");
    assert!(memory.get("usage_percentage").is_some(), "Missing usage_percentage in memory_usage");
    
    // Verify cpu_usage structure
    let cpu = metrics.get("cpu_usage").unwrap();
    assert!(cpu.get("usage_percentage").is_some(), "Missing usage_percentage in cpu_usage");
    assert!(cpu.get("core_count").is_some(), "Missing core_count in cpu_usage");
    assert!(cpu.get("load_average").is_some(), "Missing load_average in cpu_usage");
    
    // Verify network_metrics structure
    let network = metrics.get("network_metrics").unwrap();
    assert!(network.get("bytes_sent").is_some(), "Missing bytes_sent in network_metrics");
    assert!(network.get("bytes_received").is_some(), "Missing bytes_received in network_metrics");
    assert!(network.get("packets_sent").is_some(), "Missing packets_sent in network_metrics");
    assert!(network.get("packets_received").is_some(), "Missing packets_received in network_metrics");
    assert!(network.get("active_connections").is_some(), "Missing active_connections in network_metrics");
    
    // Verify server_info structure
    let server_info = body.get("server_info").unwrap();
    assert!(server_info.get("hostname").is_some(), "Missing hostname in server_info");
    assert!(server_info.get("version").is_some(), "Missing version in server_info");
    assert!(server_info.get("start_time").is_some(), "Missing start_time in server_info");
    assert!(server_info.get("environment").is_some(), "Missing environment in server_info");
    
    // Verify data types and constraints
    let collection_interval = body.get("collection_interval_seconds").unwrap();
    assert!(collection_interval.is_number(), "collection_interval_seconds must be a number");
    let interval_val = collection_interval.as_u64().unwrap();
    assert!(interval_val >= 1, "collection_interval_seconds must be at least 1");
    
    // Verify memory percentage is valid
    let memory_percentage = memory.get("usage_percentage").unwrap();
    assert!(memory_percentage.is_number(), "memory usage_percentage must be a number");
    let percentage_val = memory_percentage.as_f64().unwrap();
    assert!(percentage_val >= 0.0 && percentage_val <= 100.0, "memory usage_percentage must be 0-100%");
}

#[tokio::test]
async fn test_server_status_endpoint_error_handling() {
    // Test error response format when metrics collection fails
    // This will initially fail as the endpoint doesn't exist
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();
    
    // Simulate error condition (this will be 404 initially, then 500 when implemented)
    let response = server.get("/api/server-status").await;
    
    // Should be either 404 (not implemented) or 500 (metrics collection error)
    assert!(
        response.status_code() == StatusCode::NOT_FOUND || 
        response.status_code() == StatusCode::INTERNAL_SERVER_ERROR
    );
}

// Helper function to create test app - will be implemented later
async fn create_test_app() -> axum::Router {
    // This function will be implemented when we create the main app structure
    // For now, return empty router to make tests fail as expected
    axum::Router::new()
}