use crate::http::errors::AppError;
use image::{DynamicImage, imageops::FilterType, GenericImage, ImageFormat};
use std::io::Cursor;
use crate::image::params::{ResizeParams, RotateParams, CropParams, FormatConversionParams};

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
    image.filter3x3(&[0.0, -1.0, 0.0, -1.0, 5.0, -1.0, 0.0, -1.0, 0.0])
}

pub fn blur(image: DynamicImage) -> DynamicImage {
    image.blur(2.0) // Adjust the blur radius as needed
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
    let params = FormatConversionParams { format: "png".to_string() };
    let converted_img = convert_format(dynamic_img.clone(), &params).unwrap();
    
    assert_eq!(converted_img.color(), ColorType::Rgba8); // Check the color type
}
