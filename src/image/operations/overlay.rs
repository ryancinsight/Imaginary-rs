//! Overlay operations for images.
//!
//! This module provides functions for overlaying images and drawing text.

use crate::http::errors::AppError;
use image::Rgba;
use image::{DynamicImage, GenericImage};
use rusttype::{point, Font, Scale};

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
    image: DynamicImage,
    overlay_image: DynamicImage,
    x: u32,
    y: u32,
) -> Result<DynamicImage, AppError> {
    let mut img = image.clone();
    img.copy_from(&overlay_image, x, y)
        .map_err(|e| AppError::ImageProcessingError(e.to_string()))?;
    Ok(img)
}

/// Draws text onto the image at the specified position and font size.
#[allow(dead_code)]
pub(crate) fn draw_text(
    image: DynamicImage,
    text: &str,
    x: u32,
    y: u32,
    font_size: u32,
) -> DynamicImage {
    let font_data = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/fonts/DejaVuSans.ttf"
    ));
    let font = Font::try_from_bytes(font_data as &[u8]).expect("Failed to load font");
    let scale = Scale::uniform(font_size as f32);
    let color = Rgba([255, 255, 255, 255]);
    let mut rgba = image.to_rgba8();
    let v_metrics = font.v_metrics(scale);
    let start = point(x as f32, y as f32 + v_metrics.ascent);
    for glyph in font.layout(text, scale, start) {
        if let Some(bb) = glyph.pixel_bounding_box() {
            glyph.draw(|gx, gy, gv| {
                let px = bb.min.x + gx as i32;
                let py = bb.min.y + gy as i32;
                if px >= 0 && py >= 0 && (px as u32) < rgba.width() && (py as u32) < rgba.height() {
                    let pixel = rgba.get_pixel_mut(px as u32, py as u32);
                    // Simple alpha blend
                    for c in 0..3 {
                        pixel[c] = ((1.0 - gv) * pixel[c] as f32 + gv * color[c] as f32) as u8;
                    }
                    pixel[3] = 255;
                }
            });
        }
    }
    DynamicImage::ImageRgba8(rgba)
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

    #[test]
    fn test_draw_text_center() {
        let img = create_test_image(200, 100);
        let result = draw_text(img, "Hello", 80, 40, 24);
        // Scan a 20x20 region around (100, 50) for any non-black, fully opaque pixel
        let mut found = false;
        for dx in 90..110 {
            for dy in 40..60 {
                let px = result.get_pixel(dx, dy);
                if px[0] > 0 && px[3] == 255 {
                    found = true;
                    break;
                }
            }
            if found {
                break;
            }
        }
        assert!(found, "No text pixels found in expected region");
    }

    #[test]
    fn test_draw_text_top_left() {
        let img = create_test_image(200, 100);
        let result = draw_text(img, "A", 0, 0, 32);
        // Scan a 20x20 region in the top-left for any non-black, fully opaque pixel
        let mut found = false;
        for dx in 0..20 {
            for dy in 0..20 {
                let px = result.get_pixel(dx, dy);
                if px[0] > 0 && px[3] == 255 {
                    found = true;
                    break;
                }
            }
            if found {
                break;
            }
        }
        assert!(found, "No text pixels found in expected region");
    }
}
