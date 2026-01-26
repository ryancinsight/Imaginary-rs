use axum::{
    extract::{Multipart, Query, State},
    http::Method,
    response::IntoResponse,
    Json,
};
use image::GenericImageView;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

use crate::{
    config::Config,
    http::{
        errors::AppError,
        request_utils::{fetch_image_from_url, MAX_IMAGE_SIZE},
    },
};

#[derive(Deserialize, Archive, RkyvDeserialize, RkyvSerialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct InfoQuery {
    url: Option<String>,
}

#[derive(Serialize, Archive, RkyvDeserialize, RkyvSerialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct ImageInfo {
    pub width: u32,
    pub height: u32,
    #[serde(rename = "type")]
    pub format: String,
    pub space: String,
    pub channels: u8,
    pub depth: u8,
    pub has_alpha: bool,
}

pub async fn get_info(
    method: Method,
    State(config): State<Arc<Config>>,
    query: Option<Query<InfoQuery>>,
    multipart: Option<Multipart>,
) -> Result<impl IntoResponse, AppError> {
    let image_bytes = match method {
        Method::GET => {
            let Query(params) = query
                .ok_or_else(|| AppError::BadRequest("Missing query parameters".to_string()))?;
            let url = params
                .url
                .ok_or_else(|| AppError::BadRequest("Missing 'url' parameter".to_string()))?;
            fetch_image_from_url(&url, &config).await?
        }
        Method::POST => {
            let mut multipart = multipart
                .ok_or_else(|| AppError::BadRequest("Missing multipart data".to_string()))?;
            let mut image_data = None;

            while let Some(field) = multipart
                .next_field()
                .await
                .map_err(|e| AppError::MultipartError(e.to_string()))?
            {
                let name = field.name().unwrap_or("").to_string();
                if name == "image" || name == "file" {
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
                    break; // Found image, stop parsing
                }
            }
            image_data
                .ok_or_else(|| AppError::BadRequest("Missing image data".to_string()))?
        }
        _ => return Err(AppError::BadRequest("Method not allowed".to_string())),
    };

    let (image, format) = tokio::task::spawn_blocking(move || {
        let format = image::guess_format(&image_bytes)
            .map_err(|_| AppError::UnsupportedMediaType("Unknown format".to_string()))?;
        let image = image::load_from_memory_with_format(&image_bytes, format)
            .map_err(|e| AppError::ImageProcessingError(format!("Failed to load image: {}", e)))?;
        Ok::<(image::DynamicImage, image::ImageFormat), AppError>((image, format))
    })
    .await
    .map_err(|e| AppError::InternalServerError(format!("Task join error: {}", e)))??;

    let (width, height) = image.dimensions();
    let color_type = image.color();

    let (channels, depth, has_alpha, space) = match color_type {
        image::ColorType::L8 => (1, 8, false, "gray"),
        image::ColorType::La8 => (2, 8, true, "gray"),
        image::ColorType::Rgb8 => (3, 8, false, "srgb"),
        image::ColorType::Rgba8 => (4, 8, true, "srgb"),
        image::ColorType::L16 => (1, 16, false, "gray"),
        image::ColorType::La16 => (2, 16, true, "gray"),
        image::ColorType::Rgb16 => (3, 16, false, "srgb"),
        image::ColorType::Rgba16 => (4, 16, true, "srgb"),
        image::ColorType::Rgb32F => (3, 32, false, "srgb"),
        image::ColorType::Rgba32F => (4, 32, true, "srgb"),
        _ => (0, 0, false, "unknown"),
    };

    let info = ImageInfo {
        width,
        height,
        format: format!("{:?}", format).to_lowercase(),
        space: space.to_string(),
        channels,
        depth,
        has_alpha,
    };

    Ok(Json(info))
}
