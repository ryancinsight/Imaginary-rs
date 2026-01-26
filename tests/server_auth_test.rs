use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use imaginary::config::Config;
use imaginary::security::ApiKey;
use imaginary::server::create_router;
use std::sync::Arc;
use tower::ServiceExt; // for oneshot

#[tokio::test]
async fn test_auth_no_key_configured() {
    let config = Arc::new(Config::default());
    let app = create_router(config);

    let request = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_auth_missing_header() {
    let mut config = Config::default();
    config.security.set_key(ApiKey::from("supersecretkey".to_string()));
    let config = Arc::new(config);
    let app = create_router(config);

    let request = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_auth_wrong_header() {
    let mut config = Config::default();
    config.security.set_key(ApiKey::from("supersecretkey".to_string()));
    let config = Arc::new(config);
    let app = create_router(config);

    let request = Request::builder()
        .uri("/health")
        .header("x-api-key", "wrongkey")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_auth_correct_header() {
    let mut config = Config::default();
    config.security.set_key(ApiKey::from("supersecretkey".to_string()));
    let config = Arc::new(config);
    let app = create_router(config);

    let request = Request::builder()
        .uri("/health")
        .header("x-api-key", "supersecretkey")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
