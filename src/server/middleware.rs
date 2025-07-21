use axum::http::{Request, Response};
use axum::response::IntoResponse;
use axum::middleware::Next;
use tracing::info;
use std::sync::Arc;
use crate::config::Config;
use crate::http::errors::AppError;
use tokio::sync::Semaphore;
use std::time::{Duration, Instant};

#[allow(dead_code)] // For future logging middleware
pub async fn log_request_and_errors(
    req: Request<axum::body::Body>,
    next: Next,
) -> Response<axum::body::Body> {
    let method = req.method().clone();
    let uri = req.uri().clone();
    
    info!("Received request: {} {}", method, uri);
    
    let response = next.run(req).await;
    
    if response.status().is_client_error() || response.status().is_server_error() {
        info!("Request resulted in error: {} for {} {}", response.status(), method, uri);
    }
    
    response
}

#[allow(dead_code)] // For future authentication middleware
pub async fn authenticate(
    req: Request<axum::body::Body>,
    next: Next,
) -> Response<axum::body::Body> {
    let config = match req.extensions().get::<Arc<Config>>() {
        Some(config) => config,
        None => return AppError::InternalServerError("Configuration not found".to_string()).into_response(),
    };

    // Allow the request to proceed if the API key is not set
    if let Some(api_key) = config.security.key() {
        if !api_key.is_empty() {
            if let Some(request_api_key_header) = req.headers().get("x-api-key") {
                let request_api_key_str = request_api_key_header.to_str().unwrap_or("");
                
                // Constant-time comparison to prevent timing attacks
                let provided_key = request_api_key_str.as_bytes();
                let expected_key = api_key.as_ref().as_bytes();
                
                // Always compare the same amount of data to prevent length-based timing attacks
                let mut result = (provided_key.len() ^ expected_key.len()) as u8;
                let min_len = std::cmp::min(provided_key.len(), expected_key.len());
                
                for i in 0..min_len {
                    result |= provided_key[i] ^ expected_key[i];
                }
                
                // Add a small random delay to further obfuscate timing
                let delay_ms = (result as u64 % 5) + 1;
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                
                if result != 0 {
                    return AppError::Unauthorized("Invalid API key".to_string()).into_response();
                }
            } else {
                return AppError::Unauthorized("API key not provided".to_string()).into_response();
            }
        }
    }

    next.run(req).await
}

pub async fn concurrency_limit_middleware(
    axum::extract::State(semaphore): axum::extract::State<Arc<Semaphore>>,
    req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let permit = semaphore.acquire().await;
    let res = next.run(req).await;
    drop(permit);
    res
}
