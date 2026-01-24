//! Overlay operations for images.
//!
//! This module provides functions for overlaying images.

use crate::http::errors::AppError;
use image::{DynamicImage, GenericImage};

/// Overlay one image on top of another at the given coordinates.
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
#[allow(dead_code)]
pub(crate) fn overlay(
    mut image: DynamicImage,
    overlay_image: DynamicImage,
    x: u32,
    y: u32,
) -> Result<DynamicImage, AppError> {
    image
        .copy_from(&overlay_image, x, y)
        .map_err(|e| AppError::ImageProcessingError(e.to_string()))?;
    Ok(image)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};

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
}
