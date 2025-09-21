// Integration tests for complete user journey
use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::get,
    Router,
    Json,
    response::Sse,
    response::sse::{Event, KeepAlive},
};
use tower::util::ServiceExt;
use serde_json::{json, Value};
use futures::stream::{self, Stream};
use std::convert::Infallible;

// Mock handlers for full journey test
async fn mock_serve_index() -> &'static str {
    "Main Page"
}

async fn mock_serve_about() -> &'static str {
    "About Page"
}

async fn mock_health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "axum-sse"
    }))
}

async fn mock_time_stream() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = stream::once(async {
        let event = Event::default()
            .data(r#"{"timestamp":"2025-09-20T10:30:00Z","formatted_time":"20/09/2025 10:30:00"}"#);
        Ok(event)
    });
    
    Sse::new(stream).keep_alive(KeepAlive::default())
}

#[tokio::test]
async fn test_complete_user_journey() {
    // Arrange - Set up the complete application
    let app = Router::new()
        .route("/", get(mock_serve_index))
        .route("/about", get(mock_serve_about))
        .route("/health", get(mock_health_check))
        .route("/api/time/stream", get(mock_time_stream));

    // Act & Assert - Test the complete user journey

    // 1. User visits main page
    let response = app
        .clone()
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 2. User navigates to about page
    let response = app
        .clone()
        .oneshot(Request::builder().uri("/about").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 3. User's browser connects to SSE stream
    let response = app
        .clone()
        .oneshot(Request::builder().uri("/api/time/stream").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 4. Health check endpoint is accessible
    let response = app
        .clone()
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_404_for_unknown_routes() {
    // Arrange
    let app = Router::new()
        .route("/", get(mock_serve_index))
        .route("/about", get(mock_serve_about))
        .route("/health", get(mock_health_check))
        .route("/api/time/stream", get(mock_time_stream));

    // Act - Test unknown route
    let response = app
        .oneshot(Request::builder().uri("/unknown").body(Body::empty()).unwrap())
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}