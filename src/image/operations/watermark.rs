//! Watermarking operations for images.
//!
//! This module provides functions to apply text or image watermarks to images as part of the processing pipeline.

use image::{DynamicImage, Rgba};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};
use crate::image::params::{WatermarkParams, WatermarkImageParams, WatermarkPosition};

/// Applies a text watermark to the image with the specified parameters.
/// Supports automatic positioning or exact coordinates, opacity, and font customization.
///
/// # Arguments
/// * `image` - The input image to watermark.
/// * `params` - The watermark parameters (text, opacity, position, font size, color, x, y).
///
/// # Returns
/// A new `DynamicImage` with the watermark applied, or an error if the font cannot be loaded.
///
/// # Examples
/// # use image::DynamicImage;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let img = DynamicImage::new_rgb8(100, 100);
/// let wm_params = WatermarkParams {
///     text: "Sample".to_string(),
///     opacity: 0.5,
///     position: WatermarkPosition::BottomRight,
///     font_size: 24,
///     color: [255, 255, 255],
///     x: None,
///     y: None,
/// };
/// let watermarked = watermark(&img, &wm_params)?;
/// # Ok(())
/// # }
pub fn watermark(image: &DynamicImage, params: &WatermarkParams) -> Result<DynamicImage, String> {
    // Always operate on RGBA8
    let mut rgba_image = image.to_rgba8();
    // Load the font data from a byte array
    let font_data = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/fonts/DejaVuSans.ttf"));
    let font = Font::try_from_bytes(font_data)
        .ok_or_else(|| "Failed to load font".to_string())?;

    let scale = Scale::uniform(params.font_size as f32);
    let color = Rgba([
        params.color[0],
        params.color[1],
        params.color[2],
        (params.opacity * 255.0) as u8,
    ]);

    let (x, y) = match (params.x, params.y) {
        (Some(x), Some(y)) => (x, y),
        _ => {
            let (width, height) = rgba_image.dimensions();
            match params.position {
                WatermarkPosition::TopLeft => (10, 10),
                WatermarkPosition::TopRight => (width - 10, 10),
                WatermarkPosition::BottomLeft => (10, height - 10),
                WatermarkPosition::BottomRight => (width - 10, height - 10),
                WatermarkPosition::Center => (width / 2, height / 2),
            }
        }
    };

    draw_text_mut(
        &mut rgba_image,
        color,
        x as i32,
        y as i32,
        scale,
        &font,
        &params.text
    );

    Ok(DynamicImage::ImageRgba8(rgba_image))
}

/// Apply a watermark image to the image. (Not yet implemented)
///
/// # Arguments
/// * `image` - The input image to watermark.
/// * `params` - The watermark image parameters.
///
/// # Returns
/// The input `DynamicImage` (no-op).
pub(crate) fn watermark_image(image: DynamicImage, _params: &WatermarkImageParams) -> DynamicImage {
    image
} 