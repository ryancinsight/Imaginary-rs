use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use serde::Deserialize;
use tracing::{info, Level};
use axum::{
    Router,
    routing::{get, post},
    http::{HeaderName, StatusCode, Request},
    response::IntoResponse,
    serve,
    middleware::from_fn_with_state,
    Extension,
};
use tower_http::{
    cors::{Any, CorsLayer},
    compression::CompressionLayer,
    limit::RequestBodyLimitLayer,
    trace::{TraceLayer, DefaultOnResponse},
    timeout::TimeoutLayer,
    request_id::{SetRequestIdLayer, MakeRequestUuid},
    catch_panic::CatchPanicLayer,
};
use tokio::net::TcpListener;
use tokio::signal;
use crate::config::Config;
use crate::http::handlers::{health_check, process_image, convert_image_format};
use crate::server::middleware::authenticate;
use crate::http::errors::AppError;

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

pub fn create_router(config: Arc<Config>) -> Router<Arc<Config>> {
    Router::new()
        .route("/", get(health_check))
        .route("/process", post(process_image))
        .route("/convert", post(convert_image_format))
        .layer(from_fn_with_state(Arc::clone(&config), authenticate))
        .layer(Extension(Arc::clone(&config)))
        .layer((
            CompressionLayer::new(),
            RequestBodyLimitLayer::new(config.server.max_body_size),
            TimeoutLayer::new(Duration::from_secs(config.server.read_timeout)),
        ))
        .layer(
            CorsLayer::new()
                .allow_methods(Any)
                .allow_headers(Any)
                .max_age(Duration::from_secs(3600))
        )
        .layer(
            SetRequestIdLayer::new(
                HeaderName::from_static("x-request-id"),
                MakeRequestUuid::default(),
            )
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    let request_id = request
                        .headers()
                        .get("x-request-id")
                        .and_then(|h| h.to_str().ok())
                        .unwrap_or_default();
                    
                    tracing::span!(
                        Level::INFO,
                        "request",
                        method = %request.method(),
                        uri = %request.uri(),
                        request_id = %request_id,
                    )
                })
                .on_response(DefaultOnResponse::new().level(Level::INFO))
        )
        .layer(
            CatchPanicLayer::custom(|err: Box<dyn std::any::Any + Send + 'static>| {
                let msg = err.downcast_ref::<String>()
                    .map(|s| s.as_str())
                    .unwrap_or("Internal Server Error");
                
                (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()).into_response()
            })
        )
}

pub async fn run_server(config: Arc<Config>) -> Result<(), AppError> {
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port)
        .parse()
        .map_err(|_| AppError::InternalServerError("Failed to parse address".to_string()))?;

    let listener = std::net::TcpListener::bind(addr)
        .map_err(|_| AppError::InternalServerError("Failed to bind address".to_string()))?;
    listener.set_nonblocking(true)
        .map_err(|_| AppError::InternalServerError("Failed to set non-blocking".to_string()))?;
    let listener = TcpListener::from_std(listener)
        .map_err(|_| AppError::InternalServerError("Failed to convert listener".to_string()))?;

    let app = create_router(Arc::clone(&config)).with_state(Arc::clone(&config));
    info!("Starting server on {}", addr);
    
    let server = serve(listener, app);

    // Use the write_timeout and concurrency settings
    let _write_timeout = Duration::from_secs(config.server.write_timeout);
    let _concurrency = config.server.concurrency;

    // Wait for the server to complete or the shutdown signal
    tokio::select! {
        res = server => {
            if let Err(err) = res {
                eprintln!("Server error: {}", err);
            }
        }
        _ = signal::ctrl_c() => {
            info!("Received Ctrl+C, shutting down gracefully");
        }
    }

    Ok(())
}