use axum::{
    extract::Json,
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;
use crate::image::{operations, params::ResizeParams};

pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "Health check OK")
}

pub async fn process_image(Json(payload): Json<ResizeParams>) -> impl IntoResponse {
    // Placeholder logic for image processing
    // Load an image (this should be replaced with actual image loading logic)
    let img = image::open("path/to/image.jpg").unwrap();

    // Perform the resize operation
    let resized_img = operations::resize(img, payload.width, payload.height);

    // Save or return the processed image (this should be replaced with actual image saving logic)
    resized_img.save("path/to/output.jpg").unwrap();

    (StatusCode::OK, Json(json!({"status": "success"})))
}