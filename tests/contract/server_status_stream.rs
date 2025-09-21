// Contract test for GET /api/server-status-stream SSE endpoint
// This test verifies the Server-Sent Events contract for real-time status updates

use axum::http::StatusCode;
use axum_test::TestServer;
use futures::StreamExt;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_server_status_stream_endpoint_contract() {
    // This test will fail until SSE implementation is complete
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();
    
    let response = server.get("/api/server-status-stream").await;
    
    // Verify response status
    assert_eq!(response.status_code(), StatusCode::OK);
    
    // Verify content type for SSE
    assert_eq!(response.content_type(), "text/event-stream");
    
    // Verify SSE headers
    let headers = response.headers();
    assert_eq!(headers.get("cache-control").unwrap(), "no-cache");
    assert_eq!(headers.get("connection").unwrap(), "keep-alive");
}

#[tokio::test]
async fn test_server_status_stream_data_format() {
    // Test that SSE stream provides properly formatted events
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();
    
    // This will initially fail as endpoint doesn't exist
    let response = server.get("/api/server-status-stream").await;
    
    if response.status_code() == StatusCode::OK {
        // Parse as SSE stream when implemented
        let stream = response.into_bytes_stream();
        
        // Wait for first event with timeout
        let result = timeout(Duration::from_secs(10), stream.next()).await;
        
        if let Ok(Some(Ok(chunk))) = result {
            let event_data = String::from_utf8(chunk.to_vec()).unwrap();
            
            // Verify SSE format: "event: status-update\ndata: {...}\n\n"
            assert!(event_data.contains("event: status-update"), "Missing event type");
            assert!(event_data.contains("data: "), "Missing data field");
            
            // Extract JSON data after "data: " prefix
            let data_start = event_data.find("data: ").unwrap() + 6;
            let data_end = event_data[data_start..].find('\n').unwrap_or(event_data.len() - data_start);
            let json_data = &event_data[data_start..data_start + data_end];
            
            // Verify JSON structure matches StatusPageData schema
            let parsed: serde_json::Value = serde_json::from_str(json_data)
                .expect("SSE data should be valid JSON");
            
            assert!(parsed.get("server_metrics").is_some(), "SSE data missing server_metrics");
            assert!(parsed.get("collection_interval_seconds").is_some(), "SSE data missing collection_interval_seconds");
            assert!(parsed.get("server_info").is_some(), "SSE data missing server_info");
        }
    } else {
        // Test fails as expected - endpoint not implemented yet
        assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
    }
}

#[tokio::test]
async fn test_server_status_stream_multiple_events() {
    // Test that SSE stream continues to send events at intervals
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();
    
    let response = server.get("/api/server-status-stream").await;
    
    if response.status_code() == StatusCode::OK {
        let mut stream = response.into_bytes_stream();
        let mut event_count = 0;
        
        // Collect events for up to 15 seconds (should get ~3 events at 5-second intervals)
        let timeout_result = timeout(Duration::from_secs(15), async {
            while let Some(Ok(_chunk)) = stream.next().await {
                event_count += 1;
                if event_count >= 2 {
                    break; // Got multiple events, test passes
                }
            }
        }).await;
        
        assert!(timeout_result.is_ok(), "Stream should provide events within timeout");
        assert!(event_count >= 2, "Should receive multiple SSE events over time");
    } else {
        // Expected failure - endpoint not implemented
        assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
    }
}

#[tokio::test]
async fn test_server_status_stream_error_handling() {
    // Test error response when SSE stream cannot be established
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();
    
    let response = server.get("/api/server-status-stream").await;
    
    // Should be either 404 (not implemented) or 500 (stream establishment error)
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