use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Internal Server Error")]
    InternalServerError,
    #[error("Bad Request")]
    BadRequest,
    #[error("Image Processing Error: {0}")]
    ImageProcessingError(String),
    #[error("Unsupported Media Type")]
    UnsupportedMediaType,
    #[error("Payload Too Large")]
    PayloadTooLarge,
    #[error("Rate Limit Exceeded")]
    RateLimitExceeded,
    #[error("Invalid Operation: {0}")]
    InvalidOperation(String),
    #[error("File System Error: {0}")]
    FileSystemError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string(),
            ),
            AppError::BadRequest => (
                StatusCode::BAD_REQUEST,
                "Bad Request".to_string(),
            ),
            AppError::ImageProcessingError(msg) => (
                StatusCode::BAD_REQUEST,
                msg,
            ),
            AppError::UnsupportedMediaType => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "Unsupported media type".to_string(),
            ),
            AppError::PayloadTooLarge => (
                StatusCode::PAYLOAD_TOO_LARGE,
                "File size too large".to_string(),
            ),
            AppError::RateLimitExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                "Rate limit exceeded".to_string(),
            ),
            AppError::InvalidOperation(msg) => (
                StatusCode::BAD_REQUEST,
                format!("Invalid operation: {}", msg),
            ),
            AppError::FileSystemError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("File system error: {}", msg),
            ),
        };

        let body = Json(json!({
            "error": error_message,
            "code": status.as_u16(),
            "status": "error"
        }));
        
        (status, body).into_response()
    }
}