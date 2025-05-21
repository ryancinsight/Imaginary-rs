//! Overlay operations for images.
//!
//! This module provides functions for overlaying images and drawing text.

use crate::http::errors::AppError;
use image::{DynamicImage, GenericImage, GenericImageView};

/// Overlay one image on top of another at the specified (x, y) position.
///
/// # Arguments
/// * `image` - The base image to overlay onto.
/// * `overlay_image` - The image to overlay.
/// * `x` - The x-coordinate for the overlay position.
/// * `y` - The y-coordinate for the overlay position.
///
/// # Returns
/// A new `DynamicImage` with the overlay applied, or an error if the operation fails.
///
/// # Examples
/// # use image::DynamicImage;
/// # let base = DynamicImage::new_rgb8(100, 100);
/// # let overlay_img = DynamicImage::new_rgb8(50, 50);
/// let result = overlay(base, overlay_img, 10, 10).unwrap();
pub(crate) fn overlay(image: DynamicImage, overlay_image: DynamicImage, x: u32, y: u32) -> Result<DynamicImage, AppError> {
    let mut img = image.clone();
    img.copy_from(&overlay_image, x, y).map_err(|e| AppError::ImageProcessingError(e.to_string()))?;
    Ok(img)
}

/// Draw text on the image at the specified position and font size (placeholder).
///
/// # Arguments
/// * `image` - The input image to draw text on.
/// * `text` - The text to draw.
/// * `x` - The x-coordinate for the text position.
/// * `y` - The y-coordinate for the text position.
/// * `font_size` - The font size for the text.
///
/// # Returns
/// A new `DynamicImage` with the text drawn (currently a no-op).
pub(crate) fn draw_text(image: DynamicImage, _text: &str, _x: u32, _y: u32, _font_size: u32) -> DynamicImage {
    image
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, ImageBuffer, Rgba};

    fn create_test_image(width: u32, height: u32) -> DynamicImage {
        DynamicImage::ImageRgba8(ImageBuffer::from_pixel(
            width,
            height,
            Rgba([0u8, 0u8, 255u8, 255u8]),
        ))
    }

    #[test]
    fn test_overlay() {
        let img1 = create_test_image(100, 100);
        let img2 = create_test_image(50, 50);
        let result = overlay(img1.clone(), img2, 25, 25);
        assert!(result.is_ok());
        let overlaid_img = result.unwrap();
        assert_eq!(overlaid_img.dimensions(), (100, 100));
    }

    #[test]
    fn test_draw_text() {
        let img = create_test_image(100, 100);
        let result = draw_text(img, "Hello", 10, 10, 12);
        assert_eq!(result.dimensions(), (100, 100));
    }
} 