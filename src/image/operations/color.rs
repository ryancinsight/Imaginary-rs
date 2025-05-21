//! Color and filter operations for images.
//!
//! This module provides functions for grayscale conversion, brightness/contrast adjustment, sharpening, and blurring.

use image::DynamicImage;
use crate::image::params::{BlurParams};
use image::GenericImageView;

/// Convert the image to grayscale.
///
/// # Arguments
/// * `image` - The input image to convert.
///
/// # Returns
/// A new `DynamicImage` in grayscale.
///
/// # Examples
/// # use image::DynamicImage;
/// # let img = DynamicImage::new_rgb8(100, 100);
/// let gray = grayscale(img);
pub fn grayscale(image: DynamicImage) -> DynamicImage {
    image.to_luma8().into()
}

/// Adjust the brightness of the image by the given value.
///
/// # Arguments
/// * `image` - The input image to adjust.
/// * `value` - The brightness adjustment value (positive or negative).
///
/// # Returns
/// A new `DynamicImage` with adjusted brightness.
pub fn adjust_brightness(image: DynamicImage, value: i32) -> DynamicImage {
    image.brighten(value)
}

/// Adjust the contrast of the image by the given value.
///
/// # Arguments
/// * `image` - The input image to adjust.
/// * `value` - The contrast adjustment value (positive or negative).
///
/// # Returns
/// A new `DynamicImage` with adjusted contrast.
pub fn adjust_contrast(image: DynamicImage, value: f32) -> DynamicImage {
    image.adjust_contrast(value)
}

/// Sharpen the image using a fixed 3x3 kernel.
///
/// # Arguments
/// * `image` - The input image to sharpen.
///
/// # Returns
/// A new `DynamicImage` sharpened using a 3x3 kernel.
pub fn sharpen(image: DynamicImage) -> DynamicImage {
    let sharpen_kernel: [f32; 9] = [-1.0, -1.0, -1.0,
                                    -1.0,  9.0, -1.0,
                                    -1.0, -1.0, -1.0];
    image.filter3x3(&sharpen_kernel)
}

/// Blur the image using the specified sigma value.
///
/// # Arguments
/// * `image` - The input image to blur.
/// * `params` - The blur parameters (sigma, minampl).
///
/// # Returns
/// A new `DynamicImage` blurred by the specified sigma.
pub fn blur(image: DynamicImage, params: &BlurParams) -> DynamicImage {
    if params.minampl.is_some() {
        tracing::warn!("Blur operation: 'minampl' parameter is provided but not currently used by the image crate's basic blur. Only sigma is applied.");
    }
    image.blur(params.sigma)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, ImageBuffer, Rgba};
    use crate::image::params::BlurParams;

    fn create_test_image(width: u32, height: u32) -> DynamicImage {
        DynamicImage::ImageRgba8(ImageBuffer::from_pixel(
            width,
            height,
            Rgba([128u8, 128u8, 128u8, 255u8]),
        ))
    }

    #[test]
    fn test_grayscale() {
        let img = create_test_image(100, 100);
        let gray = grayscale(img);
        assert_eq!(gray.dimensions(), (100, 100));
    }

    #[test]
    fn test_adjust_brightness() {
        let img = create_test_image(100, 100);
        let bright = adjust_brightness(img, 20);
        assert_eq!(bright.dimensions(), (100, 100));
    }

    #[test]
    fn test_adjust_contrast() {
        let img = create_test_image(100, 100);
        let contrast = adjust_contrast(img, 1.5);
        assert_eq!(contrast.dimensions(), (100, 100));
    }

    #[test]
    fn test_sharpen() {
        let img = create_test_image(100, 100);
        let sharp = sharpen(img);
        assert_eq!(sharp.dimensions(), (100, 100));
    }

    #[test]
    fn test_blur() {
        let img = create_test_image(100, 100);
        let params = BlurParams { sigma: 2.0, minampl: None };
        let blurred = blur(img, &params);
        assert_eq!(blurred.dimensions(), (100, 100));
    }
} 