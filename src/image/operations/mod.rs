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

pub mod transform;
pub mod color;
pub mod watermark;
pub mod format;
pub mod overlay;

// Re-export most common operations for ergonomic use
pub use transform::{resize, rotate, crop, flip_horizontal, flip_vertical, enlarge, extract, zoom, smart_crop, thumbnail};
pub use color::{grayscale, adjust_brightness, adjust_contrast, sharpen, blur};
// pub use watermark::watermark; // Not re-exported at top level unless part of public API
pub use format::{convert_format, autorotate};
// Note: overlay and draw_text are not re-exported; use overlay::overlay if needed internally. 