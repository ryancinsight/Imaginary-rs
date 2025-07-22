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
use std::net::{IpAddr, Ipv4Addr};

use axum::{
    extract::{Multipart, State, Query},
    response::{Response},
    http::Method,
};
use image::{ImageFormat};
use serde::{Deserialize};
use serde_json::{from_str, from_value};
use once_cell::sync::Lazy;
use url::Url;

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

// Reusable HTTP client for performance
static HTTP_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("imaginary-rs/0.1.0")
        .build()
        .expect("Failed to create HTTP client")
});

#[derive(Deserialize)]
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
    let (image_bytes, operations_spec, original_format) = match method {
        Method::GET => handle_get_request(query, &config).await?,
        Method::POST => handle_post_request(multipart, &config).await?,
        _ => return Err(AppError::BadRequest("Method not allowed".to_string())),
    };

    let dynamic_image = image::load_from_memory_with_format(&image_bytes, original_format)
        .map_err(|e| AppError::ImageProcessingError(format!("Failed to load image: {}", e)))?;

    let processed_image = execute_pipeline(dynamic_image, operations_spec.clone())?;

    // Determine output format - default to original format unless convert operation specifies otherwise
    let output_format = determine_output_format(&operations_spec, original_format);
    let content_type = output_format.to_mime_type();

    let mut final_image_bytes = Vec::new();
    processed_image
        .write_to(&mut Cursor::new(&mut final_image_bytes), output_format)
        .map_err(|e| AppError::ImageProcessingError(format!("Failed to write processed image: {}", e)))?;

    Response::builder()
        .header("Content-Type", content_type)
        .body(axum::body::Body::from(final_image_bytes))
        .map_err(|e| AppError::InternalServerError(format!("Failed to build response: {}", e)))
}

async fn handle_get_request(
    query: Option<Query<PipelineQuery>>,
    config: &Config,
) -> Result<(Vec<u8>, Vec<PipelineOperationSpec>, ImageFormat), AppError> {
    let Query(params) = query.ok_or_else(|| AppError::BadRequest("Missing query parameters".to_string()))?;
    
    let url = params.url.ok_or_else(|| AppError::BadRequest("Missing 'url' parameter".to_string()))?;
    
    // Fetch image from URL
    let image_bytes = fetch_image_from_url(&url, config).await?;
    
    // Parse operations
    let operations_spec: Vec<PipelineOperationSpec> = from_str(&params.operations)
        .map_err(|e| AppError::BadRequest(format!("Failed to parse 'operations' JSON: {}", e)))?;
    
    if operations_spec.is_empty() {
        return Err(AppError::BadRequest("'operations' array cannot be empty".to_string()));
    }
    
    let original_format = image::guess_format(&image_bytes)
        .map_err(|_| AppError::UnsupportedMediaType("Could not determine image format".to_string()))?;
    
    Ok((image_bytes, operations_spec, original_format))
}

async fn handle_post_request(
    multipart: Option<Multipart>,
    config: &Config,
) -> Result<(Vec<u8>, Vec<PipelineOperationSpec>, ImageFormat), AppError> {
    let mut multipart = multipart.ok_or_else(|| AppError::BadRequest("Missing multipart data".to_string()))?;
    
    let mut image_data: Option<Vec<u8>> = None;
    let mut operations_json_str: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::MultipartError(e.to_string()))? {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "image" | "file" => {
                let data = field.bytes().await.map_err(|e| AppError::MultipartError(e.to_string()))?;
                if data.len() > config.server.max_body_size.min(MAX_IMAGE_SIZE) {
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
                tracing::debug!("Ignoring unknown multipart field: {}", name);
            }
        }
    }

    let image_bytes = image_data.ok_or_else(|| AppError::BadRequest("Missing image data in multipart request".to_string()))?;
    let ops_str = operations_json_str.ok_or_else(|| AppError::BadRequest("Missing 'operations' JSON string in multipart request".to_string()))?;

    let operations_spec: Vec<PipelineOperationSpec> = from_str(&ops_str)
        .map_err(|e| AppError::BadRequest(format!("Failed to parse 'operations' JSON: {}", e)))?;

    if operations_spec.is_empty() {
        return Err(AppError::BadRequest("'operations' array cannot be empty".to_string()));
    }

    let original_format = image::guess_format(&image_bytes)
        .map_err(|_| AppError::UnsupportedMediaType("Could not determine image format".to_string()))?;

    Ok((image_bytes, operations_spec, original_format))
}

/// Checks if an IP address is safe for external requests (not private/internal)
fn is_safe_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => {
            // Use De Morgan's law to simplify boolean expression
            !(ipv4.is_private() || 
              ipv4.is_loopback() || 
              ipv4.is_link_local() || 
              ipv4.is_broadcast() || 
              ipv4.is_multicast() || 
              // Reject carrier-grade NAT (RFC 6598)
              (ipv4.octets()[0] == 100 && (64..128).contains(&ipv4.octets()[1])) ||
              // Reject cloud metadata service IPs
              ipv4 == Ipv4Addr::new(169, 254, 169, 254) ||
              // Reject test networks (RFC 5737)
              (ipv4.octets()[0] == 192 && ipv4.octets()[1] == 0 && ipv4.octets()[2] == 2) ||
              (ipv4.octets()[0] == 198 && ipv4.octets()[1] == 51 && ipv4.octets()[2] == 100) ||
              (ipv4.octets()[0] == 203 && ipv4.octets()[1] == 0 && ipv4.octets()[2] == 113) ||
              // Reject documentation range (RFC 3849) - remove unnecessary parentheses
              (ipv4.octets()[0] == 192 && ipv4.octets()[1] == 88 && ipv4.octets()[2] == 99))
        }
        IpAddr::V6(ipv6) => {
            // Use De Morgan's law and simplified expressions
            !(ipv6.is_loopback() || 
              ipv6.is_multicast() || 
              // Reject link-local
              ipv6.segments()[0] & 0xffc0 == 0xfe80 ||
              // Reject unique local addresses (RFC 4193)
              ipv6.segments()[0] & 0xfe00 == 0xfc00 ||
              // Reject documentation prefix (RFC 3849)
              (ipv6.segments()[0] == 0x2001 && ipv6.segments()[1] == 0x0db8))
        }
    }
}

async fn fetch_image_from_url(url_str: &str, config: &Config) -> Result<Vec<u8>, AppError> {
    // Parse and validate URL
    let url = Url::parse(url_str)
        .map_err(|e| AppError::BadRequest(format!("Invalid URL: {}", e)))?;
    
    // Validate URL scheme
    match url.scheme() {
        "http" | "https" => {},
        _ => return Err(AppError::BadRequest("Only HTTP and HTTPS URLs are supported".to_string())),
    }
    
    // Validate hostname exists
    let hostname = url.host_str()
        .ok_or_else(|| AppError::BadRequest("URL must contain a valid hostname".to_string()))?;
    
    // Resolve hostname to IP addresses
    let addrs = tokio::net::lookup_host((hostname, url.port().unwrap_or(if url.scheme() == "https" { 443 } else { 80 })))
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to resolve hostname '{}': {}", hostname, e)))?;
    
    // Check if any resolved IP is safe
    let safe_ips: Vec<_> = addrs.filter_map(|addr| {
        let ip = addr.ip();
        if is_safe_ip(ip) {
            Some(ip)
        } else {
            None
        }
    }).collect();
    
    if safe_ips.is_empty() {
        return Err(AppError::BadRequest(format!(
            "URL '{}' resolves to private/internal IP addresses and is not allowed for security reasons", 
            hostname
        )));
    }
    
    // Make the HTTP request using the reusable client
    let response = HTTP_CLIENT.get(url_str)
        .send()
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to fetch image from URL: {}", e)))?;
    
    if !response.status().is_success() {
        return Err(AppError::BadRequest(format!("HTTP error when fetching image: {}", response.status())));
    }
    
    // Check content length
    let content_length = response.content_length().unwrap_or(0);
    let max_size = config.server.max_body_size.min(MAX_IMAGE_SIZE) as u64;
    if content_length > max_size {
        return Err(AppError::PayloadTooLarge(format!(
            "Image size {} exceeds limit of {} bytes",
            content_length, max_size
        )));
    }
    
    // Read response body with size limit
    let bytes = response.bytes().await
        .map_err(|e| AppError::BadRequest(format!("Failed to read image data: {}", e)))?;
    
    if bytes.len() > config.server.max_body_size.min(MAX_IMAGE_SIZE) {
        return Err(AppError::PayloadTooLarge(format!(
            "Image size {} exceeds limit of {} bytes",
            bytes.len(), 
            config.server.max_body_size.min(MAX_IMAGE_SIZE)
        )));
    }
    
    Ok(bytes.to_vec())
}

fn determine_output_format(operations_spec: &[PipelineOperationSpec], original_format: ImageFormat) -> ImageFormat {
    // Check the last convert operation to determine output format
    for spec in operations_spec.iter().rev() {
        if spec.operation == SupportedOperation::Convert {
            if let Ok(convert_params) = from_value::<FormatConversionParams>(spec.params.clone()) {
                match convert_params.format.to_lowercase().as_str() {
                    "png" => return ImageFormat::Png,
                    "jpeg" | "jpg" => return ImageFormat::Jpeg,
                    "gif" => return ImageFormat::Gif,
                    "webp" => return ImageFormat::WebP,
                    "bmp" => return ImageFormat::Bmp,
                    "tiff" | "tif" => return ImageFormat::Tiff,
                    _ => {
                        tracing::warn!("Unsupported format in convert operation: {}, using original format", convert_params.format);
                        return original_format;
                    }
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

    #[test]
    fn test_determine_output_format_with_convert() {
        let operations = vec![
            PipelineOperationSpec {
                operation: SupportedOperation::Resize,
                params: json!({"width": 100, "height": 100}),
                ignore_failure: false,
            },
            PipelineOperationSpec {
                operation: SupportedOperation::Convert,
                params: json!({"format": "jpeg", "quality": 85}),
                ignore_failure: false,
            },
        ];
        
        let result = determine_output_format(&operations, ImageFormat::Png);
        assert_eq!(result, ImageFormat::Jpeg);
    }

    #[test]
    fn test_determine_output_format_without_convert() {
        let operations = vec![
            PipelineOperationSpec {
                operation: SupportedOperation::Resize,
                params: json!({"width": 100, "height": 100}),
                ignore_failure: false,
            },
        ];
        
        let result = determine_output_format(&operations, ImageFormat::Png);
        assert_eq!(result, ImageFormat::Png);
    }

    #[test]
    fn test_determine_output_format_multiple_converts() {
        let operations = vec![
            PipelineOperationSpec {
                operation: SupportedOperation::Convert,
                params: json!({"format": "png"}),
                ignore_failure: false,
            },
            PipelineOperationSpec {
                operation: SupportedOperation::Resize,
                params: json!({"width": 100, "height": 100}),
                ignore_failure: false,
            },
            PipelineOperationSpec {
                operation: SupportedOperation::Convert,
                params: json!({"format": "webp"}),
                ignore_failure: false,
            },
        ];
        
        // Should use the last convert operation
        let result = determine_output_format(&operations, ImageFormat::Jpeg);
        assert_eq!(result, ImageFormat::WebP);
    }

    #[test]
    fn test_is_safe_ip_private_ranges() {
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
        use std::net::{IpAddr, Ipv4Addr};
        
        // Public IPv4 addresses should be allowed
        assert!(is_safe_ip(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)))); // Google DNS
        assert!(is_safe_ip(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)))); // Cloudflare DNS
        assert!(is_safe_ip(IpAddr::V4(Ipv4Addr::new(208, 67, 222, 222)))); // OpenDNS
    }

    #[test]
    fn test_is_safe_ip_ipv6() {
        use std::net::{IpAddr, Ipv6Addr};
        
        // IPv6 loopback should be rejected
        assert!(!is_safe_ip(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1))));
        
        // IPv6 link-local should be rejected
        assert!(!is_safe_ip(IpAddr::V6(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1))));
        
        // IPv6 unique local should be rejected
        assert!(!is_safe_ip(IpAddr::V6(Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 1))));
        
        // IPv6 documentation should be rejected
        assert!(!is_safe_ip(IpAddr::V6(Ipv6Addr::new(0x2001, 0x0db8, 0, 0, 0, 0, 0, 1))));
        
        // Public IPv6 should be allowed (Google DNS)
        assert!(is_safe_ip(IpAddr::V6(Ipv6Addr::new(0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8888))));
    }
}