use image::{DynamicImage, ImageBuffer, Rgba};
use std::path::PathBuf;

/// Creates a test image with the specified dimensions filled with a solid color (red).
///
/// # Arguments
/// * `width` - The width of the image in pixels.
/// * `height` - The height of the image in pixels.
///
/// # Returns
/// * `DynamicImage` filled with red pixels.
pub fn create_test_image(width: u32, height: u32) -> DynamicImage {
    DynamicImage::ImageRgba8(ImageBuffer::from_pixel(
        width,
        height,
        Rgba([255u8, 0u8, 0u8, 255u8]), // Red image
    ))
}

/// Loads a test image from the `images_test` directory relative to the project root.
///
/// # Arguments
/// * `filename` - The name of the image file to load.
///
/// # Panics
/// Panics if the image cannot be loaded.
///
/// # Returns
/// * `DynamicImage` loaded from the specified file.
pub fn load_test_image(filename: &str) -> DynamicImage {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("images_test");
    path.push(filename);
    image::open(path).expect("Failed to load test image")
}

/// Saves a test image to the `target/test_output` directory.
///
/// # Arguments
/// * `img` - The image to save.
/// * `filename` - The name to use for the saved file.
///
/// # Returns
/// * `Ok(())` if the image was saved successfully, or an error otherwise.
pub fn save_test_image(img: &DynamicImage, filename: &str) -> std::io::Result<()> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("test_output");
    std::fs::create_dir_all(&path)?;
    path.push(filename);
    img.save(path).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
} 