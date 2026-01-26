use crate::http::errors::ImageError;
use crate::image::params::{FitParams, ResizeFilter, Validate};
use image::{imageops, DynamicImage, GenericImageView, ImageBuffer, Rgba};

/// Resize the image to fit within the given dimensions while preserving aspect ratio.
///
/// This operation behaves like "contain":
/// - Scales down if the image is larger than the target dimensions.
/// - Does NOT upscale if the image is smaller than the target dimensions.
/// - Pads the remaining space with the specified background color (default black) to match the target dimensions.
pub fn fit(image: DynamicImage, params: &FitParams) -> Result<DynamicImage, (DynamicImage, ImageError)> {
    if let Err(e) = params.validate() {
        return Err((image, e));
    }

    let (orig_w, orig_h) = image.dimensions();
    let (target_w, target_h) = (params.width, params.height);

    // Calculate scale factor
    // Scale down if needed, but do not upscale (max scale is 1.0)
    let scale_w = target_w as f64 / orig_w as f64;
    let scale_h = target_h as f64 / orig_h as f64;
    let scale = scale_w.min(scale_h).min(1.0);

    let new_w = (orig_w as f64 * scale).round().max(1.0) as u32;
    let new_h = (orig_h as f64 * scale).round().max(1.0) as u32;

    // Convert ResizeFilter to image::imageops::FilterType
    let filter = match params.filter {
        ResizeFilter::Lanczos3 => imageops::FilterType::Lanczos3,
        ResizeFilter::Gaussian => imageops::FilterType::Gaussian,
        ResizeFilter::Nearest => imageops::FilterType::Nearest,
        ResizeFilter::Triangle => imageops::FilterType::Triangle,
        ResizeFilter::CatmullRom => imageops::FilterType::CatmullRom,
    };

    // Resize using image::imageops
    // Note: This converts the image to RGBA8 format as imageops::resize returns an ImageBuffer
    // compatible with the GenericImageView input, but typically we want consistent RGBA output for padding.
    let resized_buffer = imageops::resize(&image, new_w, new_h, filter);
    let resized = DynamicImage::ImageRgba8(resized_buffer);

    // If dimensions match target exactly, return the resized image
    // (This only happens if aspect ratios match AND no padding is needed)
    if new_w == target_w && new_h == target_h {
        return Ok(resized);
    }

    // Create background and overlay resized image
    let background_color = params.background.unwrap_or([0, 0, 0, 255]);
    let mut background = ImageBuffer::from_pixel(target_w, target_h, Rgba(background_color));

    let x = (target_w.saturating_sub(new_w)) / 2;
    let y = (target_h.saturating_sub(new_h)) / 2;

    imageops::overlay(&mut background, &resized, x as i64, y as i64);

    Ok(DynamicImage::ImageRgba8(background))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};

    fn create_test_image(width: u32, height: u32) -> DynamicImage {
        DynamicImage::ImageRgba8(ImageBuffer::from_pixel(
            width,
            height,
            Rgba([255u8, 0u8, 0u8, 255u8]), // Red
        ))
    }

    #[test]
    fn test_fit_downscale_pad_width() {
        // Original: 100x50 (2:1)
        // Target: 50x50
        // Expected: Scale to 50x25. Pad top/bottom.
        let img = create_test_image(100, 50);
        let params = FitParams {
            width: 50,
            height: 50,
            filter: ResizeFilter::Nearest,
            background: Some([0, 255, 0, 255]), // Green background
        };

        let result = fit(img, &params).unwrap();
        assert_eq!(result.dimensions(), (50, 50));

        // Check center pixel (should be image -> Red)
        assert_eq!(result.get_pixel(25, 25), Rgba([255, 0, 0, 255]));

        // Check top pixel (should be padding -> Green)
        // Image is 25px high, centered in 50px. Y range: 12 to 37 (approx).
        // Y=5 should be background.
        assert_eq!(result.get_pixel(25, 5), Rgba([0, 255, 0, 255]));
    }

    #[test]
    fn test_fit_downscale_pad_height() {
        // Original: 50x100 (1:2)
        // Target: 50x50
        // Expected: Scale to 25x50. Pad left/right.
        let img = create_test_image(50, 100);
        let params = FitParams {
            width: 50,
            height: 50,
            filter: ResizeFilter::Nearest,
            background: None, // Default black
        };

        let result = fit(img, &params).unwrap();
        assert_eq!(result.dimensions(), (50, 50));

        // Check center pixel (Red)
        assert_eq!(result.get_pixel(25, 25), Rgba([255, 0, 0, 255]));

        // Check left pixel (Black)
        // Image is 25px wide, centered in 50px. X range: 12 to 37.
        // X=5 should be background.
        assert_eq!(result.get_pixel(5, 25), Rgba([0, 0, 0, 255]));
    }

    #[test]
    fn test_fit_no_upscale() {
        // Original: 50x50
        // Target: 100x100
        // Expected: No scale (50x50). Pad to 100x100.
        let img = create_test_image(50, 50);
        let params = FitParams {
            width: 100,
            height: 100,
            filter: ResizeFilter::Nearest,
            background: None,
        };

        let result = fit(img, &params).unwrap();
        assert_eq!(result.dimensions(), (100, 100));

        // Image should be 50x50 in center.
        // Center: 50,50.
        assert_eq!(result.get_pixel(50, 50), Rgba([255, 0, 0, 255]));

        // Corner (should be black)
        assert_eq!(result.get_pixel(0, 0), Rgba([0, 0, 0, 255]));
    }

    #[test]
    fn test_fit_exact() {
        // Original: 50x50
        // Target: 50x50
        // Expected: No change.
        let img = create_test_image(50, 50);
        let params = FitParams {
            width: 50,
            height: 50,
            filter: ResizeFilter::Nearest,
            background: None,
        };

        let result = fit(img, &params).unwrap();
        assert_eq!(result.dimensions(), (50, 50));
    }
}
