//! Transform operations for images.
//!
//! This module provides functions for resizing, rotating, cropping, flipping, enlarging, extracting, zooming, smart cropping, and creating thumbnails.

use crate::image::params::{
    CropParams, EmbedParams, ExtractParams, FillParams, FitParams, ResizeFilter, ResizeParams, RotateParams,
    SmartCropParams, ThumbnailParams, Validate, ZoomParams,
};
use fast_image_resize::images::Image;
use fast_image_resize::{FilterType as FastFilterType, PixelType, ResizeAlg, ResizeOptions, Resizer};
use image::{imageops::FilterType, DynamicImage, GenericImageView, ImageBuffer, Rgba};
use imageproc::gradients::sobel_gradients;
use rayon::prelude::*;
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

pub mod fit;
pub use fit::fit;
pub mod extend;
pub use extend::extend;
pub mod trim;
pub use trim::trim;

/// Helper function to perform resizing using fast_image_resize.
/// This attempts to preserve the pixel format to avoid unnecessary conversions.
pub(crate) fn resize_fast(
    image: DynamicImage,
    width: u32,
    height: u32,
    filter: &ResizeFilter,
) -> DynamicImage {
    let width_nz = NonZeroU32::new(width).unwrap_or(NonZeroU32::new(1).unwrap());
    let height_nz = NonZeroU32::new(height).unwrap_or(NonZeroU32::new(1).unwrap());
    let resize_alg = ResizeAlg::from(filter);

    match image {
        DynamicImage::ImageRgb8(img) => {
            let src_width = NonZeroU32::new(img.width()).unwrap_or(NonZeroU32::new(1).unwrap());
            let src_height = NonZeroU32::new(img.height()).unwrap_or(NonZeroU32::new(1).unwrap());
            let src = Image::from_vec_u8(
                src_width.get(),
                src_height.get(),
                img.into_raw(),
                PixelType::U8x3,
            )
            .expect("Failed to create source image");

            let mut dst = Image::new(width_nz.get(), height_nz.get(), PixelType::U8x3);
            let mut resizer = Resizer::new();
            resizer
                .resize(&src, &mut dst, &ResizeOptions::new().resize_alg(resize_alg))
                .expect("Resize failed");

            DynamicImage::ImageRgb8(
                image::ImageBuffer::from_raw(width_nz.get(), height_nz.get(), dst.into_vec())
                    .expect("Failed to create buffer"),
            )
        }
        DynamicImage::ImageLuma8(img) => {
            let src_width = NonZeroU32::new(img.width()).unwrap_or(NonZeroU32::new(1).unwrap());
            let src_height = NonZeroU32::new(img.height()).unwrap_or(NonZeroU32::new(1).unwrap());
            let src = Image::from_vec_u8(
                src_width.get(),
                src_height.get(),
                img.into_raw(),
                PixelType::U8,
            )
            .expect("Failed to create source image");

            let mut dst = Image::new(width_nz.get(), height_nz.get(), PixelType::U8);
            let mut resizer = Resizer::new();
            resizer
                .resize(&src, &mut dst, &ResizeOptions::new().resize_alg(resize_alg))
                .expect("Resize failed");

            DynamicImage::ImageLuma8(
                image::ImageBuffer::from_raw(width_nz.get(), height_nz.get(), dst.into_vec())
                    .expect("Failed to create buffer"),
            )
        }
        DynamicImage::ImageLumaA8(img) => {
            let src_width = NonZeroU32::new(img.width()).unwrap_or(NonZeroU32::new(1).unwrap());
            let src_height = NonZeroU32::new(img.height()).unwrap_or(NonZeroU32::new(1).unwrap());
            let src = Image::from_vec_u8(
                src_width.get(),
                src_height.get(),
                img.into_raw(),
                PixelType::U8x2,
            )
            .expect("Failed to create source image");

            let mut dst = Image::new(width_nz.get(), height_nz.get(), PixelType::U8x2);
            let mut resizer = Resizer::new();
            resizer
                .resize(&src, &mut dst, &ResizeOptions::new().resize_alg(resize_alg))
                .expect("Resize failed");

            DynamicImage::ImageLumaA8(
                image::ImageBuffer::from_raw(width_nz.get(), height_nz.get(), dst.into_vec())
                    .expect("Failed to create buffer"),
            )
        }
        _ => {
            // Fallback to RGBA8
            let src_image = image.into_rgba8();
            let src_width =
                NonZeroU32::new(src_image.width()).unwrap_or(NonZeroU32::new(1).unwrap());
            let src_height =
                NonZeroU32::new(src_image.height()).unwrap_or(NonZeroU32::new(1).unwrap());

            let src = Image::from_vec_u8(
                src_width.get(),
                src_height.get(),
                src_image.into_raw(),
                PixelType::U8x4,
            )
            .expect("Failed to create source image for resizing");

            let mut dst = Image::new(width_nz.get(), height_nz.get(), PixelType::U8x4);

            let mut resizer = Resizer::new();
            let resize_opts = ResizeOptions::new().resize_alg(resize_alg);

            resizer
                .resize(&src, &mut dst, &resize_opts)
                .expect("Resize failed");

            let dst_raw = dst.into_vec();
            DynamicImage::ImageRgba8(
                image::ImageBuffer::from_raw(width_nz.get(), height_nz.get(), dst_raw)
                    .expect("Failed to create buffer from resized data"),
            )
        }
    }
}

/// Resize the image to the given dimensions using fast_image_resize.
pub fn resize(image: DynamicImage, params: &ResizeParams) -> DynamicImage {
    resize_fast(image, params.width, params.height, &params.filter)
}

/// Resize the image to fit within the given dimensions, preserving aspect ratio.
/// This internal helper allows upscaling (unlike the public fit operation).
fn resize_fit_scale(image: DynamicImage, params: &FitParams) -> DynamicImage {
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
pub fn fill(image: DynamicImage, params: &FillParams) -> DynamicImage {
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

/// Embed the image in a box of the given dimensions, centered, with the given background color.
pub fn embed(image: DynamicImage, params: &EmbedParams) -> DynamicImage {
    params.validate().expect("Invalid embed params");

    // 1. Fit the image into the box
    let fit_params = FitParams {
        width: params.width,
        height: params.height,
        filter: ResizeFilter::Lanczos3,
        background: None,
    };
    let resized = resize_fit_scale(image, &fit_params).into_rgba8();

    // 2. Create a new image with background color
    let mut background = ImageBuffer::from_pixel(
        params.width,
        params.height,
        Rgba(params.background),
    );

    // 3. Overlay the resized image on the background (centered)
    let (w, h) = resized.dimensions();
    let x = (params.width.saturating_sub(w)) / 2;
    let y = (params.height.saturating_sub(h)) / 2;

    image::imageops::overlay(&mut background, &resized, x as i64, y as i64);

    DynamicImage::ImageRgba8(background)
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
        resize_fast(image, params.width, params.height, &params.filter)
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
    // Zoom usually implies high quality
    let filter = ResizeFilter::Lanczos3;
    resize_fast(image, new_w, new_h, &filter)
}

/// Perform a smart crop on the image using the given parameters.
/// It uses edge detection (Sobel) to find the region with the highest entropy/energy.
pub fn smart_crop(image: DynamicImage, params: &SmartCropParams) -> DynamicImage {
    params.validate().expect("Invalid smart crop params");
    let (img_w, img_h) = image.dimensions();
    let crop_w = params.width.min(img_w);
    let crop_h = params.height.min(img_h);

    if crop_w == img_w && crop_h == img_h {
        return image;
    }

    // Convert to grayscale for edge detection
    let gray = image.to_luma8();

    // Calculate Sobel gradients (energy map)
    let gradients = sobel_gradients(&gray);
    // gradients is ImageBuffer<Luma<u16>, Vec<u16>>

    // Create integral image (Summed Area Table) for O(1) window sum
    // Use u64 to prevent overflow. Dimensions + 1 for boundary handling.
    let w_plus_1 = (img_w + 1) as usize;
    let h_plus_1 = (img_h + 1) as usize;
    let mut integral = vec![0u64; w_plus_1 * h_plus_1];

    // Optimize: Access raw buffer directly to avoid bounds checks in inner loop
    let raw_gradients = gradients.as_raw();
    let mut grad_iter = raw_gradients.iter();

    for y in 0..img_h {
        let mut row_sum = 0u64;

        let row_offset = (y + 1) as usize * w_plus_1;
        let prev_row_offset = y as usize * w_plus_1;

        for x in 0..img_w {
            // We can safely unwrap here because the loop bounds match the image dimensions
            // and we're iterating over the raw buffer which corresponds exactly to these dimensions.
            let val = unsafe { *grad_iter.next().unwrap_unchecked() } as u64;
            row_sum += val;

            let x_plus_1 = (x + 1) as usize;
            let i = row_offset + x_plus_1;
            let i_prev = prev_row_offset + x_plus_1;

            unsafe {
                *integral.get_unchecked_mut(i) = *integral.get_unchecked(i_prev) + row_sum;
            }
        }
    }

    // Helper to get sum of rect (x, y, w, h)
    let get_energy = |x: u32, y: u32, w: u32, h: u32| -> u64 {
        let x0 = x as usize;
        let y0 = y as usize;
        let x1 = (x + w) as usize;
        let y1 = (y + h) as usize;

        // I(D) + I(A) - I(B) - I(C)
        let i_d = integral[y1 * w_plus_1 + x1];
        let i_a = integral[y0 * w_plus_1 + x0];
        let i_b = integral[y0 * w_plus_1 + x1];
        let i_c = integral[y1 * w_plus_1 + x0];

        i_d + i_a - i_b - i_c
    };

    // Find best crop position
    let mut best_x = 0;
    let mut best_y = 0;

    // Calculate stride based on quality.
    // Default (None) uses a heuristic (approx 5% of crop dimension).
    // Quality 100 uses step 1.
    // Quality 0 uses step ~10% of crop dimension.
    let max_step_x = (crop_w / 10).max(1);
    let max_step_y = (crop_h / 10).max(1);
    let min_step = 1;

    let q = params.quality.unwrap_or(50).min(100) as f32 / 100.0;

    let step_x = (max_step_x as f32 * (1.0 - q) + min_step as f32 * q)
        .round()
        .max(1.0) as u32;
    let step_y = (max_step_y as f32 * (1.0 - q) + min_step as f32 * q)
        .round()
        .max(1.0) as u32;

    // Generate candidates for parallel processing
    let mut candidates = Vec::new();
    let mut x = 0;
    while x <= img_w - crop_w {
        let mut y = 0;
        while y <= img_h - crop_h {
            candidates.push((x, y));
            if y == img_h - crop_h { break; }
            y = (y + step_y).min(img_h - crop_h);
        }
        if x == img_w - crop_w { break; }
        x = (x + step_x).min(img_w - crop_w);
    }

    // Find best candidate using parallel iterator
    let best_candidate = candidates.par_iter()
        .map(|&(x, y)| {
            (x, y, get_energy(x, y, crop_w, crop_h))
        })
        .max_by_key(|&(_, _, energy)| energy);

    let mut max_energy = 0;

    if let Some((bx, by, energy)) = best_candidate {
        best_x = bx;
        best_y = by;
        max_energy = energy;
    }

    // Refine search around best_x, best_y with step=1
    let range_x = step_x;
    let range_y = step_y;

    let start_x = best_x.saturating_sub(range_x);
    let end_x = (best_x + range_x).min(img_w - crop_w);
    let start_y = best_y.saturating_sub(range_y);
    let end_y = (best_y + range_y).min(img_h - crop_h);

    for x in start_x..=end_x {
        for y in start_y..=end_y {
             let energy = get_energy(x, y, crop_w, crop_h);
             if energy > max_energy {
                 max_energy = energy;
                 best_x = x;
                 best_y = y;
             }
        }
    }

    image.crop_imm(best_x, best_y, crop_w, crop_h)
}

/// Create a thumbnail of the image with the given parameters.
pub fn thumbnail(image: DynamicImage, params: &ThumbnailParams) -> DynamicImage {
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
        CropParams, EmbedParams, ExtractParams, ResizeParams, RotateParams, SmartCropParams, ThumbnailParams,
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
    fn test_fit_internal() {
        let img = create_test_image(100, 50); // 2:1 ratio
        let params = FitParams {
            width: 50,
            height: 50,
            filter: ResizeFilter::Nearest,
            background: None,
        };
        let fitted = resize_fit_scale(img, &params);
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
        let filled = fill(img, &params);
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
            ..Default::default()
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
    fn test_smart_crop_quality() {
        let img = create_test_image(100, 100);

        // Test with quality 100 (step 1)
        let params_high = SmartCropParams {
            width: 50,
            height: 50,
            quality: Some(100),
        };
        let cropped_high = smart_crop(img.clone(), &params_high);
        assert_eq!(cropped_high.dimensions(), (50, 50));

        // Test with quality 0 (step ~10% crop)
        let params_low = SmartCropParams {
            width: 50,
            height: 50,
            quality: Some(0),
        };
        let cropped_low = smart_crop(img.clone(), &params_low);
        assert_eq!(cropped_low.dimensions(), (50, 50));

        // Test with quality 50 (default)
        let params_mid = SmartCropParams {
            width: 50,
            height: 50,
            quality: Some(50),
        };
        let cropped_mid = smart_crop(img, &params_mid);
        assert_eq!(cropped_mid.dimensions(), (50, 50));
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

    #[test]
    fn test_embed() {
        let img = create_test_image(100, 50); // 2:1
        let params = EmbedParams {
            width: 100,
            height: 100,
            background: [0, 0, 0, 0],
        };
        let embedded = embed(img, &params);
        // Should be 100x100
        assert_eq!(embedded.dimensions(), (100, 100));
        // Image should be centered.
        // Fit result: 100x50.
        // Y offset: (100-50)/2 = 25.
        // Pixel at 50, 10 should be background (0).
        let p_bg = embedded.get_pixel(50, 10);
        assert_eq!(p_bg, Rgba([0, 0, 0, 0]));
        // Pixel at 50, 50 should be image (Red).
        let p_img = embedded.get_pixel(50, 50);
        assert_eq!(p_img, Rgba([255, 0, 0, 255]));
    }
}
