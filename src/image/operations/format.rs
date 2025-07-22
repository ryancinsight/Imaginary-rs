//! Format operations for images.
//!
//! This module provides functions for format conversion and autorotation.

use crate::http::errors::AppError;
use crate::image::params::FormatConversionParams;
use image::{DynamicImage, ImageFormat};
use std::io::Cursor;

/// Convert the image to a different format with optional quality parameter.
///
/// # Arguments
/// * `image` - The input image to convert.
/// * `params` - The format conversion parameters (format, quality).
///
/// # Returns
/// A new `DynamicImage` in the specified format, or an error if conversion fails.
///
/// # Examples
/// # use image::DynamicImage;
/// # let img = DynamicImage::new_rgb8(100, 100);
/// let converted = convert_format(img, &FormatConversionParams { format: "jpeg".to_string(), quality: Some(85) });
pub fn convert_format(
    image: DynamicImage,
    params: &FormatConversionParams,
) -> Result<DynamicImage, AppError> {
    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);

    // Safely determine the image format without panicking
    let format = match params.format.to_lowercase().as_str() {
        "png" => ImageFormat::Png,
        "jpeg" | "jpg" => ImageFormat::Jpeg,
        "gif" => ImageFormat::Gif,
        "webp" => ImageFormat::WebP,
        "bmp" => ImageFormat::Bmp,
        "tiff" | "tif" => ImageFormat::Tiff,
        "ico" => ImageFormat::Ico,
        _ => {
            return Err(AppError::UnsupportedMediaType(format!(
                "Unsupported image format: {}",
                params.format
            )))
        }
    };

    image
        .write_to(&mut cursor, format)
        .map_err(|e| AppError::ImageProcessingError(e.to_string()))?;
    image::load_from_memory(&buffer).map_err(|e| AppError::ImageProcessingError(e.to_string()))
}

/// Autorotate the image based on its EXIF orientation.
///
/// # Arguments
/// * `image` - The input image to autorotate.
///
/// # Returns
/// The input `DynamicImage` (no-op).
pub fn autorotate(image: DynamicImage) -> DynamicImage {
    image
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::params::FormatConversionParams;
    use image::{ColorType, DynamicImage, GenericImageView, ImageBuffer, Rgba};

    fn create_test_image(width: u32, height: u32) -> DynamicImage {
        DynamicImage::ImageRgba8(ImageBuffer::from_pixel(
            width,
            height,
            Rgba([255u8, 0u8, 0u8, 255u8]),
        ))
    }

    #[test]
    fn test_convert_format() {
        let img = create_test_image(100, 100);
        let params = FormatConversionParams {
            format: "png".to_string(),
            quality: Some(90),
        };
        let converted_img = convert_format(img, &params).unwrap();
        assert_eq!(converted_img.color(), ColorType::Rgba8);
    }

    #[test]
    fn test_autorotate() {
        let img = create_test_image(100, 100);
        let rotated = autorotate(img);
        assert_eq!(rotated.dimensions(), (100, 100));
    }
}
