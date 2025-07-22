use axum::{
    extract::{multipart::Field, Multipart, State},
    http::{header},
    response::{IntoResponse, Response, Json},
};
use serde_json::json;
use chrono;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

use crate::config::Config;
use crate::http::errors::AppError;
use crate::image::operations;
use crate::image::params::ResizeParams;
use crate::storage::{cache_result, get_result, check_cached_metadata}; // Assuming storage brings in check_cached_metadata

// Note: This handler interacts heavily with the filesystem for temporary files and a basic cache.
// This is different from the /pipeline handler which operates in-memory.

pub async fn process_image(
    State(config): State<Arc<Config>>,
    mut multipart: Multipart,
) -> Result<Response, AppError> {
    info!("Processing image upload (legacy endpoint)");

    let temp_dir_path = &config.storage.temp_dir;
    // Ensure temp_dir is a valid string path, fallback if needed, though config should guarantee PathBuf
    let temp_dir_str = temp_dir_path.to_str().unwrap_or("temp"); 

    if !temp_dir_path.exists() {
        fs::create_dir_all(temp_dir_path).map_err(|e| {
            AppError::FileSystemError(format!("Failed to create temp directory '{}': {}", temp_dir_str, e))
        })?;
    }

    #[allow(clippy::never_loop)]
    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::MultipartError(e.to_string()))? {
        // Early metadata check using the helper function
        // The operation and params for check_early_cache are hardcoded here, reflecting its original usage.
        if let Some(cached_path) = check_early_cache(&field, "resize", "100x100").await {
            if cached_path.exists() {
                info!("Image retrieved from cache (metadata match): {:?}", cached_path);
                return Ok(Json(json!({
                    "status": "success",
                    "message": "Image retrieved from cache (metadata match)",
                    "output_path": cached_path
                })).into_response());
            }
        }

        if let Some(content_length) = field.headers().get(header::CONTENT_LENGTH) {
            let size = content_length.to_str().unwrap_or("0").parse::<usize>().unwrap_or(0);
            if size > config.server.max_body_size { // Use configured max_body_size
                return Err(AppError::PayloadTooLarge(format!("Payload too large: {} bytes", size)));
            }
        }

        let name = field.file_name().unwrap_or("uploaded_image.tmp").to_string();
        let data = field.bytes().await.map_err(|e| AppError::MultipartError(format!("Failed to read file data: {}", e)))?;
        info!("Received file: {} ({} bytes)", name, data.len());

        let unique_name = format!("{}_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default(), name);
        let file_path = temp_dir_path.join(&unique_name);
        
        let mut file = File::create(&file_path)
            .map_err(|e| AppError::FileSystemError(format!("Failed to create temp file '{:?}': {}", file_path, e)))?;
        file.write_all(&data)
            .map_err(|e| AppError::FileSystemError(format!("Failed to write to temp file '{:?}': {}", file_path, e)))?;
        drop(file); // Ensure file is closed

        let img = image::open(&file_path)
            .map_err(|e| AppError::ImageProcessingError(format!("Failed to open image '{:?}': {}", file_path, e)))?;

        // Hardcoded operation for this legacy endpoint
        let operation_name_cache = "resize";
        let operation_params_cache = "100x100"; 

        if let Some(cached_path) = get_result(&file_path, operation_name_cache, operation_params_cache) {
            if cached_path.exists() {
                info!("Image retrieved from cache (post-upload): {:?}", cached_path);
                return Ok(Json(json!({
                    "status": "success",
                    "message": "Image retrieved from cache",
                    "output_path": cached_path
                })).into_response());
            }
        }

        let params = ResizeParams { width: 100, height: 100 };
        let resized_img = operations::resize(img, &params);
        
        let output_filename = format!("processed_{}", unique_name);
        let output_path = temp_dir_path.join(output_filename);

        resized_img.save(&output_path)
            .map_err(|e| AppError::FileSystemError(format!("Failed to save processed image '{:?}': {}", output_path, e)))?;

        cache_result(&file_path, operation_name_cache, operation_params_cache, &output_path);
        info!("Image processed successfully: {:?}", output_path);

        return Ok(Json(json!({
            "status": "success",
            "message": "Image processed successfully",
            "output_path": output_path
        })).into_response());
    }

    Err(AppError::BadRequest("No image field in multipart request".to_string()))
}

// Helper function for early cache check based on metadata.
// Parameters `operation` and `op_params` are passed to match the signature of check_cached_metadata.
async fn check_early_cache(field: &Field<'_>, operation: &str, op_params: &str) -> Option<PathBuf> {
    let filename = field.file_name()?;
    let content_type = field.content_type()?.to_string();
    let content_length = field.headers()
        .get(header::CONTENT_LENGTH)?
        .to_str()
        .ok()?
        .parse::<usize>()
        .ok()?;

    // Call the function from the storage module
    check_cached_metadata(filename, content_length, &content_type, operation, op_params)
} 