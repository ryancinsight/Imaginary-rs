//! Watermarking operations for images.
//!
//! This module provides functions to apply text or image watermarks to images as part of the processing pipeline.

use crate::config::Config;
use crate::http::errors::AppError;
use crate::http::request_utils::fetch_image_from_url;
use crate::image::params::{WatermarkImageParams, WatermarkParams, WatermarkPosition};
use ab_glyph::{FontRef, PxScale};
use image::{DynamicImage, GenericImageView, Rgba};
use imageproc::drawing::{draw_text_mut, text_size};

/// Applies a text watermark to the image with the specified parameters.
/// Supports automatic positioning or exact coordinates, opacity, and font customization.
///
/// # Arguments
/// * `image` - The input image to watermark.
/// * `params` - The watermark parameters (text, opacity, position, font size, color, x, y).
///
/// # Returns
/// A new `DynamicImage` with the watermark applied, or an error if the font cannot be loaded (returning original image).
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
/// let watermarked = watermark(img, &wm_params).map_err(|e| e.1)?;
/// # Ok(())
/// # }
pub fn watermark(
    image: DynamicImage,
    params: &WatermarkParams,
) -> Result<DynamicImage, (DynamicImage, String)> {
    // Load the font data from a byte array
    let font_data = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/fonts/DejaVuSans.ttf"
    ));
    let font = match FontRef::try_from_slice(font_data) {
        Ok(f) => f,
        Err(_) => return Err((image, "Failed to load font".to_string())),
    };

    // Always operate on RGBA8
    let mut rgba_image = image.into_rgba8();

    let scale = PxScale::from(params.font_size as f32);
    let color = Rgba([
        params.color[0],
        params.color[1],
        params.color[2],
        (params.opacity * 255.0) as u8,
    ]);

    // Measure text width/height
    let (glyphs_width, glyphs_height) = text_size(scale, &font, &params.text);

    let margin = 10u32;
    let (width, height) = rgba_image.dimensions();

    let (x, y) = match (params.x, params.y) {
        (Some(x), Some(y)) => (x, y),
        _ => match params.position {
            WatermarkPosition::TopLeft => (margin, margin),
            WatermarkPosition::TopRight => (width.saturating_sub(glyphs_width + margin), margin),
            WatermarkPosition::BottomLeft => {
                (margin, height.saturating_sub(glyphs_height + margin))
            }
            WatermarkPosition::BottomRight => (
                width.saturating_sub(glyphs_width + margin),
                height.saturating_sub(glyphs_height + margin),
            ),
            WatermarkPosition::Center => (
                (width.saturating_sub(glyphs_width)) / 2,
                (height.saturating_sub(glyphs_height)) / 2,
            ),
        },
    };

    draw_text_mut(
        &mut rgba_image,
        color,
        x as i32,
        y as i32,
        scale,
        &font,
        &params.text,
    );

    Ok(DynamicImage::ImageRgba8(rgba_image))
}

/// Overlays a watermark image onto the base image at the specified position and opacity.
pub fn watermark_image(
    image: DynamicImage,
    params: &WatermarkImageParams,
    config: &Config,
    handle: &tokio::runtime::Handle,
) -> Result<DynamicImage, (DynamicImage, AppError)> {
    let url = &params.watermark_url;

    // Block on async fetch using the provided runtime handle
    let bytes = match handle.block_on(fetch_image_from_url(url, config)) {
        Ok(b) => b,
        Err(e) => return Err((image, e)),
    };

    let watermark_img = match image::load_from_memory(&bytes) {
        Ok(img) => img,
        Err(e) => {
            return Err((
                image,
                AppError::ImageProcessingError(format!("Failed to load watermark: {}", e)),
            ))
        }
    };

    Ok(apply_watermark(image, watermark_img, params))
}

fn apply_watermark(
    mut image: DynamicImage,
    watermark: DynamicImage,
    params: &WatermarkImageParams,
) -> DynamicImage {
    let mut watermark = watermark.into_rgba8();

    // Scale watermark if needed
    if let Some(scale) = params.scale {
        let target_width = (image.width() as f32 * scale) as u32;
        if target_width > 0 {
            let w = DynamicImage::ImageRgba8(watermark);
            watermark = w
                .resize(
                    target_width,
                    u32::MAX,
                    image::imageops::FilterType::Lanczos3,
                )
                .into_rgba8();
        }
    }

    // Apply opacity
    if params.opacity < 1.0 {
        for pixel in watermark.pixels_mut() {
            pixel[3] = (pixel[3] as f32 * params.opacity) as u8;
        }
    }

    // Calculate position
    let (width, height) = image.dimensions();
    let (w_width, w_height) = watermark.dimensions();

    let (mut x, mut y) = match params.position {
        WatermarkPosition::TopLeft => (0, 0),
        WatermarkPosition::TopRight => (width.saturating_sub(w_width), 0),
        WatermarkPosition::BottomLeft => (0, height.saturating_sub(w_height)),
        WatermarkPosition::BottomRight => (
            width.saturating_sub(w_width),
            height.saturating_sub(w_height),
        ),
        WatermarkPosition::Center => (
            (width.saturating_sub(w_width)) / 2,
            (height.saturating_sub(w_height)) / 2,
        ),
    };

    // Apply offsets
    if let Some(offset_x) = params.x_offset {
        x = (x as i64 + offset_x as i64).max(0) as u32;
    }
    if let Some(offset_y) = params.y_offset {
        y = (y as i64 + offset_y as i64).max(0) as u32;
    }

    image::imageops::overlay(&mut image, &watermark, x as i64, y as i64);
    image
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::params::{WatermarkParams, WatermarkPosition};
    use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};

    fn create_test_image(width: u32, height: u32) -> DynamicImage {
        DynamicImage::ImageRgba8(ImageBuffer::from_pixel(
            width,
            height,
            Rgba([0u8, 0u8, 0u8, 255u8]),
        ))
    }

    #[test]
    fn test_watermark_top_left() {
        let img = create_test_image(200, 100);
        let params = WatermarkParams {
            text: "TL".to_string(),
            opacity: 1.0,
            position: WatermarkPosition::TopLeft,
            font_size: 24,
            color: [255, 255, 255],
            x: None,
            y: None,
        };
        let result = watermark(img, &params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_watermark_top_right() {
        let img = create_test_image(200, 100);
        let params = WatermarkParams {
            text: "TR".to_string(),
            opacity: 1.0,
            position: WatermarkPosition::TopRight,
            font_size: 24,
            color: [255, 255, 255],
            x: None,
            y: None,
        };
        let result = watermark(img, &params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_watermark_bottom_left() {
        let img = create_test_image(200, 100);
        let params = WatermarkParams {
            text: "BL".to_string(),
            opacity: 1.0,
            position: WatermarkPosition::BottomLeft,
            font_size: 24,
            color: [255, 255, 255],
            x: None,
            y: None,
        };
        let result = watermark(img, &params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_watermark_bottom_right() {
        let img = create_test_image(200, 100);
        let params = WatermarkParams {
            text: "BR".to_string(),
            opacity: 1.0,
            position: WatermarkPosition::BottomRight,
            font_size: 24,
            color: [255, 255, 255],
            x: None,
            y: None,
        };
        let result = watermark(img, &params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_watermark_center() {
        let img = create_test_image(200, 100);
        let params = WatermarkParams {
            text: "Center".to_string(),
            opacity: 1.0,
            position: WatermarkPosition::Center,
            font_size: 24,
            color: [255, 255, 255],
            x: None,
            y: None,
        };
        let result = watermark(img, &params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_watermark_long_text() {
        let img = create_test_image(300, 100);
        let params = WatermarkParams {
            text: "This is a very long watermark text to test boundaries".to_string(),
            opacity: 1.0,
            position: WatermarkPosition::BottomRight,
            font_size: 18,
            color: [255, 255, 255],
            x: None,
            y: None,
        };
        let result = watermark(img, &params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_watermark_small_font() {
        let img = create_test_image(100, 50);
        let params = WatermarkParams {
            text: "SmallFont".to_string(),
            opacity: 1.0,
            position: WatermarkPosition::TopLeft,
            font_size: 8,
            color: [255, 255, 255],
            x: None,
            y: None,
        };
        let result = watermark(img, &params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_watermark_large_font() {
        let img = create_test_image(400, 200);
        let params = WatermarkParams {
            text: "LargeFont".to_string(),
            opacity: 1.0,
            position: WatermarkPosition::Center,
            font_size: 64,
            color: [255, 255, 255],
            x: None,
            y: None,
        };
        let result = watermark(img, &params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_watermark_center() {
        let img = create_test_image(200, 100);
        let watermark = DynamicImage::ImageRgba8(ImageBuffer::from_pixel(
            50,
            50,
            Rgba([255, 255, 255, 255]),
        ));
        let params = WatermarkImageParams {
            watermark_url: "http://example.com/wm.png".to_string(),
            opacity: 0.5,
            position: WatermarkPosition::Center,
            scale: None,
            x_offset: None,
            y_offset: None,
        };
        let result = apply_watermark(img, watermark, &params);
        let px = result.get_pixel(100, 50);
        // Blended: 0.5 * 0 + 0.5 * 255 = 127.
        assert!(px[0] > 100);
    }

    #[test]
    fn test_apply_watermark_scaling() {
        let img = create_test_image(200, 100);
        let watermark = DynamicImage::ImageRgba8(ImageBuffer::from_pixel(
            100,
            100,
            Rgba([255, 255, 255, 255]),
        ));
        let params = WatermarkImageParams {
            watermark_url: "http://example.com/wm.png".to_string(),
            opacity: 1.0,
            position: WatermarkPosition::TopLeft,
            scale: Some(0.1), // Should scale to 20px width (200 * 0.1)
            x_offset: None,
            y_offset: None,
        };
        let result = apply_watermark(img, watermark, &params);
        // Check pixel at 10,10 (inside 20x20 box)
        let px = result.get_pixel(10, 10);
        assert_eq!(px[0], 255);
        // Check pixel at 25,25 (outside 20x20 box)
        let px_out = result.get_pixel(25, 25);
        assert_eq!(px_out[0], 0);
    }

    #[test]
    fn test_apply_watermark_offset() {
        let img = create_test_image(200, 100);
        let watermark = DynamicImage::ImageRgba8(ImageBuffer::from_pixel(
            20,
            20,
            Rgba([255, 255, 255, 255]),
        ));
        let params = WatermarkImageParams {
            watermark_url: "http://example.com/wm.png".to_string(),
            opacity: 1.0,
            position: WatermarkPosition::TopLeft,
            scale: None,
            x_offset: Some(50),
            y_offset: Some(20),
        };
        let result = apply_watermark(img, watermark, &params);
        // Should be at 50, 20
        assert_eq!(result.get_pixel(55, 25)[0], 255);
        assert_eq!(result.get_pixel(10, 10)[0], 0);
    }
}
