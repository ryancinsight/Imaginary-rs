use axum::http::{Request, Response};
use axum::response::IntoResponse;
use axum::middleware::Next;
use tracing::info;
use std::sync::Arc;
use crate::config::Config;
use crate::http::errors::AppError;

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
                // TODO: Use a constant-time comparison for API keys to prevent timing attacks (e.g., subtle crate)
                if request_api_key_str != api_key.as_ref() {
                    return AppError::Unauthorized("Invalid API key".to_string()).into_response();
                }
            } else {
                return AppError::Unauthorized("API key not provided".to_string()).into_response();
            }
        }
    }

    next.run(req).await
}
