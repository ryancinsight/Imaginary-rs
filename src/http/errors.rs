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
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}