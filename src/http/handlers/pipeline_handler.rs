//! HTTP handler for the /pipeline endpoint.
//!
//! Accepts multipart/form-data with an image and a JSON array of operations.
//! Applies the operations in sequence and returns the processed image.
//!
//! Example usage:
//!   POST /pipeline
//!   - image: file
//!   - operations: '[{"operation": "resize", "params": {"width": 200, "height": 200}}]'

use std::io::Cursor;
use std::sync::Arc;

use axum::body::Bytes;
use axum::{
    extract::{Multipart, Query, State},
    http::Method,
    response::Response,
};
use image::ImageFormat;
use serde::Deserialize;
use serde_json::from_str;
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

use crate::{
    config::Config, // Assuming Config is at crate::config
    http::errors::AppError,
    http::request_utils::{fetch_image_from_url, MAX_IMAGE_SIZE},
    image::{
        pipeline_executor::execute_pipeline,
        pipeline_types::{PipelineOperation, PipelineOperationSpec},
    },
};

#[derive(Deserialize, Archive, RkyvDeserialize, RkyvSerialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct PipelineQuery {
    url: Option<String>,
    operations: String,
}

/// Handles both POST and GET /pipeline requests
///
/// POST: Accepts multipart/form-data with fields:
/// - `image`: the image file
/// - `operations`: JSON array of operation specs
///
/// GET: Accepts query parameters:
/// - `url`: URL of the image to process
/// - `operations`: JSON-encoded array of operation specs
///
/// Returns the processed image as binary data.
pub async fn process_pipeline(
    method: Method,
    State(config): State<Arc<Config>>,
    query: Option<Query<PipelineQuery>>,
    multipart: Option<Multipart>,
) -> Result<Response, AppError> {
    let (image_bytes, operations_spec, original_format) = match method.clone() {
        Method::GET | Method::HEAD => handle_get_request(query, &config).await?,
        Method::POST => handle_post_request(multipart, &config).await?,
        _ => return Err(AppError::BadRequest("Method not allowed".to_string())),
    };

    // Determine output format - default to original format unless convert operation specifies otherwise
    let output_format = determine_output_format(&operations_spec, original_format);
    let content_type = output_format.to_mime_type();

    // Calculate hash for caching
    let ops_string = serde_json::to_string(&operations_spec).unwrap_or_else(|_| "".to_string());

    // Perform hashing in a blocking task to avoid stalling the async runtime
    let image_bytes_for_hash = image_bytes.clone();
    let ops_string_for_hash = ops_string.clone();
    let hash = tokio::task::spawn_blocking(move || {
        crate::storage::calculate_hash_from_memory(
            &image_bytes_for_hash,
            "pipeline",
            &ops_string_for_hash,
        )
    })
    .await
    .map_err(|e| AppError::InternalServerError(format!("Hash calculation task failed: {}", e)))?;

    let mut response_builder = Response::builder().header("Content-Type", content_type);

    match method {
        Method::GET | Method::HEAD => {
            response_builder =
                response_builder.header("Cache-Control", "public, max-age=31536000, immutable");
        }
        Method::POST => {
            response_builder = response_builder.header("Cache-Control", "no-store");
        }
        _ => {}
    }

    // Check cache (async)
    if let Some(path) = crate::storage::get_cached_path(&hash).await {
        if let Ok(cached_bytes) = tokio::fs::read(&path).await {
            return response_builder
                .header("X-Cache", "HIT")
                .body(axum::body::Body::from(cached_bytes))
                .map_err(|e| {
                    AppError::InternalServerError(format!("Failed to build response: {}", e))
                });
        }
    }

    let config_clone = config.clone();
    let handle = tokio::runtime::Handle::current();
    let final_image_vec = tokio::task::spawn_blocking(move || {
        let dynamic_image = image::load_from_memory_with_format(&image_bytes, original_format)
            .map_err(|e| AppError::ImageProcessingError(format!("Failed to load image: {}", e)))?;

        let processed_image = execute_pipeline(dynamic_image, operations_spec, &config_clone, &handle)?;

        let mut bytes = Vec::new();
        processed_image
            .write_to(&mut Cursor::new(&mut bytes), output_format)
            .map_err(|e| {
                AppError::ImageProcessingError(format!("Failed to write processed image: {}", e))
            })?;
        Ok::<Vec<u8>, AppError>(bytes)
    })
    .await
    .map_err(|e| AppError::InternalServerError(format!("Image processing task failed: {}", e)))??;

    // Convert to Bytes immediately to allow cheap cloning (shared ownership)
    let final_image_bytes = Bytes::from(final_image_vec);

    // Save to cache (async)
    let hash_clone = hash.clone();
    let bytes_for_cache = final_image_bytes.clone();
    tokio::spawn(async move {
        if let Err(e) = crate::storage::save_buffer_to_cache(&hash_clone, &bytes_for_cache).await {
            tracing::error!("Failed to save to cache: {}", e);
        }
    });

    response_builder
        .header("X-Cache", "MISS")
        .body(axum::body::Body::from(final_image_bytes))
        .map_err(|e| AppError::InternalServerError(format!("Failed to build response: {}", e)))
}

async fn handle_get_request(
    query: Option<Query<PipelineQuery>>,
    config: &Config,
) -> Result<(Bytes, Vec<PipelineOperationSpec>, ImageFormat), AppError> {
    let Query(params) =
        query.ok_or_else(|| AppError::BadRequest("Missing query parameters".to_string()))?;

    let url = params
        .url
        .ok_or_else(|| AppError::BadRequest("Missing 'url' parameter".to_string()))?;

    // Fetch image from URL
    let image_bytes = fetch_image_from_url(&url, config).await?;

    // Parse operations
    let operations_spec: Vec<PipelineOperationSpec> = from_str(&params.operations)
        .map_err(|e| AppError::BadRequest(format!("Failed to parse 'operations' JSON: {}", e)))?;

    if operations_spec.is_empty() {
        return Err(AppError::BadRequest(
            "'operations' array cannot be empty".to_string(),
        ));
    }

    let original_format = image::guess_format(&image_bytes).map_err(|_| {
        AppError::UnsupportedMediaType("Could not determine image format".to_string())
    })?;

    Ok((image_bytes, operations_spec, original_format))
}

async fn handle_post_request(
    multipart: Option<Multipart>,
    config: &Config,
) -> Result<(Bytes, Vec<PipelineOperationSpec>, ImageFormat), AppError> {
    let mut multipart =
        multipart.ok_or_else(|| AppError::BadRequest("Missing multipart data".to_string()))?;

    let mut image_data: Option<Bytes> = None;
    let mut operations_json_str: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::MultipartError(e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "image" | "file" => {
                let data = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::MultipartError(e.to_string()))?;
                if data.len() > config.server.max_body_size.min(MAX_IMAGE_SIZE) {
                    return Err(AppError::PayloadTooLarge(format!(
                        "Image size {} exceeds limit",
                        data.len()
                    )));
                }
                image_data = Some(data);
            }
            "operations" => {
                operations_json_str = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| AppError::MultipartError(e.to_string()))?,
                );
            }
            _ => {
                tracing::debug!("Ignoring unknown multipart field: {}", name);
            }
        }
    }

    let image_bytes = image_data.ok_or_else(|| {
        AppError::BadRequest("Missing image data in multipart request".to_string())
    })?;
    let ops_str = operations_json_str.ok_or_else(|| {
        AppError::BadRequest("Missing 'operations' JSON string in multipart request".to_string())
    })?;

    let operations_spec: Vec<PipelineOperationSpec> = from_str(&ops_str)
        .map_err(|e| AppError::BadRequest(format!("Failed to parse 'operations' JSON: {}", e)))?;

    if operations_spec.is_empty() {
        return Err(AppError::BadRequest(
            "'operations' array cannot be empty".to_string(),
        ));
    }

    let original_format = image::guess_format(&image_bytes).map_err(|_| {
        AppError::UnsupportedMediaType("Could not determine image format".to_string())
    })?;

    Ok((image_bytes, operations_spec, original_format))
}


fn determine_output_format(
    operations_spec: &[PipelineOperationSpec],
    original_format: ImageFormat,
) -> ImageFormat {
    // Check the last convert operation to determine output format
    for spec in operations_spec.iter().rev() {
        if let PipelineOperation::Convert(convert_params) = &spec.operation {
            match convert_params.format.to_lowercase().as_str() {
                "png" => return ImageFormat::Png,
                "jpeg" | "jpg" => return ImageFormat::Jpeg,
                "gif" => return ImageFormat::Gif,
                "webp" => return ImageFormat::WebP,
                "avif" => return ImageFormat::Avif,
                "bmp" => return ImageFormat::Bmp,
                "tiff" | "tif" => return ImageFormat::Tiff,
                _ => {
                    tracing::warn!(
                        "Unsupported format in convert operation: {}, using original format",
                        convert_params.format
                    );
                    return original_format;
                }
            }
        }
    }

    // Default to original format if no convert operation found
    original_format
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::server::ServerConfig;
    use serde_json::json;

    #[allow(dead_code)]
    fn create_test_config() -> Arc<Config> {
        Arc::new(Config {
            server: ServerConfig {
                max_body_size: 1024 * 1024, // 1MB for tests
                ..Default::default()
            },
            ..Default::default()
        })
    }

    // Helper
    fn create_op_spec(json: serde_json::Value) -> PipelineOperationSpec {
        serde_json::from_value(json).expect("Failed to create PipelineOperationSpec")
    }

    #[test]
    fn test_determine_output_format_with_convert() {
        let operations = vec![
            create_op_spec(json!({
                "operation": "resize",
                "params": {"width": 100, "height": 100},
                "ignoreFailure": false
            })),
            create_op_spec(json!({
                "operation": "convert",
                "params": {"format": "jpeg", "quality": 85},
                "ignoreFailure": false
            })),
        ];

        let result = determine_output_format(&operations, ImageFormat::Png);
        assert_eq!(result, ImageFormat::Jpeg);
    }

    #[test]
    fn test_determine_output_format_without_convert() {
        let operations = vec![create_op_spec(json!({
            "operation": "resize",
            "params": {"width": 100, "height": 100},
            "ignoreFailure": false
        }))];

        let result = determine_output_format(&operations, ImageFormat::Png);
        assert_eq!(result, ImageFormat::Png);
    }

    #[test]
    fn test_determine_output_format_multiple_converts() {
        let operations = vec![
            create_op_spec(json!({
                "operation": "convert",
                "params": {"format": "png"},
                "ignoreFailure": false
            })),
            create_op_spec(json!({
                "operation": "resize",
                "params": {"width": 100, "height": 100},
                "ignoreFailure": false
            })),
            create_op_spec(json!({
                "operation": "convert",
                "params": {"format": "webp"},
                "ignoreFailure": false
            })),
        ];

        // Should use the last convert operation
        let result = determine_output_format(&operations, ImageFormat::Jpeg);
        assert_eq!(result, ImageFormat::WebP);
    }

    #[test]
    fn test_is_safe_ip_private_ranges() {
        use crate::http::request_utils::is_safe_ip;
        use std::net::{IpAddr, Ipv4Addr};

        // Private IPv4 ranges should be rejected
        assert!(!is_safe_ip(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
        assert!(!is_safe_ip(IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1))));
        assert!(!is_safe_ip(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))));

        // Loopback should be rejected
        assert!(!is_safe_ip(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))));

        // Link-local should be rejected
        assert!(!is_safe_ip(IpAddr::V4(Ipv4Addr::new(169, 254, 1, 1))));

        // Cloud metadata service should be rejected
        assert!(!is_safe_ip(IpAddr::V4(Ipv4Addr::new(169, 254, 169, 254))));

        // Carrier-grade NAT should be rejected
        assert!(!is_safe_ip(IpAddr::V4(Ipv4Addr::new(100, 64, 0, 1))));

        // Test networks should be rejected
        assert!(!is_safe_ip(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1))));
        assert!(!is_safe_ip(IpAddr::V4(Ipv4Addr::new(198, 51, 100, 1))));
        assert!(!is_safe_ip(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1))));
    }

    #[test]
    fn test_is_safe_ip_public_ranges() {
        use crate::http::request_utils::is_safe_ip;
        use std::net::{IpAddr, Ipv4Addr};

        // Public IPv4 addresses should be allowed
        assert!(is_safe_ip(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)))); // Google DNS
        assert!(is_safe_ip(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)))); // Cloudflare DNS
        assert!(is_safe_ip(IpAddr::V4(Ipv4Addr::new(208, 67, 222, 222)))); // OpenDNS
    }

    #[test]
    fn test_is_safe_ip_ipv6() {
        use crate::http::request_utils::is_safe_ip;
        use std::net::{IpAddr, Ipv6Addr};

        // IPv6 loopback should be rejected
        assert!(!is_safe_ip(IpAddr::V6(Ipv6Addr::new(
            0, 0, 0, 0, 0, 0, 0, 1
        ))));

        // IPv6 link-local should be rejected
        assert!(!is_safe_ip(IpAddr::V6(Ipv6Addr::new(
            0xfe80, 0, 0, 0, 0, 0, 0, 1
        ))));

        // IPv6 unique local should be rejected
        assert!(!is_safe_ip(IpAddr::V6(Ipv6Addr::new(
            0xfc00, 0, 0, 0, 0, 0, 0, 1
        ))));

        // IPv6 documentation should be rejected
        assert!(!is_safe_ip(IpAddr::V6(Ipv6Addr::new(
            0x2001, 0x0db8, 0, 0, 0, 0, 0, 1
        ))));

        // Public IPv6 should be allowed (Google DNS)
        assert!(is_safe_ip(IpAddr::V6(Ipv6Addr::new(
            0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8888
        ))));
    }
}
