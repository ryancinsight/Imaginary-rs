use axum::http::{Request, Response};
use axum::middleware::Next;
use tracing::info;

pub async fn log_request_and_errors(
    req: Request<axum::body::Body>,
    next: Next<axum::body::Body>,
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
