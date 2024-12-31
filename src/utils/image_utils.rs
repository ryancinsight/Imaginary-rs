use image::{DynamicImage, ImageFormat, ImageReader, GenericImageView};
use std::fs::File;
use crate::image::params::Validate;
use crate::http::errors;
pub fn load_image_from_path(path: &str) -> Result<DynamicImage, image::ImageError> {
    ImageReader::open(path)?.decode()
}

pub fn save_image_to_path(image: &DynamicImage, path: &str, format: ImageFormat) -> Result<(), image::ImageError> {
    let mut output = File::create(path)?;
    image.write_to(&mut output, format)
}

pub fn get_image_dimensions(image: &DynamicImage) -> (u32, u32) {
    image.dimensions()
}

// Add a new function to validate parameters
pub fn validate_params<T: Validate>(params: &T) -> Result<(), errors::ImageError> {
    params.validate()
}