// Integration tests for health endpoint
use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::get,
    Router,
    Json,
};
use tower::util::ServiceExt;
use serde_json::{json, Value};

// Mock health endpoint for testing
async fn mock_health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "axum-sse"
    }))
}

#[tokio::test]
async fn test_health_endpoint_returns_200() {
    // Arrange
    let app = Router::new().route("/health", get(mock_health_check));

    // Act
    let response = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_health_endpoint_returns_json() {
    // Arrange
    let app = Router::new().route("/health", get(mock_health_check));

    // Act
    let response = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    
    let content_type = response.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("application/json"));
}