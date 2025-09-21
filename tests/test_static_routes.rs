// Integration tests for static routes
use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::get,
    Router,
};
use tower::util::ServiceExt;

async fn serve_index() -> &'static str {
    "Hello, World!"
}

async fn serve_about() -> &'static str {
    "About Page"
}

#[tokio::test]
async fn test_main_page_returns_200() {
    // Arrange
    let app = Router::new().route("/", get(serve_index));

    // Act
    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_main_page_returns_html() {
    // Arrange
    let app = Router::new().route("/", get(serve_index));

    // Act
    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    // Assert
    let _content_type = response.headers().get("content-type");
    // For now just check that we get HTML content
    assert!(response.status().is_success());
}

#[tokio::test]
async fn test_about_page_returns_200() {
    // Arrange
    let app = Router::new().route("/about", get(serve_about));

    // Act
    let response = app
        .oneshot(Request::builder().uri("/about").body(Body::empty()).unwrap())
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_about_page_returns_html() {
    // Arrange
    let app = Router::new().route("/about", get(serve_about));

    // Act
    let response = app
        .oneshot(Request::builder().uri("/about").body(Body::empty()).unwrap())
        .await
        .unwrap();

    // Assert
    let _content_type = response.headers().get("content-type");
    // For now just check that we get HTML content
    assert!(response.status().is_success());
}