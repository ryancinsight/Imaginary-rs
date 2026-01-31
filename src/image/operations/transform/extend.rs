use crate::image::params::{ExtendBackground, ExtendParams, Gravity, Validate};
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};

/// Extend or crop an image to the specified dimensions, positioning the original content according to `Gravity`.
///
/// If the target size is larger than the source, the function pads the canvas using the chosen `ExtendBackground`:
/// - `Color(c)`: fills the canvas with `c` and places the source image at the computed offset.
/// - `Mirror`: fills the canvas by reflecting source pixels across the borders (BORDER_REFLECT-style).
/// - `CopyEdges`: fills the canvas by repeating the nearest edge pixels of the source.
///
/// If the target size is smaller than the source, the source image is cropped according to `Gravity`.
///
/// # Returns
///
/// A new `DynamicImage::ImageRgba8` containing the extended or cropped image.
///
/// # Examples
///
/// ```
/// use image::{DynamicImage, GenericImageView};
/// use imaginary::image::params::{ExtendParams, ExtendBackground, Gravity};
/// use imaginary::image::operations::transform::extend::extend;
///
/// let src = DynamicImage::new_rgba8(10, 10);
/// let params = ExtendParams {
///     width: 20,
///     height: 20,
///     gravity: Gravity::Center,
///     background: ExtendBackground::Color([0, 0, 0, 255]),
/// };
///
/// let out = extend(src, &params);
/// assert_eq!(out.dimensions(), (20, 20));
/// ```
pub fn extend(image: DynamicImage, params: &ExtendParams) -> DynamicImage {
    params.validate().expect("Invalid extend params");

    let (src_w, src_h) = image.dimensions();
    let tgt_w = params.width;
    let tgt_h = params.height;

    // No-op if dimensions match
    if src_w == tgt_w && src_h == tgt_h {
        return image;
    }

    // Determine offset based on gravity
    // Logic: calculate empty space, assign based on gravity.
    let x_diff = (tgt_w as i64) - (src_w as i64);
    let y_diff = (tgt_h as i64) - (src_h as i64);

    let (off_x, off_y) = match params.gravity {
        Gravity::Center => (x_diff / 2, y_diff / 2),
        Gravity::North => (x_diff / 2, 0),
        Gravity::NorthEast => (x_diff, 0),
        Gravity::East => (x_diff, y_diff / 2),
        Gravity::SouthEast => (x_diff, y_diff),
        Gravity::South => (x_diff / 2, y_diff),
        Gravity::SouthWest => (0, y_diff),
        Gravity::West => (0, y_diff / 2),
        Gravity::NorthWest => (0, 0),
    };

    // Convert source to RGBA8 for consistent processing
    let src_img_rgba = image.into_rgba8();
    let mut new_img = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(tgt_w, tgt_h);

    match params.background {
        ExtendBackground::Color(c) => {
            // Fill with color
            for pixel in new_img.pixels_mut() {
                *pixel = Rgba(c);
            }

            // Overlay source
            // imageops::overlay requires u32 x, y if strictly inside, but it takes i64 for signed offsets.
            // We need to handle cases where offsets are negative (cropping).
            image::imageops::overlay(&mut new_img, &src_img_rgba, off_x, off_y);
        }
        ExtendBackground::Mirror => {
            for (x, y, pixel) in new_img.enumerate_pixels_mut() {
                // Map to source coords
                let sx = (x as i64) - off_x;
                let sy = (y as i64) - off_y;

                let sx_mapped = reflect(sx, src_w as i64);
                let sy_mapped = reflect(sy, src_h as i64);

                *pixel = *src_img_rgba.get_pixel(sx_mapped as u32, sy_mapped as u32);
            }
        }
        ExtendBackground::CopyEdges => {
            for (x, y, pixel) in new_img.enumerate_pixels_mut() {
                // Map to source coords
                let sx = (x as i64) - off_x;
                let sy = (y as i64) - off_y;

                let sx_clamped = sx.max(0).min((src_w as i64) - 1);
                let sy_clamped = sy.max(0).min((src_h as i64) - 1);

                *pixel = *src_img_rgba.get_pixel(sx_clamped as u32, sy_clamped as u32);
            }
        }
    }

    DynamicImage::ImageRgba8(new_img)
}

/// Map an integer coordinate into the range [0, max-1] using BORDER_REFLECT-style reflection.

///

/// If `max <= 1`, always returns `0`. Otherwise, the input `val` is reflected across the

/// interval boundaries until it lies within `[0, max - 1]`.

///

/// # Examples

///

/// ```
/// use imaginary::image::operations::transform::extend::reflect;
/// assert_eq!(reflect(-1, 5), 0); // -1 reflects to 0

/// assert_eq!(reflect(0, 5), 0);

/// assert_eq!(reflect(4, 5), 4);

/// assert_eq!(reflect(5, 5), 3); // 5 reflects to 3

/// assert_eq!(reflect(6, 5), 2); // 6 reflects to 2

/// assert_eq!(reflect(0, 1), 0); // max == 1 always returns 0

/// assert_eq!(reflect(10, 0), 0); // max <= 0 always returns 0

/// ```
pub fn reflect(mut val: i64, max: i64) -> i64 {
    if max <= 0 {
        return 0;
    }
    if max == 1 {
        return 0;
    }

    while val < 0 || val >= max {
        if val < 0 {
            val = -val - 1;
        } else {
            val = 2 * max - 1 - 1 - val;
        }
    }
    val
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};

    /// Creates a test RGBA image of the given width and height with a simple gradient.
    ///
    /// The pixel at (x, y) will have the value `Rgba([x as u8, y as u8, 0, 255])`.
    ///
    /// # Examples
    ///
    /// ```
    /// let img = create_test_image(2, 3);
    /// let rgba = img.to_rgba8();
    /// assert_eq!(rgba.get_pixel(0, 0).0, [0, 0, 0, 255]);
    /// assert_eq!(rgba.get_pixel(1, 2).0, [1, 2, 0, 255]);
    /// ```
    fn create_test_image(w: u32, h: u32) -> DynamicImage {
        let mut img = ImageBuffer::new(w, h);
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            *pixel = Rgba([x as u8, y as u8, 0, 255]);
        }
        DynamicImage::ImageRgba8(img)
    }

    #[test]
    fn test_extend_color_center() {
        let img = create_test_image(10, 10);
        let params = ExtendParams {
            width: 20,
            height: 20,
            background: ExtendBackground::Color([255, 0, 0, 255]),
            gravity: Gravity::Center,
        };
        let res = extend(img, &params);
        assert_eq!(res.dimensions(), (20, 20));
        // Check background
        assert_eq!(res.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
        // Check image pixel at center offset. (20-10)/2 = 5.
        // 5, 5 should be 0,0 of source (0,0,0,255)
        assert_eq!(res.get_pixel(5, 5), Rgba([0, 0, 0, 255]));
    }

    #[test]
    fn test_extend_copy_edges() {
        let img = create_test_image(10, 10);
        let params = ExtendParams {
            width: 12,
            height: 12,
            background: ExtendBackground::CopyEdges,
            gravity: Gravity::Center, // Offset 1,1
        };
        let res = extend(img, &params);
        // (0,0) should be same as (1,1) -> source(0,0)
        assert_eq!(res.get_pixel(0, 0), res.get_pixel(1, 1));
        // (1,1) corresponds to source(0,0) which is 0,0,0,255
        assert_eq!(res.get_pixel(1, 1), Rgba([0, 0, 0, 255]));
    }

    /// Verifies that `ExtendBackground::Mirror` reflects source pixels into the extended canvas.
    ///
    /// Confirms BORDER_REFLECT-style mapping for out-of-bounds destination coordinates by
    /// checking that pixels outside the source area are read from reflected source positions.
    ///
    /// # Examples
    ///
    /// ```
    /// // Creates a 10x10 test image, extends it to 12x12 with Mirror background and Center gravity,
    /// // and asserts that out-of-bounds pixels correspond to reflected source pixels.
    /// let img = create_test_image(10, 10);
    /// let params = ExtendParams {
    ///     width: 12,
    ///     height: 12,
    ///     background: ExtendBackground::Mirror,
    ///     gravity: Gravity::Center, // Offset 1,1
    /// };
    /// let res = extend(img, &params);
    /// assert_eq!(res.get_pixel(0, 1), Rgba([0, 0, 0, 255])); // reflects to source(0,0)
    /// assert_eq!(res.get_pixel(11, 10), Rgba([8, 9, 0, 255])); // reflects to source(8,9)
    /// ```
    #[test]
    fn test_extend_mirror() {
        let img = create_test_image(10, 10);
        let params = ExtendParams {
            width: 12,
            height: 12,
            background: ExtendBackground::Mirror,
            gravity: Gravity::Center, // Offset 1,1
        };
        let res = extend(img, &params);
        // source(0,0) is at res(1,1).
        // res(0,1) -> sx = 0-1 = -1. reflect(-1) -> 0. should match source(0,0).
        assert_eq!(res.get_pixel(0, 1), Rgba([0, 0, 0, 255]));

        // Let's test bottom right. source(9,9) at res(10,10).
        // res(11,10) -> sx = 11-1 = 10. max=10. reflect(10) -> 2*10-2-10 = 8.
        // should match source(8,9).
        // source(8,9) -> [8, 9, 0, 255]
        assert_eq!(res.get_pixel(11, 10), Rgba([8, 9, 0, 255]));
    }

    #[test]
    fn test_crop() {
        let img = create_test_image(10, 10);
        let params = ExtendParams {
            width: 4,
            height: 4,
            background: ExtendBackground::default(),
            gravity: Gravity::Center,
        };
        let res = extend(img, &params);
        assert_eq!(res.dimensions(), (4, 4));
        // Offset: (4-10)/2 = -3.
        // res(0,0) -> sx = 0 - (-3) = 3. sy = 3.
        // Should match source(3,3) -> [3, 3, 0, 255]
        assert_eq!(res.get_pixel(0, 0), Rgba([3, 3, 0, 255]));
    }

    #[test]
    fn test_gravity_north_west() {
        let img = create_test_image(10, 10);
        let params = ExtendParams {
            width: 20,
            height: 20,
            background: ExtendBackground::default(),
            gravity: Gravity::NorthWest,
        };
        let res = extend(img, &params);
        // NorthWest means (0,0) offset.
        assert_eq!(res.get_pixel(0, 0), Rgba([0, 0, 0, 255]));
        assert_eq!(res.get_pixel(19, 19), Rgba([0, 0, 0, 255])); // default bg black
    }

    #[test]
    fn test_gravity_south_east() {
        let img = create_test_image(10, 10);
        let params = ExtendParams {
            width: 20,
            height: 20,
            background: ExtendBackground::default(),
            gravity: Gravity::SouthEast,
        };
        let res = extend(img, &params);
        // SouthEast means offset (10, 10).
        assert_eq!(res.get_pixel(10, 10), Rgba([0, 0, 0, 255]));
        assert_eq!(res.get_pixel(0, 0), Rgba([0, 0, 0, 255])); // default bg black
    }
}
