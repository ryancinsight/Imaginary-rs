use std::sync::Arc;

use axum::body::Bytes;
use axum::{
    extract::{Multipart, Query, State},
    http::Method,
    response::{IntoResponse, Response},
    Json,
};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};

use crate::{
    config::Config,
    http::errors::AppError,
    http::request_utils::{fetch_image_from_url, MAX_IMAGE_SIZE},
    image::operations::palette::extract_palette,
};

#[derive(Deserialize, Archive, RkyvDeserialize, RkyvSerialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct PaletteQuery {
    url: Option<String>,
    #[serde(default = "default_max_colors")]
    max_colors: u8,
}

fn default_max_colors() -> u8 {
    5
}

#[derive(Serialize, Archive, RkyvDeserialize, RkyvSerialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct PaletteResponse {
    colors: Vec<[u8; 3]>,
}

pub async fn get_palette(
    method: Method,
    State(config): State<Arc<Config>>,
    query: Option<Query<PaletteQuery>>,
    multipart: Option<Multipart>,
) -> Result<Response, AppError> {
    let (image_bytes, max_colors) = match method {
        Method::GET => {
            let Query(params) = query
                .ok_or_else(|| AppError::BadRequest("Missing query parameters".to_string()))?;
            let url = params
                .url
                .ok_or_else(|| AppError::BadRequest("Missing 'url' parameter".to_string()))?;
            let bytes = fetch_image_from_url(&url, &config).await?;
            (bytes, params.max_colors)
        }
        Method::POST => {
            let mut multipart = multipart
                .ok_or_else(|| AppError::BadRequest("Missing multipart data".to_string()))?;
            let mut image_data: Option<Bytes> = None;
            let mut max_colors = 5;

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
                    "max_colors" => {
                        if let Ok(val) = field.text().await.unwrap_or_default().parse::<u8>() {
                            max_colors = val;
                        }
                    }
                    _ => {}
                }
            }
            let bytes =
                image_data.ok_or_else(|| AppError::BadRequest("Missing image data".to_string()))?;
            (bytes, max_colors)
        }
        _ => return Err(AppError::BadRequest("Method not allowed".to_string())),
    };

    let colors = tokio::task::spawn_blocking(move || {
        let image = image::load_from_memory(&image_bytes)
            .map_err(|e| AppError::ImageProcessingError(format!("Failed to load image: {}", e)))?;
        extract_palette(&image, max_colors)
    })
    .await
    .map_err(|e| AppError::InternalServerError(format!("Task failed: {}", e)))??;

    Ok(Json(PaletteResponse { colors }).into_response())
}
