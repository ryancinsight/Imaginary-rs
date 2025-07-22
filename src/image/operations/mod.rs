//! Image operations module.
//!
//! This module organizes all image processing operations into submodules:
//! - [`transform`]: resizing, rotating, cropping, flipping, enlarging, extracting, zooming, smart cropping, thumbnails
//! - [`color`]: grayscale, brightness/contrast, sharpen, blur
//! - [`watermark`]: text and image watermarking
//! - [`format`]: format conversion, autorotate
//! - [`overlay`]: overlaying images, drawing text
//!
//! Most common operations are re-exported at this level for ergonomic imports.

pub mod color;
pub mod format;
pub mod overlay;
pub mod transform;
pub mod watermark;

// Re-export most common operations for ergonomic use
pub use color::{adjust_brightness, adjust_contrast, blur, grayscale, sharpen};
pub use transform::{
    crop, enlarge, extract, flip_horizontal, flip_vertical, resize, rotate, smart_crop, thumbnail,
    zoom,
};
// pub use watermark::watermark; // Not re-exported at top level unless part of public API
pub use format::{autorotate, convert_format};
// Note: overlay and draw_text are not re-exported; use overlay::overlay if needed internally.
