use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;
use tracing::info;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub async fn health_check() -> impl IntoResponse {
    info!("Health check endpoint called");
    Json(json!({
        "status": "OK",
        "message": "Health check OK",
        "version": VERSION,
        "uptime": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default() // Use unwrap_or_default for robustness
            .as_secs()
    }))
} 