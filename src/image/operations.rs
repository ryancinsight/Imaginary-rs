use crate::http::errors::AppError;
use image::{DynamicImage, imageops::FilterType, GenericImage};

pub fn resize(image: DynamicImage, width: u32, height: u32) -> DynamicImage {
    image.resize_exact(width, height, FilterType::Lanczos3)
}

pub fn rotate(image: DynamicImage, degrees: f32) -> DynamicImage {
    match degrees {
        90.0 => image.rotate90(),
        180.0 => image.rotate180(),
        270.0 => image.rotate270(),
        _ => image.rotate90(),
    }
}

pub fn crop(image: DynamicImage, x: u32, y: u32, width: u32, height: u32) -> DynamicImage {
    image.crop_imm(x, y, width, height)
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
}