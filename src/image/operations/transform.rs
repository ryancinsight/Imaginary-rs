//! Transform operations for images.
//!
//! This module provides functions for resizing, rotating, cropping, flipping, enlarging, extracting, zooming, smart cropping, and creating thumbnails.

use crate::image::params::{
    CropParams, ExtractParams, FillParams, FitParams, ResizeFilter, ResizeParams, RotateParams,
    SmartCropParams, ThumbnailParams, Validate, ZoomParams,
};
use fast_image_resize::images::Image;
use fast_image_resize::{FilterType as FastFilterType, PixelType, ResizeAlg, ResizeOptions, Resizer};
use image::{imageops::FilterType, DynamicImage, GenericImageView};
use std::num::NonZeroU32;

impl From<&ResizeFilter> for ResizeAlg {
    fn from(filter: &ResizeFilter) -> Self {
        match filter {
            ResizeFilter::Lanczos3 => ResizeAlg::Convolution(FastFilterType::Lanczos3),
            ResizeFilter::Gaussian => ResizeAlg::Convolution(FastFilterType::Gaussian),
            ResizeFilter::Nearest => ResizeAlg::Nearest,
            ResizeFilter::Triangle => ResizeAlg::Convolution(FastFilterType::Bilinear),
            ResizeFilter::CatmullRom => ResizeAlg::Convolution(FastFilterType::CatmullRom),
        }
    }
}

// Fallback for image crate operations (rotate/crop)
impl From<&ResizeFilter> for FilterType {
    fn from(filter: &ResizeFilter) -> Self {
        match filter {
            ResizeFilter::Lanczos3 => FilterType::Lanczos3,
            ResizeFilter::Gaussian => FilterType::Gaussian,
            ResizeFilter::Nearest => FilterType::Nearest,
            ResizeFilter::Triangle => FilterType::Triangle,
            ResizeFilter::CatmullRom => FilterType::CatmullRom,
        }
    }
}

/// Helper function to perform resizing using fast_image_resize.
/// This converts the image to RGBA8, resizes it, and returns a new DynamicImage.
fn resize_fast(
    image: &DynamicImage,
    width: u32,
    height: u32,
    filter: &ResizeFilter,
) -> DynamicImage {
    let width_nz = NonZeroU32::new(width).unwrap_or(NonZeroU32::new(1).unwrap());
    let height_nz = NonZeroU32::new(height).unwrap_or(NonZeroU32::new(1).unwrap());

    // Convert to RGBA8 which is U8x4. This ensures compatibility.
    // Note: This involves a clone/conversion if not already RGBA8.
    let src_image = image.to_rgba8();
    // Default to 1x1 if source image has 0 dimension (which technically shouldn't happen for loaded images but safe to handle)
    let src_width = NonZeroU32::new(src_image.width()).unwrap_or(NonZeroU32::new(1).unwrap());
    let src_height = NonZeroU32::new(src_image.height()).unwrap_or(NonZeroU32::new(1).unwrap());

    let src = Image::from_vec_u8(
        src_width.get(),
        src_height.get(),
        src_image.into_raw(),
        PixelType::U8x4,
    )
    .expect("Failed to create source image for resizing");

    let mut dst = Image::new(width_nz.get(), height_nz.get(), PixelType::U8x4);

    let mut resizer = Resizer::new();
    let resize_opts = ResizeOptions::new().resize_alg(ResizeAlg::from(filter));

    resizer
        .resize(&src, &mut dst, &resize_opts)
        .expect("Resize failed");

    let dst_raw = dst.into_vec();
    DynamicImage::ImageRgba8(
        image::ImageBuffer::from_raw(width_nz.get(), height_nz.get(), dst_raw)
            .expect("Failed to create buffer from resized data"),
    )
}

/// Resize the image to the given dimensions using fast_image_resize.
pub fn resize(image: &DynamicImage, params: &ResizeParams) -> DynamicImage {
    resize_fast(image, params.width, params.height, &params.filter)
}

/// Resize the image to fit within the given dimensions, preserving aspect ratio.
pub fn fit(image: &DynamicImage, params: &FitParams) -> DynamicImage {
    params.validate().expect("Invalid fit params");
    let (orig_w, orig_h) = image.dimensions();

    // Calculate new dimensions
    // Fit means the result must fit INSIDE the box.
    let scale_w = (params.width as f64) / (orig_w as f64);
    let scale_h = (params.height as f64) / (orig_h as f64);
    let scale = scale_w.min(scale_h);

    let new_w = (orig_w as f64 * scale).round().max(1.0) as u32;
    let new_h = (orig_h as f64 * scale).round().max(1.0) as u32;

    resize_fast(image, new_w, new_h, &params.filter)
}

/// Resize the image to fill the given dimensions, cropping excess.
pub fn fill(image: &DynamicImage, params: &FillParams) -> DynamicImage {
    params.validate().expect("Invalid fill params");
    let (orig_w, orig_h) = image.dimensions();

    // Calculate scale to COVER
    let scale_w = (params.width as f64) / (orig_w as f64);
    let scale_h = (params.height as f64) / (orig_h as f64);
    let scale = scale_w.max(scale_h);

    let resized_w = (orig_w as f64 * scale).round().max(1.0) as u32;
    let resized_h = (orig_h as f64 * scale).round().max(1.0) as u32;

    // Resize first
    let resized = resize_fast(image, resized_w, resized_h, &params.filter);

    // Then Center Crop
    let x = (resized_w.saturating_sub(params.width)) / 2;
    let y = (resized_h.saturating_sub(params.height)) / 2;

    resized.crop_imm(x, y, params.width, params.height)
}

/// Rotate the image by the given degrees.
pub fn rotate(image: &DynamicImage, params: &RotateParams) -> DynamicImage {
    match params.degrees {
        90.0 => image.rotate90(),
        180.0 => image.rotate180(),
        270.0 => image.rotate270(),
        _ => image.rotate90(),
    }
}

/// Crop the image to the given rectangle.
pub fn crop(image: &DynamicImage, params: &CropParams) -> DynamicImage {
    image.crop_imm(params.x, params.y, params.width, params.height)
}

/// Flip the image horizontally.
pub fn flip_horizontal(image: &DynamicImage) -> DynamicImage {
    image.fliph()
}

/// Flip the image vertically.
pub fn flip_vertical(image: &DynamicImage) -> DynamicImage {
    image.flipv()
}

/// Enlarge the image using the given resize parameters.
pub fn enlarge(image: &DynamicImage, params: &ResizeParams) -> DynamicImage {
    params.validate().expect("Invalid enlarge params");
    let (orig_w, orig_h) = image.dimensions();
    if params.width > orig_w || params.height > orig_h {
        resize_fast(image, params.width, params.height, &params.filter)
    } else {
        image.clone()
    }
}

/// Extract a subregion from the image.
pub fn extract(image: &DynamicImage, params: &ExtractParams) -> DynamicImage {
    params.validate().expect("Invalid extract params");
    let (img_w, img_h) = image.dimensions();
    let x = params.x.min(img_w);
    let y = params.y.min(img_h);
    let w = params.width.min(img_w.saturating_sub(x));
    let h = params.height.min(img_h.saturating_sub(y));
    image.crop_imm(x, y, w, h)
}

/// Zoom into the image by the given factor.
pub fn zoom(image: &DynamicImage, params: &ZoomParams) -> DynamicImage {
    params.validate().expect("Invalid zoom params");
    let (orig_w, orig_h) = image.dimensions();
    let new_w = ((orig_w as f32) * params.factor).round().max(1.0) as u32;
    let new_h = ((orig_h as f32) * params.factor).round().max(1.0) as u32;
    // Zoom usually implies high quality
    let filter = ResizeFilter::Lanczos3;
    resize_fast(image, new_w, new_h, &filter)
}

/// Perform a smart crop on the image using the given parameters.
pub fn smart_crop(image: &DynamicImage, params: &SmartCropParams) -> DynamicImage {
    params.validate().expect("Invalid smart crop params");
    let (img_w, img_h) = image.dimensions();
    let crop_w = params.width.min(img_w);
    let crop_h = params.height.min(img_h);
    let x = (img_w.saturating_sub(crop_w)) / 2;
    let y = (img_h.saturating_sub(crop_h)) / 2;
    image.crop_imm(x, y, crop_w, crop_h)
}

/// Create a thumbnail of the image with the given parameters.
pub fn thumbnail(image: &DynamicImage, params: &ThumbnailParams) -> DynamicImage {
    params.validate().expect("Invalid thumbnail params");

    // Calculate thumbnail dimensions (preserving aspect ratio)
    let (orig_w, orig_h) = image.dimensions();
    let scale = (params.width as f64 / orig_w as f64).min(params.height as f64 / orig_h as f64);

    let new_w = (orig_w as f64 * scale).round().max(1.0) as u32;
    let new_h = (orig_h as f64 * scale).round().max(1.0) as u32;

    resize_fast(image, new_w, new_h, &ResizeFilter::Lanczos3)
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
            ..Default::default()
        };
        let resized = resize(&img, &params);
        assert_eq!(resized.dimensions(), (50, 50));
    }

    #[test]
    fn test_rotate() {
        let img = create_test_image(100, 100);
        let params = RotateParams { degrees: 90.0 };
        let rotated = rotate(&img, &params);
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
        let cropped = crop(&img, &params);
        assert_eq!(cropped.dimensions(), (50, 50));
    }

    #[test]
    fn test_fit() {
        let img = create_test_image(100, 50); // 2:1 ratio
        let params = FitParams {
            width: 50,
            height: 50,
            filter: ResizeFilter::Nearest,
        };
        let fitted = fit(&img, &params);
        // Should fit into 50x50.
        // 100x50 -> 50x25 to preserve aspect ratio.
        assert_eq!(fitted.dimensions(), (50, 25));
    }

    #[test]
    fn test_fill() {
        let img = create_test_image(100, 50); // 2:1 ratio
        let params = FillParams {
            width: 50,
            height: 50,
            filter: ResizeFilter::Nearest,
        };
        let filled = fill(&img, &params);
        // Should fill 50x50.
        // Scale to cover 50x50:
        // scale_w = 50/100 = 0.5
        // scale_h = 50/50 = 1.0
        // max(0.5, 1.0) = 1.0
        // resized = 100x50
        // crop center 50x50
        assert_eq!(filled.dimensions(), (50, 50));
    }

    #[test]
    fn test_flip_horizontal() {
        let img = create_test_image(100, 100);
        let flipped = flip_horizontal(&img);
        assert_eq!(flipped.dimensions(), (100, 100));
    }

    #[test]
    fn test_flip_vertical() {
        let img = create_test_image(100, 100);
        let flipped = flip_vertical(&img);
        assert_eq!(flipped.dimensions(), (100, 100));
    }

    #[test]
    fn test_enlarge() {
        let img = create_test_image(50, 50);
        let params = ResizeParams {
            width: 100,
            height: 100,
            ..Default::default()
        };
        let enlarged = enlarge(&img, &params);
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
        let extracted = extract(&img, &params);
        assert_eq!(extracted.dimensions(), (30, 30));
    }

    #[test]
    fn test_zoom() {
        let img = create_test_image(100, 100);
        let params = ZoomParams { factor: 2.0 };
        let zoomed = zoom(&img, &params);
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
        let cropped = smart_crop(&img, &params);
        assert_eq!(cropped.dimensions(), (50, 50));
    }

    #[test]
    fn test_thumbnail() {
        let img = create_test_image(100, 100);
        let params = ThumbnailParams {
            width: 20,
            height: 20,
        };
        let thumb = thumbnail(&img, &params);
        assert_eq!(thumb.dimensions(), (20, 20));
    }
}
