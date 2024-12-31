use axum::{
    routing::{get, post},
    Router,
    http::{HeaderName, StatusCode, Request},
    response::IntoResponse,
    extract::DefaultBodyLimit,
};
use std::time::Duration;
use tower::layer::Layer;
use tower_http::{
    cors::{Any, CorsLayer},
    compression::CompressionLayer,
    limit::RequestBodyLimitLayer,
    trace::{TraceLayer, DefaultOnResponse},
    timeout::TimeoutLayer,
    request_id::{SetRequestIdLayer, MakeRequestUuid},
    catch_panic::CatchPanicLayer,
};
use tracing::Level;

use crate::http::handlers::{health_check, process_image};
use crate::config::Config;
use std::sync::Arc;

// Changed the return type to be explicit about the state type
pub fn create_router() -> Router<Arc<Config>> {
    // Create base router
    Router::new()
        .route("/", get(health_check))
        .route("/process", post(process_image))
        // Add basic middleware first
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(CompressionLayer::new())
        // These layers will get their config values from the state when handling requests
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024)) // Default 10MB
        .layer(
            SetRequestIdLayer::new(
                HeaderName::from_static("x-request-id"),
                MakeRequestUuid::default(),
            )
        )
        .layer(TimeoutLayer::new(Duration::from_secs(30))) // Default 30 seconds
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024)) // Default 10MB
        // Add tracing last
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    let request_id = request
                        .headers()
                        .get("x-request-id")
                        .map(|h| h.to_str().unwrap_or_default())
                        .unwrap_or_default();
                    
                    tracing::span!(
                        Level::INFO,
                        "request",
                        method = %request.method(),
                        uri = %request.uri(),
                        request_id = %request_id,
                    )
                })
                .on_response(DefaultOnResponse::new().include_headers(true))
        )
        // Add panic handler as the outermost layer
        .layer(
            CatchPanicLayer::custom(|_| (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string()
            ).into_response())
        )
}