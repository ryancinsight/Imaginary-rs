use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::fmt;
use tracing::info;

#[derive(Debug)]
pub enum AppInfo {
    #[allow(dead_code)]
    SecurityConfigSecure,
    #[allow(dead_code)]
    SecurityConfigNotSecure,
    #[allow(dead_code)]
    OriginAllowed(String),
    #[allow(dead_code)]
    OriginNotAllowed(String),
    GeneratedSignature(String),
    ValidatedSignature(bool),
    ExpectedApiKey(String),
}

#[derive(Debug)]
pub enum ImageInfo {
    #[allow(dead_code)]
    ImageProcessedSuccessfully(String),
    #[allow(dead_code)]
    ImageConvertedSuccessfully(String),
}

impl fmt::Display for AppInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let info_message = match self {
            AppInfo::SecurityConfigSecure => "Security configuration is secure.".to_string(),
            AppInfo::SecurityConfigNotSecure => "Security configuration is not secure.".to_string(),
            AppInfo::OriginAllowed(origin) => format!("Origin {} is allowed.", origin),
            AppInfo::OriginNotAllowed(origin) => format!("Origin {} is not allowed.", origin),
            AppInfo::GeneratedSignature(signature) => format!("Generated signature: {}", signature),
            AppInfo::ValidatedSignature(is_valid) => format!("Signature is valid: {}", is_valid),
            AppInfo::ExpectedApiKey(api_key) => format!("Expected API key: '{}'", api_key),
        };
        write!(f, "{}", info_message)
    }
}

impl fmt::Display for ImageInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let info_message = match self {
            ImageInfo::ImageProcessedSuccessfully(output_path) => {
                format!("Image processed successfully: {}", output_path)
            }
            ImageInfo::ImageConvertedSuccessfully(output_path) => {
                format!("Image converted successfully: {}", output_path)
            }
        };
        write!(f, "{}", info_message)
    }
}

impl IntoResponse for AppInfo {
    fn into_response(self) -> Response {
        let (status, info_message) = match &self {
            AppInfo::SecurityConfigSecure => (
                StatusCode::OK,
                "Security configuration is secure.".to_string(),
            ),
            AppInfo::SecurityConfigNotSecure => (
                StatusCode::OK,
                "Security configuration is not secure.".to_string(),
            ),
            AppInfo::OriginAllowed(origin) => {
                (StatusCode::OK, format!("Origin {} is allowed.", origin))
            }
            AppInfo::OriginNotAllowed(origin) => {
                (StatusCode::OK, format!("Origin {} is not allowed.", origin))
            }
            AppInfo::GeneratedSignature(signature) => (
                StatusCode::OK,
                format!("Generated signature: {}", signature),
            ),
            AppInfo::ValidatedSignature(is_valid) => {
                (StatusCode::OK, format!("Signature is valid: {}", is_valid))
            }
            AppInfo::ExpectedApiKey(api_key) => {
                (StatusCode::OK, format!("Expected API key: '{}'", api_key))
            }
        };

        // Log the info message
        info!("Info: {}", info_message);

        let body = Json(json!({
            "info": info_message,
            "code": status.as_u16(),
            "status": "success"
        }));

        (status, body).into_response()
    }
}

impl IntoResponse for ImageInfo {
    fn into_response(self) -> Response {
        let (status, info_message) = match &self {
            ImageInfo::ImageProcessedSuccessfully(output_path) => (
                StatusCode::OK,
                format!("Image processed successfully: {}", output_path),
            ),
            ImageInfo::ImageConvertedSuccessfully(output_path) => (
                StatusCode::OK,
                format!("Image converted successfully: {}", output_path),
            ),
        };

        // Log the info message
        info!("Info: {}", info_message);

        let body = Json(json!({
            "info": info_message,
            "code": status.as_u16(),
            "status": "success"
        }));

        (status, body).into_response()
    }
}
