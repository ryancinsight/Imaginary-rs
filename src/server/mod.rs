//! Server setup and execution module.
//!
//! This module is responsible for configuring and running the Axum web server.
//! It sets up common middleware, routing, and a two-tier error handling strategy:
//!
//! 1.  **Panic Handling**: The `CatchPanicLayer` middleware catches any panics that occur
//!     during request processing and converts them into HTTP 500 responses.
//! 2.  **Service-Level Error Handling (Timeouts)**: The `TimeoutLayer` enforces request timeouts.
//!     If a timeout occurs (producing a `tower::timeout::error::Elapsed` error, which is a `BoxError`),
//!     the subsequent `HandleErrorLayer` (using `outer_error_handler`) catches this specific error
//!     and converts it into an HTTP 408 Request Timeout response. Other `BoxError` types caught by
//!     this handler are converted to HTTP 500 Internal Server Error responses.
//!
//! The `create_router` function constructs the main application router with common middleware.
//! It is designed to produce an `Infallible` service from the router's perspective, meaning
//! its own errors are either handled internally by Axum (e.g., 404s) or are panics caught by `CatchPanicLayer`.
//!
//! `BoxCloneService` is utilized for type erasure to simplify service types, making them suitable
//! for use with `axum::serve`.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use serde::Deserialize;
use tracing::{info, Level};
use axum::{
    Router,
    routing::{get, post},
    http::{Request, StatusCode, HeaderName, Response},
    body::Body,
    BoxError,
    Json,
    response::IntoResponse,
    ServiceExt,
};
use tower_http::{
    cors::{Any, CorsLayer},
    compression::CompressionLayer,
    trace::{TraceLayer, DefaultOnResponse, DefaultMakeSpan, DefaultOnRequest},
    request_id::{SetRequestIdLayer, MakeRequestUuid},
    catch_panic::CatchPanicLayer,
};
use tower::ServiceBuilder;
use tower::util::BoxCloneService;
use crate::config::Config;
use crate::http::handlers::health_handler::health_check;
use crate::http::errors::AppError;
use serde_json::json;
use std::convert::Infallible;
use tokio::net::TcpListener;
use crate::http::handlers::legacy_process_handler::process_image;
use crate::http::handlers::pipeline_handler::process_pipeline;
use tokio::sync::Semaphore;
use crate::server::middleware::concurrency_limit_middleware;

pub mod middleware;

#[derive(Debug, Deserialize, Default)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_read_timeout")]
    pub read_timeout: u64,
    #[serde(default = "default_write_timeout")]
    pub write_timeout: u64,
    #[serde(default = "default_concurrency")]
    pub concurrency: usize,
    #[serde(default = "default_max_body_size")]
    pub max_body_size: usize,
}

fn default_port() -> u16 { 8080 }
fn default_host() -> String { "127.0.0.1".to_string() }
fn default_read_timeout() -> u64 { 30 }
fn default_write_timeout() -> u64 { 30 }
fn default_concurrency() -> usize { 4 }
fn default_max_body_size() -> usize { 10 * 1024 * 1024 }

pub fn create_router(config: Arc<Config>) -> BoxCloneService<Request<Body>, Response<Body>, Infallible> {
    let common_middleware = ServiceBuilder::new()
        .layer(SetRequestIdLayer::new(HeaderName::from_static("x-request-id"), MakeRequestUuid::default()))
        .layer(TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().level(Level::INFO).include_headers(true))
            .on_request(DefaultOnRequest::new().level(Level::INFO))
            .on_response(DefaultOnResponse::new().level(Level::INFO).latency_unit(tower_http::LatencyUnit::Micros)))
        .layer(CorsLayer::new()
            .allow_origin(Any))
        .layer(CompressionLayer::new())
        .layer(CatchPanicLayer::new());

    let router_service = Router::new()
        .route("/health", get(health_check))
        .route("/process", post(process_image))
        .route("/pipeline", post(process_pipeline))
        .layer(common_middleware)
        .with_state(config);

    BoxCloneService::new(router_service)
}

// Define the error handler as a standalone async function
async fn outer_error_handler(err: BoxError) -> Response<Body> {
    tracing::error!(error = %err, "Outer error handler (timeout or propagated)");
    if err.is::<tower::timeout::error::Elapsed>() {
        (StatusCode::REQUEST_TIMEOUT, Json(json!({ "error": "Request timed out" }))).into_response()
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("Unhandled error: {}", err) }))).into_response()
    }
}

pub async fn run_server(config: Arc<Config>, semaphore: Option<Arc<Semaphore>>) -> Result<(), AppError> {
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port)
        .parse()
        .map_err(|_| AppError::InternalServerError("Failed to parse address".to_string()))?;

    let std_listener = std::net::TcpListener::bind(addr)
        .map_err(|e| AppError::InternalServerError(format!("Failed to bind std listener: {}", e)))?;
    std_listener.set_nonblocking(true)
        .map_err(|e| AppError::InternalServerError(format!("Failed to set std listener to non-blocking: {}", e)))?;
    let listener = TcpListener::from_std(std_listener)
        .map_err(|e| AppError::InternalServerError(format!("Failed to convert std listener to tokio listener: {}", e)))?;

    let common_middleware = ServiceBuilder::new()
        .layer(SetRequestIdLayer::new(HeaderName::from_static("x-request-id"), MakeRequestUuid::default()))
        .layer(TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().level(Level::INFO).include_headers(true))
            .on_request(DefaultOnRequest::new().level(Level::INFO))
            .on_response(DefaultOnResponse::new().level(Level::INFO).latency_unit(tower_http::LatencyUnit::Micros)))
        .layer(CorsLayer::new().allow_origin(Any))
        .layer(CompressionLayer::new())
        .layer(CatchPanicLayer::new());

    let mut router = Router::new()
        .route("/health", get(health_check))
        .route("/process", post(process_image))
        .route("/pipeline", post(process_pipeline))
        .layer(common_middleware)
        .with_state(config.clone());

    if let Some(semaphore) = semaphore {
        router = router.layer(axum::middleware::from_fn_with_state(
            semaphore,
            concurrency_limit_middleware,
        ));
    }

    let app_service_inner = BoxCloneService::new(router);
    let timeout_duration = config.server.read_timeout;

    // Explicitly compose Timeout and HandleError services
    let timed_service = tower::timeout::Timeout::new(
        app_service_inner, // This is BoxCloneService<..., Infallible>
        Duration::from_secs(timeout_duration),
    );

    let final_service_logic = axum::error_handling::HandleError::new(
        timed_service,
        outer_error_handler, // Takes BoxError, returns Response<Body>
    );
    let boxed_final_service = BoxCloneService::new(final_service_logic);

    info!("Starting server on {}", addr);
    axum::serve(listener, boxed_final_service.into_make_service())
        .await
        .map_err(|e| AppError::InternalServerError(format!("Server failed: {}", e)))?;

    Ok(())
}