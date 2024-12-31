use axum::http::{Request, Response};
use axum::response::IntoResponse;
use axum::middleware::Next;
use tracing::info;
use std::sync::Arc;
use crate::config::Config;
use crate::http::errors::AppError;

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

pub async fn authenticate(
    req: Request<axum::body::Body>,
    next: Next,
) -> Response<axum::body::Body> {
    let config = match req.extensions().get::<Arc<Config>>() {
        Some(config) => config,
        None => return AppError::InternalServerError("Configuration not found".to_string()).into_response(),
    };

    // Allow the request to proceed if the API key is not set
    if let Some(api_key) = &config.security.key {
        if !api_key.is_empty() {
            if let Some(request_api_key) = req.headers().get("x-api-key") {
                let request_api_key = request_api_key.to_str().unwrap_or("");
                if request_api_key != api_key {
                    return AppError::Unauthorized("Invalid API key".to_string()).into_response();
                }
            } else {
                return AppError::Unauthorized("API key not provided".to_string()).into_response();
            }
        }
    }

    next.run(req).await
}
