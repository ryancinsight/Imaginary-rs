use crate::http::errors::AppError;
use color_thief::{get_palette, ColorFormat};
use image::DynamicImage;

pub fn extract_palette(image: &DynamicImage, max_colors: u8) -> Result<Vec<[u8; 3]>, AppError> {
    let rgb_image = image.to_rgb8();
    let bytes = rgb_image.as_raw();

    // Quality 10 is standard.
    match get_palette(bytes, ColorFormat::Rgb, 10, max_colors) {
        Ok(palette) => Ok(palette.into_iter().map(|c| [c.r, c.g, c.b]).collect()),
        Err(e) => Err(AppError::ImageProcessingError(format!("Failed to extract palette: {:?}", e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};

    fn create_test_image(width: u32, height: u32) -> DynamicImage {
        DynamicImage::ImageRgba8(ImageBuffer::from_pixel(
            width,
            height,
            Rgba([255u8, 0u8, 0u8, 255u8]), // Red image
        ))
    }

    #[test]
    fn test_extract_palette() {
        let img = create_test_image(100, 100);
        let palette = extract_palette(&img, 5).unwrap();
        println!("Palette: {:?}", palette);
        assert!(!palette.is_empty());
        // Should contain red or close to it
        let red = [255, 0, 0];
        // Check if any color is close to red
        let found = palette.iter().any(|c| {
            let dist = (c[0] as i32 - red[0] as i32).abs() +
                       (c[1] as i32 - red[1] as i32).abs() +
                       (c[2] as i32 - red[2] as i32).abs();
            dist < 10
        });
        assert!(found, "Red color not found in palette: {:?}", palette);
    }
}
