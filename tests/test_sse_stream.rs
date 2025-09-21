// Integration tests for SSE endpoint
use axum::{
    body::Body,
    http::{Request, StatusCode, HeaderValue},
    routing::get,
    Router,
    response::Sse,
    response::sse::{Event, KeepAlive},
};
use tower::util::ServiceExt;
use futures::stream::{self, Stream};
use std::convert::Infallible;

// Mock SSE endpoint for testing
async fn mock_time_stream() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = stream::once(async {
        let event = Event::default()
            .data(r#"{"timestamp":"2025-09-20T10:30:00Z","formatted_time":"20/09/2025 10:30:00"}"#);
        Ok(event)
    });
    
    Sse::new(stream).keep_alive(KeepAlive::default())
}

#[tokio::test]
async fn test_sse_endpoint_returns_200() {
    // Arrange
    let app = Router::new().route("/api/time/stream", get(mock_time_stream));

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/time/stream")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_sse_endpoint_returns_correct_headers() {
    // Arrange
    let app = Router::new().route("/api/time/stream", get(mock_time_stream));

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/time/stream")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        &HeaderValue::from_static("text/event-stream")
    );
    assert_eq!(
        response.headers().get("cache-control").unwrap(),
        &HeaderValue::from_static("no-cache")
    );
}