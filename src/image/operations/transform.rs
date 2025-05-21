//! Transform operations for images.
//!
//! This module provides functions for resizing, rotating, cropping, flipping, enlarging, extracting, zooming, smart cropping, and creating thumbnails.

use image::{DynamicImage, imageops::FilterType, GenericImageView};
use crate::image::params::{ResizeParams, RotateParams, CropParams, ThumbnailParams, ExtractParams, ZoomParams, SmartCropParams, Validate};

/// Resize the image to the exact width and height specified in `params` using Lanczos3 filter.
///
/// # Arguments
/// * `image` - The input image to resize.
/// * `params` - The resize parameters (width, height).
///
/// # Returns
/// A new `DynamicImage` resized to the specified dimensions.
///
/// # Examples
/// # use image::DynamicImage;
/// # let img = DynamicImage::new_rgb8(100, 100);
/// let resized = resize(img, &ResizeParams { width: 100, height: 100 });
pub fn resize(image: DynamicImage, params: &ResizeParams) -> DynamicImage {
    image.resize_exact(params.width, params.height, FilterType::Lanczos3)
}

/// Rotate the image by the specified degrees (90, 180, 270 supported).
///
/// # Arguments
/// * `image` - The input image to rotate.
/// * `params` - The rotation parameters (degrees).
///
/// # Returns
/// A new `DynamicImage` rotated by the specified degrees.
pub fn rotate(image: DynamicImage, params: &RotateParams) -> DynamicImage {
    match params.degrees {
        90.0 => image.rotate90(),
        180.0 => image.rotate180(),
        270.0 => image.rotate270(),
        _ => image.rotate90(),
    }
}

/// Crop the image to the rectangle specified in `params`.
///
/// # Arguments
/// * `image` - The input image to crop.
/// * `params` - The crop parameters (x, y, width, height).
///
/// # Returns
/// A new `DynamicImage` cropped to the specified rectangle.
pub fn crop(image: DynamicImage, params: &CropParams) -> DynamicImage {
    image.crop_imm(params.x, params.y, params.width, params.height)
}

/// Flip the image horizontally.
///
/// # Arguments
/// * `image` - The input image to flip.
///
/// # Returns
/// A new `DynamicImage` flipped horizontally.
pub fn flip_horizontal(image: DynamicImage) -> DynamicImage {
    image.fliph()
}

/// Flip the image vertically.
///
/// # Arguments
/// * `image` - The input image to flip.
///
/// # Returns
/// A new `DynamicImage` flipped vertically.
pub fn flip_vertical(image: DynamicImage) -> DynamicImage {
    image.flipv()
}

/// Enlarge the image to the given width and height, only if the new size is larger.
///
/// # Arguments
/// * `image` - The input image to enlarge.
/// * `params` - The resize parameters (width, height).
///
/// # Returns
/// A new `DynamicImage` enlarged to the specified dimensions, or the original if not larger.
pub fn enlarge(image: DynamicImage, params: &ResizeParams) -> DynamicImage {
    params.validate().expect("Invalid enlarge params");
    let (orig_w, orig_h) = image.dimensions();
    if params.width > orig_w || params.height > orig_h {
        image.resize(params.width, params.height, FilterType::Lanczos3)
    } else {
        image
    }
}

/// Extract (crop) a region from the image at (x, y) with the given width and height.
///
/// # Arguments
/// * `image` - The input image to extract from.
/// * `params` - The extract parameters (x, y, width, height).
///
/// # Returns
/// A new `DynamicImage` containing the extracted region.
pub fn extract(image: DynamicImage, params: &ExtractParams) -> DynamicImage {
    params.validate().expect("Invalid extract params");
    let (img_w, img_h) = image.dimensions();
    let x = params.x.min(img_w);
    let y = params.y.min(img_h);
    let w = params.width.min(img_w.saturating_sub(x));
    let h = params.height.min(img_h.saturating_sub(y));
    image.crop_imm(x, y, w, h)
}

/// Zoom (scale) an image by a given factor (>0).
///
/// # Arguments
/// * `image` - The input image to zoom.
/// * `params` - The zoom parameters (factor).
///
/// # Returns
/// A new `DynamicImage` zoomed by the specified factor.
pub fn zoom(image: DynamicImage, params: &ZoomParams) -> DynamicImage {
    params.validate().expect("Invalid zoom params");
    let (orig_w, orig_h) = image.dimensions();
    let new_w = ((orig_w as f32) * params.factor).round().max(1.0) as u32;
    let new_h = ((orig_h as f32) * params.factor).round().max(1.0) as u32;
    image.resize(new_w, new_h, FilterType::Lanczos3)
}

/// Smart crop an image to the given width and height (currently a center crop).
///
/// # Arguments
/// * `image` - The input image to crop.
/// * `params` - The smart crop parameters (width, height, quality).
///
/// # Returns
/// A new `DynamicImage` smart-cropped to the specified dimensions.
pub fn smart_crop(image: DynamicImage, params: &SmartCropParams) -> DynamicImage {
    params.validate().expect("Invalid smart crop params");
    let (img_w, img_h) = image.dimensions();
    let crop_w = params.width.min(img_w);
    let crop_h = params.height.min(img_h);
    let x = (img_w.saturating_sub(crop_w)) / 2;
    let y = (img_h.saturating_sub(crop_h)) / 2;
    image.crop_imm(x, y, crop_w, crop_h)
}

/// Create a thumbnail of the image with the specified dimensions.
///
/// # Arguments
/// * `image` - The input image to thumbnail.
/// * `params` - The thumbnail parameters (width, height).
///
/// # Returns
/// A new `DynamicImage` thumbnail.
pub fn thumbnail(image: DynamicImage, params: &ThumbnailParams) -> DynamicImage {
    params.validate().expect("Invalid thumbnail params");
    image.thumbnail(params.width, params.height)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, ImageBuffer, Rgba};
    use crate::image::params::{ResizeParams, RotateParams, CropParams, ThumbnailParams, ExtractParams, ZoomParams, SmartCropParams};

    fn create_test_image(width: u32, height: u32) -> DynamicImage {
        DynamicImage::ImageRgba8(ImageBuffer::from_pixel(
            width,
            height,
            Rgba([255u8, 0u8, 0u8, 255u8]),
        ))
    }

    #[test]
    fn test_resize() {
        let img = create_test_image(100, 100);
        let params = ResizeParams { width: 50, height: 50 };
        let resized = resize(img, &params);
        assert_eq!(resized.dimensions(), (50, 50));
    }

    #[test]
    fn test_rotate() {
        let img = create_test_image(100, 100);
        let params = RotateParams { degrees: 90.0 };
        let rotated = rotate(img, &params);
        assert_eq!(rotated.dimensions(), (100, 100));
    }

    #[test]
    fn test_crop() {
        let img = create_test_image(100, 100);
        let params = CropParams { x: 10, y: 10, width: 50, height: 50 };
        let cropped = crop(img, &params);
        assert_eq!(cropped.dimensions(), (50, 50));
    }

    #[test]
    fn test_flip_horizontal() {
        let img = create_test_image(100, 100);
        let flipped = flip_horizontal(img);
        assert_eq!(flipped.dimensions(), (100, 100));
    }

    #[test]
    fn test_flip_vertical() {
        let img = create_test_image(100, 100);
        let flipped = flip_vertical(img);
        assert_eq!(flipped.dimensions(), (100, 100));
    }

    #[test]
    fn test_enlarge() {
        let img = create_test_image(50, 50);
        let params = ResizeParams { width: 100, height: 100 };
        let enlarged = enlarge(img, &params);
        assert_eq!(enlarged.dimensions(), (100, 100));
    }

    #[test]
    fn test_extract() {
        let img = create_test_image(100, 100);
        let params = ExtractParams { x: 10, y: 10, width: 30, height: 30 };
        let extracted = extract(img, &params);
        assert_eq!(extracted.dimensions(), (30, 30));
    }

    #[test]
    fn test_zoom() {
        let img = create_test_image(100, 100);
        let params = ZoomParams { factor: 2.0 };
        let zoomed = zoom(img, &params);
        assert_eq!(zoomed.dimensions(), (200, 200));
    }

    #[test]
    fn test_smart_crop() {
        let img = create_test_image(100, 100);
        let params = SmartCropParams { width: 50, height: 50, quality: None };
        let cropped = smart_crop(img, &params);
        assert_eq!(cropped.dimensions(), (50, 50));
    }

    #[test]
    fn test_thumbnail() {
        let img = create_test_image(100, 100);
        let params = ThumbnailParams { width: 20, height: 20 };
        let thumb = thumbnail(img, &params);
        assert_eq!(thumb.dimensions(), (20, 20));
    }
} 