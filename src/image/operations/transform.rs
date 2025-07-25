//! Transform operations for images.
//!
//! This module provides functions for resizing, rotating, cropping, flipping, enlarging, extracting, zooming, smart cropping, and creating thumbnails.

use crate::image::params::{
    CropParams, ExtractParams, ResizeParams, RotateParams, SmartCropParams, ThumbnailParams,
    Validate, ZoomParams,
};
use image::{imageops::FilterType, DynamicImage, GenericImageView};

/// Resize the image to the given dimensions.
pub fn resize(image: DynamicImage, params: &ResizeParams) -> DynamicImage {
    image.resize_exact(params.width, params.height, FilterType::Lanczos3)
}

/// Rotate the image by the given degrees.
pub fn rotate(image: DynamicImage, params: &RotateParams) -> DynamicImage {
    match params.degrees {
        90.0 => image.rotate90(),
        180.0 => image.rotate180(),
        270.0 => image.rotate270(),
        _ => image.rotate90(),
    }
}

/// Crop the image to the given rectangle.
pub fn crop(image: DynamicImage, params: &CropParams) -> DynamicImage {
    image.crop_imm(params.x, params.y, params.width, params.height)
}

/// Flip the image horizontally.
pub fn flip_horizontal(image: DynamicImage) -> DynamicImage {
    image.fliph()
}

/// Flip the image vertically.
pub fn flip_vertical(image: DynamicImage) -> DynamicImage {
    image.flipv()
}

/// Enlarge the image using the given resize parameters.
pub fn enlarge(image: DynamicImage, params: &ResizeParams) -> DynamicImage {
    params.validate().expect("Invalid enlarge params");
    let (orig_w, orig_h) = image.dimensions();
    if params.width > orig_w || params.height > orig_h {
        image.resize(params.width, params.height, FilterType::Lanczos3)
    } else {
        image
    }
}

/// Extract a subregion from the image.
pub fn extract(image: DynamicImage, params: &ExtractParams) -> DynamicImage {
    params.validate().expect("Invalid extract params");
    let (img_w, img_h) = image.dimensions();
    let x = params.x.min(img_w);
    let y = params.y.min(img_h);
    let w = params.width.min(img_w.saturating_sub(x));
    let h = params.height.min(img_h.saturating_sub(y));
    image.crop_imm(x, y, w, h)
}

/// Zoom into the image by the given factor.
pub fn zoom(image: DynamicImage, params: &ZoomParams) -> DynamicImage {
    params.validate().expect("Invalid zoom params");
    let (orig_w, orig_h) = image.dimensions();
    let new_w = ((orig_w as f32) * params.factor).round().max(1.0) as u32;
    let new_h = ((orig_h as f32) * params.factor).round().max(1.0) as u32;
    image.resize(new_w, new_h, FilterType::Lanczos3)
}

/// Perform a smart crop on the image using the given parameters.
pub fn smart_crop(image: DynamicImage, params: &SmartCropParams) -> DynamicImage {
    params.validate().expect("Invalid smart crop params");
    let (img_w, img_h) = image.dimensions();
    let crop_w = params.width.min(img_w);
    let crop_h = params.height.min(img_h);
    let x = (img_w.saturating_sub(crop_w)) / 2;
    let y = (img_h.saturating_sub(crop_h)) / 2;
    image.crop_imm(x, y, crop_w, crop_h)
}

/// Create a thumbnail of the image with the given parameters.
pub fn thumbnail(image: DynamicImage, params: &ThumbnailParams) -> DynamicImage {
    params.validate().expect("Invalid thumbnail params");
    image.thumbnail(params.width, params.height)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::params::{
        CropParams, ExtractParams, ResizeParams, RotateParams, SmartCropParams, ThumbnailParams,
        ZoomParams,
    };
    use image::{DynamicImage, ImageBuffer, Rgba};

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
        let params = ResizeParams {
            width: 50,
            height: 50,
        };
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
        let params = CropParams {
            x: 10,
            y: 10,
            width: 50,
            height: 50,
        };
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
        let params = ResizeParams {
            width: 100,
            height: 100,
        };
        let enlarged = enlarge(img, &params);
        assert_eq!(enlarged.dimensions(), (100, 100));
    }

    #[test]
    fn test_extract() {
        let img = create_test_image(100, 100);
        let params = ExtractParams {
            x: 10,
            y: 10,
            width: 30,
            height: 30,
        };
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
        let params = SmartCropParams {
            width: 50,
            height: 50,
            quality: None,
        };
        let cropped = smart_crop(img, &params);
        assert_eq!(cropped.dimensions(), (50, 50));
    }

    #[test]
    fn test_thumbnail() {
        let img = create_test_image(100, 100);
        let params = ThumbnailParams {
            width: 20,
            height: 20,
        };
        let thumb = thumbnail(img, &params);
        assert_eq!(thumb.dimensions(), (20, 20));
    }
}
