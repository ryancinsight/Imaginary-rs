use axum::{
    extract::{Multipart, State},
    response::{IntoResponse, Response, Json},
};
use image::ImageFormat;
use serde_json::json;
use chrono;
use std::fs::{self, File};
use std::io::Write;
use std::sync::Arc;
use tracing::info;

use crate::config::Config;
use crate::http::errors::AppError;
use crate::image::operations;
use crate::image::params::{FormatConversionParams, Validate};

pub async fn convert_image_format(
    State(config): State<Arc<Config>>,
    mut multipart: Multipart,
) -> Result<Response, AppError> {
    info!("Processing format conversion (legacy endpoint)");

    let temp_dir_path = &config.storage.temp_dir;
    let temp_dir_str = temp_dir_path.to_str().unwrap_or("temp");

    if !temp_dir_path.exists() {
        fs::create_dir_all(temp_dir_path).map_err(|e| {
            AppError::FileSystemError(format!("Failed to create temp directory '{}': {}", temp_dir_str, e))
        })?;
    }

    // This handler expects only one field, the image itself.
    // The format is determined from the filename or a query param (not implemented here yet).
    let field = match multipart.next_field().await {
        Ok(Some(field)) => field,
        Ok(None) => return Err(AppError::BadRequest("Missing image file in request".to_string())),
        Err(e) => return Err(AppError::MultipartError(format!("Failed to process multipart upload: {}", e))),
    };

    let name = field.file_name().unwrap_or("uploaded_image.tmp").to_string();
    let data = field.bytes().await.map_err(|e| AppError::MultipartError(format!("Failed to read file data: {}", e)))?;
    info!("Received file for conversion: {} ({} bytes)", name, data.len());

    // This endpoint implies the target format should be known, e.g. from path or query.
    // The original code derived it from `ImageFormat::from_path(name.clone())` which is odd for a *target* format.
    // For a true conversion endpoint, target format must be specified by the client.
    // For now, I'll assume a hardcoded target or require it to be passed differently.
    // The h2non/imaginary /convert endpoint takes a `type` query parameter.
    // This legacy handler doesn't seem to parse such a param from multipart.
    // Let's assume for this refactor it attempts to convert to PNG as a default if not specified.
    // However, the original code used the *source* format as the *target* for the params struct which is a bug.
    // We need a target format. Let's default to PNG if no other info.
    // A more robust solution would parse a 'type' or 'format' field from multipart, like the /pipeline handler does for operations.
    
    // For simplicity in refactoring the existing logic, we will save the file, then reload.
    // A more efficient approach for a convert-only might pass bytes directly if possible.
    let unique_name = format!("convert_in_{}_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default(), name);
    let temp_path = temp_dir_path.join(&unique_name);
    
    let mut file = File::create(&temp_path)
        .map_err(|e| AppError::FileSystemError(format!("Failed to create temp file '{:?}': {}", temp_path, e)))?;
    file.write_all(&data)
        .map_err(|e| AppError::FileSystemError(format!("Failed to write to temp file '{:?}': {}", temp_path, e)))?;
    drop(file); // Ensure file is closed

    let img = image::open(&temp_path)
        .map_err(|e| AppError::ImageProcessingError(format!("Failed to open image '{:?}': {}", temp_path, e)))?;

    // THIS IS WHERE TARGET FORMAT IS NEEDED. Original code was flawed.
    // For now, let's default to converting to PNG for this refactor, assuming no client input for target type.
    let target_format_str = "png".to_string(); 

    let params = FormatConversionParams {
        format: target_format_str.clone(), 
        quality: None, // Quality could be another multipart field
    };
    params.validate().map_err(|e| AppError::BadRequest(format!("Format params validation error: {}",e)))?;

    let converted_img = operations::convert_format(img, &params)?;
    
    let output_filename = format!("converted_out_{}.{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default(), target_format_str);
    let output_path = temp_dir_path.join(output_filename);
    
    let output_image_format = ImageFormat::from_extension(target_format_str)
        .unwrap_or(ImageFormat::Png); // Fallback, though validation should catch bad format strings.

    converted_img.save_with_format(&output_path, output_image_format)
        .map_err(|e| AppError::FileSystemError(format!("Failed to save converted image '{:?}': {}", output_path, e)))?;
    
    info!("Image converted successfully: {:?}", output_path);

    Ok(Json(json!({
        "status": "success",
        "message": "Image converted successfully",
        "output_path": output_path
    })).into_response())
} 