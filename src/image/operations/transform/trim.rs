use image::{DynamicImage, GenericImageView, Rgba};
use crate::image::params::{TrimParams, Validate};

/// Automatically crops uniform borders from the image.
///
/// Scans the image edges to find the bounding box where pixels deviate from the
/// reference color (background) by more than the specified threshold.
///
/// # Arguments
///
/// * `image` - The input image.
/// * `params` - Trim parameters including threshold and optional background color.
///
/// # Returns
///
/// The cropped image, or the original image if no trimming is possible (e.g. fully uniform).
pub fn trim(image: DynamicImage, params: &TrimParams) -> DynamicImage {
    params.validate().expect("Invalid trim params");

    let (width, height) = image.dimensions();
    if width == 0 || height == 0 {
        return image;
    }

    let img_rgba = image.to_rgba8();

    // Determine reference color
    let ref_color = if let Some(c) = params.background_color {
        Rgba(c)
    } else {
        *img_rgba.get_pixel(0, 0)
    };

    // Calculate max allowed distance
    // Max distance in RGBA (0-255 per channel) is sqrt(255^2 * 4) approx 510.
    let max_dist = (255.0 * 255.0 * 4.0f32).sqrt();
    let threshold_dist = params.threshold * max_dist;
    let threshold_sq = threshold_dist * threshold_dist;

    // Helper to check if a pixel is "background"
    let is_background = |p: &Rgba<u8>| -> bool {
        let r_diff = p[0] as f32 - ref_color[0] as f32;
        let g_diff = p[1] as f32 - ref_color[1] as f32;
        let b_diff = p[2] as f32 - ref_color[2] as f32;
        let a_diff = p[3] as f32 - ref_color[3] as f32;

        let dist_sq = r_diff * r_diff + g_diff * g_diff + b_diff * b_diff + a_diff * a_diff;
        dist_sq <= threshold_sq
    };

    // 1. Find Top
    let mut top = 0;
    let mut found_top = false;
    for y in 0..height {
        for x in 0..width {
            if !is_background(img_rgba.get_pixel(x, y)) {
                top = y;
                found_top = true;
                break;
            }
        }
        if found_top { break; }
    }

    if !found_top {
        // Image is fully background
        return image;
    }

    // 2. Find Bottom
    let mut bottom = height - 1;
    for y in (top..height).rev() {
         let mut row_has_content = false;
         for x in 0..width {
             if !is_background(img_rgba.get_pixel(x, y)) {
                 row_has_content = true;
                 break;
             }
         }
         if row_has_content {
             bottom = y;
             break;
         }
    }

    // 3. Find Left
    let mut left = 0;
    let mut found_left = false;
    for x in 0..width {
        for y in top..=bottom {
            if !is_background(img_rgba.get_pixel(x, y)) {
                left = x;
                found_left = true;
                break;
            }
        }
        if found_left { break; }
    }

    // 4. Find Right
    let mut right = width - 1;
    for x in (left..width).rev() {
        let mut col_has_content = false;
        for y in top..=bottom {
            if !is_background(img_rgba.get_pixel(x, y)) {
                col_has_content = true;
                break;
            }
        }
        if col_has_content {
            right = x;
            break;
        }
    }

    let crop_width = right - left + 1;
    let crop_height = bottom - top + 1;

    // Return cropped view
    image.crop_imm(left, top, crop_width, crop_height)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};

    fn create_image(w: u32, h: u32, color: [u8; 4]) -> DynamicImage {
        DynamicImage::ImageRgba8(ImageBuffer::from_pixel(w, h, Rgba(color)))
    }

    #[test]
    fn test_trim_uniform_no_op() {
        let img = create_image(100, 100, [255, 255, 255, 255]);
        let params = TrimParams { threshold: 0.1, background_color: None };
        let trimmed = trim(img.clone(), &params);
        assert_eq!(trimmed.dimensions(), (100, 100));
    }

    #[test]
    fn test_trim_simple_border() {
        let mut img = ImageBuffer::from_pixel(100, 100, Rgba([255, 255, 255, 255])); // White bg
        // Draw a black box in the middle: 10,10 to 89,89 (80x80)
        for y in 10..90 {
            for x in 10..90 {
                img.put_pixel(x, y, Rgba([0, 0, 0, 255]));
            }
        }
        let dynamic_img = DynamicImage::ImageRgba8(img);
        let params = TrimParams { threshold: 0.1, background_color: None };

        let trimmed = trim(dynamic_img, &params);
        assert_eq!(trimmed.dimensions(), (80, 80));
    }

    #[test]
    fn test_trim_with_noise_within_threshold() {
        let mut img = ImageBuffer::from_pixel(100, 100, Rgba([0, 0, 0, 255])); // Black bg

        // Add some noise (values 5,5,5) which is close to 0,0,0
        // Sqrt(5^2*3) = sqrt(75) ~ 8.66. Max dist ~510. 8.66/510 ~ 0.017.
        // Threshold 0.1 allows dist up to 51.
        img.put_pixel(5, 5, Rgba([10, 10, 10, 255]));

        // Draw content (White)
        for y in 20..80 {
            for x in 20..80 {
                img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
            }
        }

        let dynamic_img = DynamicImage::ImageRgba8(img);
        let params = TrimParams { threshold: 0.1, background_color: None }; // auto detect black

        // The noise at 5,5 should be treated as background.
        // Content is at 20,20 60x60.
        let trimmed = trim(dynamic_img, &params);
        assert_eq!(trimmed.dimensions(), (60, 60));
    }

    #[test]
    fn test_trim_explicit_background() {
        let mut img = ImageBuffer::from_pixel(100, 100, Rgba([255, 0, 0, 255])); // Red bg
        // Draw Blue box
        for y in 10..90 {
            for x in 10..90 {
                img.put_pixel(x, y, Rgba([0, 0, 255, 255]));
            }
        }
        let dynamic_img = DynamicImage::ImageRgba8(img);

        // If we don't specify bg, it detects red (corner).
        // Let's specify Red explicitly.
        let params = TrimParams { threshold: 0.1, background_color: Some([255, 0, 0, 255]) };
        let trimmed = trim(dynamic_img, &params);
        assert_eq!(trimmed.dimensions(), (80, 80));
    }

    #[test]
    fn test_trim_different_background() {
         let mut img = ImageBuffer::from_pixel(100, 100, Rgba([255, 0, 0, 255])); // Red bg
         // Draw Blue content
         for y in 10..90 {
            for x in 10..90 {
                img.put_pixel(x, y, Rgba([0, 0, 255, 255]));
            }
         }
         let dynamic_img = DynamicImage::ImageRgba8(img);

         // Specify Green as background. The whole image (Red and Blue) is "not Green".
         // So it should return the whole image.
         // Red vs Green: dist is large.
         let params = TrimParams { threshold: 0.1, background_color: Some([0, 255, 0, 255]) };
         let trimmed = trim(dynamic_img, &params);
         assert_eq!(trimmed.dimensions(), (100, 100));
    }
}
