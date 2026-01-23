//! Color and filter operations for images.
//!
//! This module provides functions for grayscale conversion, brightness/contrast adjustment, sharpening, and blurring.

use crate::image::params::{BlurParams, GammaParams};
use image::DynamicImage;

/// Convert an image to grayscale.
pub fn grayscale(image: DynamicImage) -> DynamicImage {
    image.to_luma8().into()
}

/// Adjust the brightness of an image by the given value.
pub fn adjust_brightness(image: DynamicImage, value: i32) -> DynamicImage {
    image.brighten(value)
}

/// Adjust the contrast of an image by the given value.
pub fn adjust_contrast(image: DynamicImage, value: f32) -> DynamicImage {
    image.adjust_contrast(value)
}

/// Sharpen the image using a simple kernel.
pub fn sharpen(image: DynamicImage) -> DynamicImage {
    let sharpen_kernel: [f32; 9] = [-1.0, -1.0, -1.0, -1.0, 9.0, -1.0, -1.0, -1.0, -1.0];
    image.filter3x3(&sharpen_kernel)
}

/// Apply a Gaussian blur to the image with the given parameters.
pub fn blur(image: DynamicImage, params: &BlurParams) -> DynamicImage {
    if params.minampl.is_some() {
        tracing::warn!("Blur operation: 'minampl' parameter is provided but not currently used by the image crate's basic blur. Only sigma is applied.");
    }
    image.blur(params.sigma)
}

/// Apply gamma correction to the image.
pub fn gamma(image: DynamicImage, params: &GammaParams) -> DynamicImage {
    let value = params.value;
    let mut lut = [0u8; 256];
    for i in 0..256 {
        let normalized = (i as f32) / 255.0;
        let corrected = normalized.powf(1.0 / value);
        lut[i] = (corrected * 255.0).round().clamp(0.0, 255.0) as u8;
    }

    // Convert to RGBA8 to handle all formats uniformly and enable in-place modification
    let mut rgba = image.into_rgba8();
    for pixel in rgba.pixels_mut() {
        pixel[0] = lut[pixel[0] as usize];
        pixel[1] = lut[pixel[1] as usize];
        pixel[2] = lut[pixel[2] as usize];
        // Alpha channel is preserved
    }
    DynamicImage::ImageRgba8(rgba)
}

/// Invert the colors of the image.
pub fn negate(mut image: DynamicImage) -> DynamicImage {
    image.invert();
    image
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::params::{BlurParams, GammaParams};
    use image::GenericImageView;
    use image::{DynamicImage, ImageBuffer, Rgba};

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
        let params = BlurParams {
            sigma: 2.0,
            minampl: None,
        };
        let blurred = blur(img, &params);
        assert_eq!(blurred.dimensions(), (100, 100));
    }

    #[test]
    fn test_gamma() {
        let img = create_test_image(100, 100);
        let params = GammaParams { value: 2.2 };
        let corrected = gamma(img, &params);
        assert_eq!(corrected.dimensions(), (100, 100));
        // Check pixel value change
        let p = corrected.get_pixel(0, 0);
        // Original 128. 128/255 = 0.502. 0.502^(1/2.2) = 0.73. 0.73*255 = 186.
        assert!(p[0] > 128);
    }

    #[test]
    fn test_negate() {
        let img = create_test_image(100, 100);
        let inverted = negate(img);
        assert_eq!(inverted.dimensions(), (100, 100));
        let p = inverted.get_pixel(0, 0);
        assert_eq!(p[0], 255 - 128);
    }
}
