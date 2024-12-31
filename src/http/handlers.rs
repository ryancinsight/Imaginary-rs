use axum::{
    extract::{Multipart, State},
    response::Response,
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
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
        return Ok((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "status": "error",
            "message": format!("Failed to create temp directory: {}", e)
        }))).into_response());
    }
    
    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::MultipartError(e.to_string()))? {
        if let Some(content_length) = field.headers().get(header::CONTENT_LENGTH) {
            let size = content_length.to_str()
                .unwrap_or("0")
                .parse::<usize>()
                .unwrap_or(0);
            
            if size > 10 * 1024 * 1024 {  // 10MB
                return Err(AppError::PayloadTooLarge);
            }
        }

        let name = field.file_name()
            .unwrap_or("uploaded_image")
            .to_string();

        let data = match field.bytes().await {
            Ok(data) => data,
            Err(e) => return Ok((StatusCode::BAD_REQUEST, Json(json!({
                "status": "error",
                "message": format!("Failed to read file data: {}", e)
            }))).into_response()),
        };

        info!("Received file: {}", name);

        // Save the uploaded image to a temporary file
        let file_path = format!("{}/{}", temp_dir, name);
        let mut file = match File::create(&file_path) {
            Ok(f) => f,
            Err(e) => return Ok((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "status": "error",
                "message": format!("Failed to create file: {}", e)
            }))).into_response()),
        };
        if let Err(e) = file.write_all(&data) {
            return Ok((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "status": "error",
                "message": format!("Failed to write to file: {}", e)
            }))).into_response());
        }

        // Ensure the file is flushed and closed properly
        drop(file);

        // Load the image
        let img = match image::open(&file_path) {
            Ok(i) => i,
            Err(e) => {
                info!("Failed to open image: {}", e);
                return Ok((StatusCode::BAD_REQUEST, Json(json!({
                    "status": "error",
                    "message": format!("Failed to open image: {}", e)
                }))).into_response());
            },
        };

        // Perform the resize operation (example)
        let resized_img = operations::resize(img, 100, 100);

        // Save the processed image
        let output_path = format!("{}/processed_{}", temp_dir, name);
        if let Err(e) = resized_img.save(&output_path) {
            return Ok((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "status": "error",
                "message": format!("Failed to save processed image: {}", e)
            }))).into_response());
        }

        info!("Image processed successfully: {}", output_path);

        return Ok((StatusCode::OK, Json(json!({
            "status": "success",
            "message": "Image processed successfully",
            "output_path": output_path
        }))).into_response());
    }

    Ok((StatusCode::BAD_REQUEST, Json(json!({
        "status": "error",
        "message": "No file uploaded"
    }))).into_response())
}

pub async fn convert_image_format(
    State(config): State<Arc<Config>>,
    mut multipart: Multipart
) -> impl IntoResponse {
    info!("Processing format conversion");
    
    // Use configured temp directory
    let temp_dir = config.storage.temp_dir.to_str().unwrap_or("temp");
    if let Err(e) = fs::create_dir_all(temp_dir) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": format!("Failed to create temp directory: {}", e)
            }))
        );
    }
    
    let field = match multipart.next_field().await {
        Ok(Some(field)) => field,
        Ok(None) => return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "error",
                "message": "No file uploaded"
            }))
        ),
        Err(e) => return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "error",
                "message": format!("Failed to process upload: {}", e)
            }))
        ),
    };

    let name = field.file_name()
        .unwrap_or("uploaded_image")
        .to_string();

    let data = match field.bytes().await {
        Ok(data) => data,
        Err(e) => return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "error",
                "message": format!("Failed to read file data: {}", e)
            }))
        ),
    };

    let format = match ImageFormat::from_path(name.clone()) {
        Ok(f) => f,
        Err(e) => return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "error",
                "message": format!("Invalid image format: {}", e)
            }))
        ),
    };

    // Process image conversion
    let temp_path = format!("{}/{}", temp_dir, name);
    let mut file = match File::create(&temp_path) {
        Ok(f) => f,
        Err(e) => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": format!("Failed to create file: {}", e)
            }))
        ),
    };
    if let Err(e) = file.write_all(&data) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": format!("Failed to write to file: {}", e)
            }))
        );
    }

    // Ensure the file is flushed and closed properly
    drop(file);

    let img = match image::open(&temp_path) {
        Ok(i) => i,
        Err(e) => {
            info!("Failed to open image: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "status": "error",
                    "message": format!("Failed to open image: {}", e)
                }))
            );
        },
    };
    let converted = match operations::convert_format(img, format) {
        Ok(c) => c,
        Err(e) => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": format!("Failed to convert image: {}", e)
            }))
        ),
    };
    
    let output_path = format!("{}/converted_{}", temp_dir, name);
    if let Err(e) = converted.save(&output_path) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": format!("Failed to save converted image: {}", e)
            }))
        );
    }

    return (
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "message": "Image converted successfully",
            "output_path": output_path
        }))
    );
}