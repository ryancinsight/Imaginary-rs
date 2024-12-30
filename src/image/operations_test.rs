use super::*; // Import everything from operations.rs
use image::{DynamicImage, RgbaImage};

#[test]
fn test_resize() {
    let img = RgbaImage::new(100, 100); // Create a dummy image
    let dynamic_img = DynamicImage::ImageRgba8(img);
    let resized_img = resize(dynamic_img.clone(), 50, 50);
    
    assert_eq!(resized_img.dimensions(), (50, 50));
}

#[test]
fn test_rotate() {
    let img = RgbaImage::new(100, 100);
    let dynamic_img = DynamicImage::ImageRgba8(img);
    let rotated_img = rotate(dynamic_img.clone(), 90.0);
    
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
