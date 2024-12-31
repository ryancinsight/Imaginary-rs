use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;
use tracing::error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Internal Server Error: {0}")]
    InternalServerError(String),
    #[error("Bad Request: {0}")]
    BadRequest(String),
    #[error("Image Processing Error: {0}")]
    ImageProcessingError(String),
    #[error("Unsupported Media Type: {0}")]
    UnsupportedMediaType(String),
    #[error("Payload Too Large: {0}")]
    PayloadTooLarge(String),
    #[error("Rate Limit Exceeded: {0}")]
    RateLimitExceeded(String),
    #[error("Invalid Operation: {0}")]
    InvalidOperation(String),
    #[error("File System Error: {0}")]
    FileSystemError(String),
    #[error("Multipart Error: {0}")]
    MultipartError(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
}

#[derive(Error, Debug)]
pub enum ImageError {
    #[error("Invalid dimensions: {0}")]
    InvalidDimensions(String),
    
    #[error("Invalid degrees: {0}")]
    InvalidDegrees(String),
    
    #[error("Invalid opacity: {0}")]
    InvalidOpacity(String),
    
    #[error("Invalid quality: {0}")]
    InvalidQuality(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            AppError::InternalServerError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Internal Server Error: {}", msg),
            ),
            AppError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                format!("Bad Request: {}", msg),
            ),
            AppError::ImageProcessingError(msg) => (
                StatusCode::BAD_REQUEST,
                format!("Image Processing Error: {}", msg),
            ),
            AppError::UnsupportedMediaType(msg) => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                format!("Unsupported Media Type: {}", msg),
            ),
            AppError::PayloadTooLarge(msg) => (
                StatusCode::PAYLOAD_TOO_LARGE,
                format!("Payload Too Large: {}", msg),
            ),
            AppError::RateLimitExceeded(msg) => (
                StatusCode::TOO_MANY_REQUESTS,
                format!("Rate Limit Exceeded: {}", msg),
            ),
            AppError::InvalidOperation(msg) => (
                StatusCode::BAD_REQUEST,
                format!("Invalid Operation: {}", msg),
            ),
            AppError::FileSystemError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("File System Error: {}", msg),
            ),
            AppError::MultipartError(msg) => (
                StatusCode::BAD_REQUEST,
                format!("Multipart Error: {}", msg),
            ),
            AppError::Unauthorized(msg) => (
                StatusCode::UNAUTHORIZED,
                format!("Unauthorized: {}", msg),
            ),
        };

        // Log the error
        error!("Error occurred: {}", error_message);

        let body = Json(json!({
            "error": error_message,
            "code": status.as_u16(),
            "status": "error"
        }));
        
        (status, body).into_response()
    }
}

impl IntoResponse for ImageError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            ImageError::InvalidDimensions(msg) => (
                StatusCode::BAD_REQUEST,
                format!("Invalid dimensions: {}", msg),
            ),
            ImageError::InvalidDegrees(msg) => (
                StatusCode::BAD_REQUEST,
                format!("Invalid degrees: {}", msg),
            ),
            ImageError::InvalidOpacity(msg) => (
                StatusCode::BAD_REQUEST,
                format!("Invalid opacity: {}", msg),
            ),
            ImageError::InvalidQuality(msg) => (
                StatusCode::BAD_REQUEST,
                format!("Invalid quality: {}", msg),
            ),
        };

        // Log the error
        error!("Image error occurred: {}", error_message);

        let body = Json(json!({
            "error": error_message,
            "code": status.as_u16(),
            "status": "error"
        }));
        
        (status, body).into_response()
    }
}