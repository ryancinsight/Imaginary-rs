use image::{DynamicImage, ImageFormat, GenericImageView};
use std::io::Cursor;
use crate::image::params::Validate;
use crate::http::errors;
use std::fs::File;
use image::io::Reader as ImageReader;

pub fn load_image_from_path(path: &str) -> Result<DynamicImage, image::ImageError> {
    ImageReader::open(path)?.decode()
}

pub fn save_image_to_path(image: &DynamicImage, path: &str, format: ImageFormat) -> Result<(), image::ImageError> {
    let mut output = File::create(path)?;
    image.write_to(&mut output, format)
}

pub fn get_image_dimensions(image_bytes: &[u8]) -> Option<(u32, u32)> {
    if let Ok(img) = image::load_from_memory(image_bytes) {
        Some(img.dimensions())
    } else {
        None
    }
}

pub fn get_image_format(image_bytes: &[u8]) -> Option<ImageFormat> {
    image::guess_format(image_bytes).ok()
}

pub fn load_image_from_bytes(image_bytes: &[u8]) -> Result<DynamicImage, String> {
    image::load_from_memory(image_bytes)
        .map_err(|e| format!("Failed to load image: {}", e))
}

pub fn save_image_to_bytes(image: &DynamicImage, format: ImageFormat) -> Result<Vec<u8>, String> {
    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);
    image.write_to(&mut cursor, format)
        .map_err(|e| format!("Failed to save image: {}", e))?;
    Ok(buffer)
}

// Add a new function to validate parameters
pub fn validate_params<T: Validate>(params: &T) -> Result<(), errors::ImageError> {
    params.validate()
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};

    fn create_test_image() -> Vec<u8> {
        let img = ImageBuffer::from_pixel(100, 100, Rgba([255u8, 0u8, 0u8, 255u8]));
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        img.write_to(&mut cursor, ImageFormat::Png).unwrap();
        buffer
    }

    #[test]
    fn test_get_image_dimensions() {
        let image_bytes = create_test_image();
        let dimensions = get_image_dimensions(&image_bytes);
        assert_eq!(dimensions, Some((100, 100)));
    }

    #[test]
    fn test_get_image_format() {
        let image_bytes = create_test_image();
        let format = get_image_format(&image_bytes);
        assert_eq!(format, Some(ImageFormat::Png));
    }

    #[test]
    fn test_load_image_from_bytes() {
        let image_bytes = create_test_image();
        let result = load_image_from_bytes(&image_bytes);
        assert!(result.is_ok());
        let image = result.unwrap();
        assert_eq!(image.dimensions(), (100, 100));
    }

    #[test]
    fn test_save_image_to_bytes() {
        let image = DynamicImage::ImageRgba8(
            ImageBuffer::from_pixel(100, 100, Rgba([255u8, 0u8, 0u8, 255u8]))
        );
        let result = save_image_to_bytes(&image, ImageFormat::Png);
        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert!(!bytes.is_empty());
    }
}