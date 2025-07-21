//! Watermarking operations for images.
//!
//! This module provides functions to apply text or image watermarks to images as part of the processing pipeline.

use image::{DynamicImage, Rgba};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale, point};
use crate::image::params::{WatermarkParams, WatermarkImageParams, WatermarkPosition};
use image::{GenericImage, GenericImageView, RgbaImage};

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

    // --- NEW: Measure text width/height ---
    let v_metrics = font.v_metrics(scale);
    let glyphs: Vec<_> = font.layout(&params.text, scale, point(0.0, 0.0)).collect();
    let glyphs_width = glyphs
        .iter()
        .filter_map(|g| g.pixel_bounding_box().map(|bb| bb.max.x as f32))
        .last()
        .unwrap_or(0.0)
        .ceil() as u32;
    let glyphs_height = (v_metrics.ascent - v_metrics.descent).ceil() as u32;
    let margin = 10u32;
    let (width, height) = rgba_image.dimensions();

    let (x, y) = match (params.x, params.y) {
        (Some(x), Some(y)) => (x, y),
        _ => {
            match params.position {
                WatermarkPosition::TopLeft => (margin, margin + glyphs_height),
                WatermarkPosition::TopRight => (
                    width.saturating_sub(glyphs_width + margin),
                    margin + glyphs_height
                ),
                WatermarkPosition::BottomLeft => (
                    margin,
                    height.saturating_sub(margin)
                ),
                WatermarkPosition::BottomRight => (
                    width.saturating_sub(glyphs_width + margin),
                    height.saturating_sub(margin)
                ),
                WatermarkPosition::Center => (
                    width.saturating_sub(glyphs_width) / 2,
                    height.saturating_sub(glyphs_height) / 2 + glyphs_height
                ),
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

/// Overlays a watermark image onto the base image at the specified position and opacity.
pub(crate) fn watermark_image(
    mut image: DynamicImage,
    params: &WatermarkImageParams,
) -> DynamicImage {
    // For demonstration, use a placeholder watermark image (solid color or pattern)
    // In a real implementation, params would include the watermark image bytes or path
    let (img_width, img_height) = image.dimensions();
    let watermark_width = img_width / 4;
    let watermark_height = img_height / 4;
    let watermark = RgbaImage::from_pixel(
        watermark_width,
        watermark_height,
        Rgba([255, 255, 255, (params.opacity * 255.0) as u8]),
    );

    // Positioning logic (center by default)
    let (x, y) = match params.position {
        WatermarkPosition::TopLeft => (0, 0),
        WatermarkPosition::TopRight => (img_width - watermark_width, 0),
        WatermarkPosition::BottomLeft => (0, img_height - watermark_height),
        WatermarkPosition::BottomRight => (img_width - watermark_width, img_height - watermark_height),
        WatermarkPosition::Center => (
            (img_width - watermark_width) / 2,
            (img_height - watermark_height) / 2,
        ),
    };

    // Blend watermark onto the image
    for wy in 0..watermark_height {
        for wx in 0..watermark_width {
            let px = watermark.get_pixel(wx, wy);
            let ix = x + wx;
            let iy = y + wy;
            if ix < img_width && iy < img_height {
                let mut base_px = image.get_pixel(ix, iy);
                // Alpha blend
                let alpha = px[3] as f32 / 255.0;
                for c in 0..3 {
                    base_px[c] = ((1.0 - alpha) * base_px[c] as f32 + alpha * px[c] as f32) as u8;
                }
                image.put_pixel(ix, iy, base_px);
            }
        }
    }
    image
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, ImageBuffer, Rgba};
    use crate::image::params::{WatermarkParams, WatermarkPosition};

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
        let result = watermark(&img, &params);
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
        let result = watermark(&img, &params);
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
        let result = watermark(&img, &params);
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
        let result = watermark(&img, &params);
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
        let result = watermark(&img, &params);
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
        let result = watermark(&img, &params);
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
        let result = watermark(&img, &params);
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
        let result = watermark(&img, &params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_watermark_image_center() {
        let img = create_test_image(200, 100);
        let params = WatermarkImageParams {
            opacity: 0.5,
            position: WatermarkPosition::Center,
        };
        let result = watermark_image(img, &params);
        // Check that the center region is not pure black (watermark applied)
        let px = result.get_pixel(100, 50);
        assert!(px[0] > 0 && px[3] == 255);
    }

    #[test]
    fn test_watermark_image_top_left() {
        let img = create_test_image(200, 100);
        let params = WatermarkImageParams {
            opacity: 0.8,
            position: WatermarkPosition::TopLeft,
        };
        let result = watermark_image(img, &params);
        let px = result.get_pixel(10, 10);
        assert!(px[0] > 0 && px[3] == 255);
    }
} 