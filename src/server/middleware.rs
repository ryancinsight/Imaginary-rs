use axum::http::{Request, Response};
use axum::response::IntoResponse;
use reqwest::StatusCode;
use axum::middleware::Next;
use tracing::info;

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

pub async fn authenticate(req: Request<axum::body::Body>, next: Next) -> Response<axum::body::Body> {
    // Example: Check for an API key in the headers
    if let Some(api_key) = req.headers().get("x-api-key") {
        if api_key != "your_secret_api_key" {
            return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
        }
    } else {
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    next.run(req).await
}
