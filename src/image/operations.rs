use crate::http::errors::AppError;
use image::{DynamicImage, imageops::FilterType, GenericImage, ImageFormat};
use std::io::Cursor;
use crate::image::params::{ResizeParams, RotateParams, CropParams, FormatConversionParams, BlurParams};

pub fn resize(image: DynamicImage, params: &ResizeParams) -> DynamicImage {
    image.resize_exact(params.width, params.height, FilterType::Lanczos3)
}

pub fn rotate(image: DynamicImage, params: &RotateParams) -> DynamicImage {
    match params.degrees {
        90.0 => image.rotate90(),
        180.0 => image.rotate180(),
        270.0 => image.rotate270(),
        _ => image.rotate90(),
    }
}

pub fn crop(image: DynamicImage, params: &CropParams) -> DynamicImage {
    image.crop_imm(params.x, params.y, params.width, params.height)
}

pub fn flip_horizontal(image: DynamicImage) -> DynamicImage {
    image.fliph()
}

pub fn flip_vertical(image: DynamicImage) -> DynamicImage {
    image.flipv()
}

pub fn grayscale(image: DynamicImage) -> DynamicImage {
    image.to_luma8().into()
}

pub fn adjust_brightness(image: DynamicImage, value: i32) -> DynamicImage {
    image.brighten(value)
}

pub fn adjust_contrast(image: DynamicImage, value: f32) -> DynamicImage {
    image.adjust_contrast(value)
}

pub fn sharpen(image: DynamicImage) -> DynamicImage {
    // The image crate uses a 3x3 kernel for sharpen. 
    // Parameters like amount/radius/threshold are not directly supported by image::sharpen.
    // For more advanced sharpening, a custom convolution or different library might be needed.
    // image.sharpen3x3(); // This is a guess, image crate's sharpen is `unsharpen` or `filter3x3`.
    // Correct would be: image.filter3x3(&[0.0, -1.0, 0.0, -1.0, 5.0, -1.0, 0.0, -1.0, 0.0])
    // or use image.unsharpen(sigma, threshold) if that's closer to desired effect.
    // For simplicity, I will use the fixed 3x3 sharpen kernel here.
    let sharpen_kernel: [f32; 9] = [-1.0, -1.0, -1.0,
                                    -1.0,  9.0, -1.0,
                                    -1.0, -1.0, -1.0];
    image.filter3x3(&sharpen_kernel)
}

pub fn blur(image: DynamicImage, params: &BlurParams) -> DynamicImage {
    if params.minampl.is_some() {
        tracing::warn!("Blur operation: 'minampl' parameter is provided but not currently used by the image crate's basic blur. Only sigma is applied.");
    }
    image.blur(params.sigma)
}

pub fn overlay(image: DynamicImage, overlay_image: DynamicImage, x: u32, y: u32) -> Result<DynamicImage, AppError> {
    let mut img = image.clone();
    img.copy_from(&overlay_image, x, y).map_err(|e| AppError::ImageProcessingError(e.to_string()))?; // Handle error
    Ok(img)
}

pub fn draw_text(image: DynamicImage, _text: &str, _x: u32, _y: u32, _font_size: u32) -> DynamicImage {
    // Placeholder implementation
    image // Return the original image for now
}

pub fn convert_format(image: DynamicImage, params: &FormatConversionParams) -> Result<DynamicImage, AppError> {
    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);
    image.write_to(&mut cursor, ImageFormat::from_extension(&params.format).unwrap())
        .map_err(|e| AppError::ImageProcessingError(e.to_string()))?;
    image::load_from_memory(&buffer)
        .map_err(|e| AppError::ImageProcessingError(e.to_string()))
}

#[test]
fn test_resize() {
    let img = RgbaImage::new(100, 100); // Create a dummy image
    let dynamic_img = DynamicImage::ImageRgba8(img);
    let params = ResizeParams { width: 50, height: 50 };
    let resized_img = resize(dynamic_img.clone(), &params);
    
    assert_eq!(resized_img.dimensions(), (50, 50));
}

#[test]
fn test_rotate() {
    let img = RgbaImage::new(100, 100);
    let dynamic_img = DynamicImage::ImageRgba8(img);
    let params = RotateParams { degrees: 90.0 };
    let rotated_img = rotate(dynamic_img.clone(), &params);
    
    assert_eq!(rotated_img.dimensions(), (100, 100)); // Dimensions should remain the same
}

#[test]
fn test_overlay() {
    let img1 = RgbaImage::new(100, 100);
    let img2 = RgbaImage::new(50, 50);
    let dynamic_img1 = DynamicImage::ImageRgba8(img1);
    let dynamic_img2 = DynamicImage::ImageRgba8(img2);
    
    let result = overlay(dynamic_img1.clone(), dynamic_img2, 25, 25);
    
    assert!(result.is_ok()); // Ensure the overlay operation succeeded
    let overlaid_img = result.unwrap();
    assert_eq!(overlaid_img.dimensions(), (100, 100)); // Dimensions should remain the same
}

#[test]
fn test_convert_format() {
    let img = RgbaImage::new(100, 100);
    let dynamic_img = DynamicImage::ImageRgba8(img);
    let params = FormatConversionParams { format: "png".to_string(), quality: Some(90) };
    let converted_img = convert_format(dynamic_img.clone(), &params).unwrap();
    
    assert_eq!(converted_img.color(), ColorType::Rgba8); // Check the color type
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, GenericImage};
    use crate::image::params::WatermarkPosition; // If needed for overlay/watermark tests

    fn create_test_image(width: u32, height: u32) -> DynamicImage {
        DynamicImage::ImageRgba8(image::ImageBuffer::from_pixel(width, height, Rgba([255u8,0u8,0u8,255u8])))
    }

    #[test]
    fn test_blur_operation() {
        let img = create_test_image(10, 10);
        let params = BlurParams { sigma: 1.5, minampl: None };
        let blurred_img = blur(img, &params);
        assert_eq!(blurred_img.width(), 10);
        assert_eq!(blurred_img.height(), 10);
        // Further checks would involve pixel comparison if expected output is known
    }

     #[test]
    fn test_sharpen_operation() {
        let img = create_test_image(10,10);
        let sharpened_img = sharpen(img);
        assert_eq!(sharpened_img.width(), 10);
        assert_eq!(sharpened_img.height(), 10);
    }
}
