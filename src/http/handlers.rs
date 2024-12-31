use axum::{
    extract::{Multipart, State},
    response::Response,
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use axum::extract::multipart::Field;
use crate::storage;
use image::ImageFormat;
use serde_json::json;
use crate::image::operations;
use std::fs::{self, File};
use std::io::Write;
use tracing::info;
use std::env;
use std::sync::Arc;
use crate::http::errors::AppError;
use crate::config::Config;
use crate::storage::{cache_result, get_result};
use std::path::PathBuf;
use crate::image::params::{ResizeParams, FormatConversionParams};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub async fn health_check() -> impl IntoResponse {
    info!("Health check endpoint called");
    Json(json!({
        "status": "OK",
        "message": "Health check OK",
        "version": VERSION,
        "uptime": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }))
}

pub async fn process_image(
    State(config): State<Arc<Config>>,
    mut multipart: Multipart
) -> Result<Response, AppError> {
    info!("Processing image upload");
    
    // Use configured temp directory
    let temp_dir = config.storage.temp_dir.to_str().unwrap_or("temp");
    if let Err(e) = fs::create_dir_all(temp_dir) {
        return Err(AppError::FileSystemError(format!("Failed to create temp directory: {}", e)));
    }
    
    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::MultipartError(e.to_string()))? {
        // Early metadata check
        if let Some(cached_path) = check_early_cache(&field).await {
            if cached_path.exists() {
                return Ok((StatusCode::OK, Json(json!({
                    "status": "success",
                    "message": "Image retrieved from cache (metadata match)",
                    "output_path": cached_path
                }))).into_response());
            }
        }

        if let Some(content_length) = field.headers().get(header::CONTENT_LENGTH) {
            let size = content_length.to_str()
                .unwrap_or("0")
                .parse::<usize>()
                .unwrap_or(0);
            
            if size > 10 * 1024 * 1024 {  // 10MB
                return Err(AppError::PayloadTooLarge("Payload too large".to_string()));
            }
        }

        let name = field.file_name()
            .unwrap_or("uploaded_image")
            .to_string();

        let data = match field.bytes().await {
            Ok(data) => data,
            Err(e) => return Err(AppError::MultipartError(format!("Failed to read file data: {}", e))),
        };

        info!("Received file: {}", name);

        // Save the uploaded image to a temporary file
        let file_path = format!("{}/{}", temp_dir, name);
        let mut file = match File::create(&file_path) {
            Ok(f) => f,
            Err(e) => return Err(AppError::FileSystemError(format!("Failed to create file: {}", e))),
        };
        if let Err(e) = file.write_all(&data) {
            return Err(AppError::FileSystemError(format!("Failed to write to file: {}", e)));
        }

        // Ensure the file is flushed and closed properly
        drop(file);

        // Load the image
        let img = match image::open(&file_path) {
            Ok(i) => i,
            Err(e) => {
                info!("Failed to open image: {}", e);
                return Err(AppError::ImageProcessingError(format!("Failed to open image: {}", e)));
            },
        };

        let operation_params = format!("resize_{}x{}", 100, 100); // Example parameters
        
        // Check cache first
        if let Some(cached_path) = get_result(&PathBuf::from(&file_path), "resize", &operation_params) {
            if cached_path.exists() {
                return Ok((StatusCode::OK, Json(json!({
                    "status": "success",
                    "message": "Image retrieved from cache",
                    "output_path": cached_path
                }))).into_response());
            }
        }

        // Perform the resize operation (example)
        let params = ResizeParams { width: 100, height: 100 };
        let resized_img = operations::resize(img, &params);

        // Save the processed image
        let output_path = format!("{}/processed_{}", temp_dir, name);
        if let Err(e) = resized_img.save(&output_path) {
            return Err(AppError::FileSystemError(format!("Failed to save processed image: {}", e)));
        }

        // Cache the result
        cache_result(
            &PathBuf::from(&file_path),
            "resize",
            &operation_params,
            &PathBuf::from(&output_path)
        );

        info!("Image processed successfully: {}", output_path);

        return Ok((StatusCode::OK, Json(json!({
            "status": "success",
            "message": "Image processed successfully",
            "output_path": output_path
        }))).into_response());
    }

    Err(AppError::BadRequest("Bad request".to_string()))
}

async fn check_early_cache(field: &Field<'_>) -> Option<PathBuf> {
    let filename = field.file_name()?;
    let content_type = field.content_type()?.to_string();
    let content_length = field.headers()
        .get(header::CONTENT_LENGTH)?
        .to_str()
        .ok()?
        .parse::<usize>()
        .ok()?;

    storage::check_cached_metadata(
        filename,
        content_length,
        &content_type,
        "resize",
        "100x100",
    )
}

pub async fn convert_image_format(
    State(config): State<Arc<Config>>,
    mut multipart: Multipart
) -> Result<Response, AppError> {
    info!("Processing format conversion");
    
    // Use configured temp directory
    let temp_dir = config.storage.temp_dir.to_str().unwrap_or("temp");
    if let Err(e) = fs::create_dir_all(temp_dir) {
        return Err(AppError::FileSystemError(format!("Failed to create temp directory: {}", e)));
    }
    
    let field = match multipart.next_field().await {
        Ok(Some(field)) => field,
        Ok(None) => return Err(AppError::BadRequest("Bad request".to_string())),
        Err(e) => return Err(AppError::MultipartError(format!("Failed to process upload: {}", e))),
    };

    let name = field.file_name()
        .unwrap_or("uploaded_image")
        .to_string();

    let data = match field.bytes().await {
        Ok(data) => data,
        Err(e) => return Err(AppError::MultipartError(format!("Failed to read file data: {}", e))),
    };

    let format = match ImageFormat::from_path(name.clone()) {
        Ok(f) => f,
        Err(_e) => return Err(AppError::UnsupportedMediaType("Unsupported media type".to_string())),
    };

    // Simulate rate limit exceeded error
    if false {
        return Err(AppError::RateLimitExceeded("Rate limit exceeded".to_string()));
    }

    // Simulate invalid operation error
    if false {
        return Err(AppError::InvalidOperation("Invalid operation example".to_string()));
    }

    // Process image conversion
    let temp_path = format!("{}/{}", temp_dir, name);
    let mut file = match File::create(&temp_path) {
        Ok(f) => f,
        Err(e) => return Err(AppError::FileSystemError(format!("Failed to create file: {}", e))),
    };
    if let Err(e) = file.write_all(&data) {
        return Err(AppError::FileSystemError(format!("Failed to write to file: {}", e)));
    }

    // Ensure the file is flushed and closed properly
    drop(file);

    let img = match image::open(&temp_path) {
        Ok(i) => i,
        Err(e) => {
            info!("Failed to open image: {}", e);
            return Err(AppError::ImageProcessingError(format!("Failed to open image: {}", e)));
        },
    };
    let params = FormatConversionParams { format: format!("{:?}", format), quality: None };
    let converted = match operations::convert_format(img, &params) {
        Ok(c) => c,
        Err(e) => return Err(AppError::ImageProcessingError(format!("Failed to convert image: {}", e))),
    };
    
    let output_path = format!("{}/converted_{}", temp_dir, name);
    if let Err(e) = converted.save(&output_path) {
        return Err(AppError::FileSystemError(format!("Failed to save converted image: {}", e)));
    }

    Ok((StatusCode::OK, Json(json!({
        "status": "success",
        "message": "Image converted successfully",
        "output_path": output_path
    }))).into_response())
}