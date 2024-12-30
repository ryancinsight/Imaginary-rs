use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};

use crate::http::handlers::{health_check, process_image};

pub fn create_router() -> Router {
    Router::new()
        .route("/", get(health_check))
        .route("/process", post(process_image))
        .layer(CorsLayer::new().allow_origin(Any))
}