use std::io::Cursor;
use std::sync::Arc;

use axum::{
    extract::{Multipart, State},
    response::{Response},
};
use image::{ImageFormat};
use serde_json::{from_str, from_value};

use crate::{
    config::Config, // Assuming Config is at crate::config
    http::errors::AppError,
    image::{
        params::FormatConversionParams, // For parsing convert params
        pipeline_executor::execute_pipeline,
        pipeline_types::{PipelineOperationSpec, SupportedOperation}, // For checking op type
    },
};

const MAX_IMAGE_SIZE: usize = 10 * 1024 * 1024; // 10 MB, consistent with server config default


pub async fn process_pipeline(
    State(config): State<Arc<Config>>,
    mut multipart: Multipart,
) -> Result<Response, AppError> {
    let mut image_data: Option<Vec<u8>> = None;
    let mut operations_json_str: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::MultipartError(e.to_string()))? {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "image" | "file" => { // "file" for compatibility with h2non/imaginary common field name
                let data = field.bytes().await.map_err(|e| AppError::MultipartError(e.to_string()))?;
                if data.len() > config.server.max_body_size.min(MAX_IMAGE_SIZE) { // Use configured or default max
                    return Err(AppError::PayloadTooLarge(format!(
                        "Image size {} exceeds limit",
                        data.len()
                    )));
                }
                image_data = Some(data.into());
            }
            "operations" => {
                operations_json_str = Some(field.text().await.map_err(|e| AppError::MultipartError(e.to_string()))?);
            }
            _ => {
                // Ignore other fields or log a warning
                tracing::debug!("Ignoring unknown multipart field: {}", name);
            }
        }
    }

    let image_bytes = image_data.ok_or_else(|| AppError::BadRequest("Missing image data in multipart request".to_string()))?;
    let ops_str = operations_json_str.ok_or_else(|| AppError::BadRequest("Missing 'operations' JSON string in multipart request".to_string()))?;

    let operations_spec: Vec<PipelineOperationSpec> = from_str(&ops_str).map_err(|e| {
        AppError::BadRequest(format!("Failed to parse 'operations' JSON: {}", e))
    })?;

    if operations_spec.is_empty() {
        return Err(AppError::BadRequest("'operations' array cannot be empty".to_string()));
    }

    let image_format_guess = image::guess_format(&image_bytes)
        .map_err(|_e| AppError::UnsupportedMediaType("Could not determine image format".to_string()))?;
    
    let dynamic_image = image::load_from_memory_with_format(&image_bytes, image_format_guess)
        .map_err(|e| AppError::ImageProcessingError(format!("Failed to load image: {}", e)))?;

    let processed_image = execute_pipeline(dynamic_image, operations_spec.clone())?; // Clone spec for later inspection

    // Determine output format
    let mut output_image_format = ImageFormat::Png; // Default
    let mut final_content_type = output_image_format.to_mime_type();

    // Check the last operation for a successful 'Convert' to determine output format
    // We iterate specs again because execute_pipeline consumes it if not cloned, 
    // and we need the original spec to check its `params`.
    for spec in operations_spec.iter().rev() { // Iterate in reverse to find the *last* convert
        if spec.operation == SupportedOperation::Convert {
            // If a convert operation failed and was ignored, we wouldn't reach here
            // unless it was the *last* operation. execute_pipeline handles internal errors.
            // So, if we are here, any convert op we find *should* have been part of the success.
            match from_value::<FormatConversionParams>(spec.params.clone()) {
                Ok(convert_params) => {
                    match convert_params.format.to_lowercase().as_str() {
                        "png" => output_image_format = ImageFormat::Png,
                        "jpeg" | "jpg" => output_image_format = ImageFormat::Jpeg,
                        "gif" => output_image_format = ImageFormat::Gif,
                        "webp" => output_image_format = ImageFormat::WebP,
                        "bmp" => output_image_format = ImageFormat::Bmp,
                        "tiff" | "tif" => output_image_format = ImageFormat::Tiff,
                        // Add other formats supported by the `image` crate as needed
                        _ => {
                            // Log a warning if format is not recognized, but use default.
                            // The pipeline executor should have already validated this if it was critical.
                            tracing::warn!("Unsupported format in final convert op: {}, defaulting.", convert_params.format);
                            // Stick to previous default (or PNG)
                        }
                    }
                    final_content_type = output_image_format.to_mime_type();
                    break; // Found the last convert operation
                }
                Err(e) => {
                    // This case should ideally not happen if the pipeline executor validated params.
                    tracing::error!("Failed to parse params for a successful Convert operation: {}. Defaulting format.", e);
                    break; // Stop trying if params are corrupt for a supposedly successful op
                }
            }
        }
    }

    let mut final_image_bytes = Vec::new();
    processed_image
        .write_to(&mut Cursor::new(&mut final_image_bytes), output_image_format)
        .map_err(|e| AppError::ImageProcessingError(format!("Failed to write processed image: {}", e)))?;

    Ok(Response::builder()
        .header("Content-Type", final_content_type)
        .body(axum::body::Body::from(final_image_bytes))
        .map_err(|e| AppError::InternalServerError(format!("Failed to build response: {}", e)))?)
}

// TODO:
// - Add support for GET requests with URL/file params for image source.
// - Consider content negotiation for output format as an alternative or addition.
// - Unit/Integration tests for this handler.
// - Default to original image format if no convert op, instead of always PNG.