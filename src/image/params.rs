use crate::http::errors::ImageError;
use serde::{Deserialize, Serialize};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

/// Trait for validating operation parameters. Implemented by all parameter structs.
pub trait Validate {
    /// Validate the parameters, returning Ok(()) if valid, or an ImageError if invalid.
    fn validate(&self) -> Result<(), ImageError>;
}

/// Filter type to use for resizing.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize, Default, Archive, RkyvDeserialize, RkyvSerialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub enum ResizeFilter {
    #[default]
    Lanczos3,
    Gaussian,
    Nearest,
    Triangle,
    CatmullRom,
}

/// Parameters for resizing an image.
/// - width: target width (must be > 0)
/// - height: target height (must be > 0)
/// - filter: filter algorithm (default: Lanczos3)
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default, Archive, RkyvDeserialize, RkyvSerialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct ResizeParams {
    #[serde(default = "default_dimension")]
    pub width: u32,
    #[serde(default = "default_dimension")]
    pub height: u32,
    #[serde(default)]
    pub filter: ResizeFilter,
}

fn default_dimension() -> u32 {
    100
}

impl Validate for ResizeParams {
    fn validate(&self) -> Result<(), ImageError> {
        if self.width == 0 || self.height == 0 {
            Err(ImageError::InvalidDimensions(
                "Width and height must be greater than zero.".to_string(),
            ))
        } else {
            Ok(())
        }
    }
}

/// Parameters for rotating an image.
/// - degrees: rotation angle (0 <= degrees < 360)
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default, Archive, RkyvDeserialize, RkyvSerialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct RotateParams {
    #[serde(default = "default_degrees")]
    pub degrees: f32,
}

fn default_degrees() -> f32 {
    90.0
}

impl Validate for RotateParams {
    fn validate(&self) -> Result<(), ImageError> {
        if self.degrees < 0.0 || self.degrees >= 360.0 {
            Err(ImageError::InvalidDegrees(
                "Degrees must be between 0 and 360.".to_string(),
            ))
        } else {
            Ok(())
        }
    }
}

/// Parameters for cropping an image.
/// - x, y: top-left corner
/// - width, height: crop size (must be > 0)
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default, Archive, RkyvDeserialize, RkyvSerialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct CropParams {
    #[serde(default)]
    pub x: u32,
    #[serde(default)]
    pub y: u32,
    #[serde(default = "default_dimension")]
    pub width: u32,
    #[serde(default = "default_dimension")]
    pub height: u32,
}

impl Validate for CropParams {
    fn validate(&self) -> Result<(), ImageError> {
        if self.width == 0 || self.height == 0 {
            Err(ImageError::InvalidDimensions(
                "Width and height must be greater than zero.".to_string(),
            ))
        } else {
            Ok(())
        }
    }
}

/// Position for watermark placement.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize, Default, Archive, RkyvDeserialize, RkyvSerialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub enum WatermarkPosition {
    #[default]
    Center,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Parameters for adding a text watermark.
/// - text: watermark text (non-empty)
/// - opacity: 0.0-1.0
/// - position: WatermarkPosition
/// - font_size: > 0
/// - color: [R, G, B]
/// - x, y: optional manual position
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default, Archive, RkyvDeserialize, RkyvSerialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct WatermarkParams {
    #[serde(default)]
    pub text: String,
    #[serde(default = "default_opacity")]
    pub opacity: f32,
    #[serde(default)]
    pub position: WatermarkPosition,
    #[serde(default = "default_font_size")]
    pub font_size: u32,
    #[serde(default = "default_color")]
    pub color: [u8; 3], // RGB color
    #[serde(default)]
    pub x: Option<u32>, // If None, use position for automatic placement
    #[serde(default)]
    pub y: Option<u32>,
}

fn default_opacity() -> f32 {
    0.5
}
fn default_font_size() -> u32 {
    24
}
fn default_color() -> [u8; 3] {
    [255, 255, 255]
} // White

impl Validate for WatermarkParams {
    fn validate(&self) -> Result<(), ImageError> {
        if self.opacity < 0.0 || self.opacity > 1.0 {
            return Err(ImageError::InvalidOpacity(
                "Opacity must be between 0.0 and 1.0".to_string(),
            ));
        }
        if self.text.is_empty() {
            return Err(ImageError::InvalidParameters(
                "Watermark text cannot be empty".to_string(),
            ));
        }
        if self.font_size == 0 {
            return Err(ImageError::InvalidParameters(
                "Font size must be > 0".to_string(),
            ));
        }
        if let (Some(x), Some(y)) = (self.x, self.y) {
            if x == 0 && y == 0 {
                return Err(ImageError::InvalidParameters(
                    "Both x and y coordinates cannot be 0".to_string(),
                ));
            }
        }
        Ok(())
    }
}

/// Parameters for embedding an image in a box (resize fit + pad).
/// - width, height: box size
/// - background: optional RGBA color (default white)
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default, Archive, RkyvDeserialize, RkyvSerialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct EmbedParams {
    #[serde(default = "default_dimension")]
    pub width: u32,
    #[serde(default = "default_dimension")]
    pub height: u32,
    #[serde(default = "default_background_color")]
    pub background: [u8; 4],
}

fn default_background_color() -> [u8; 4] {
    [255, 255, 255, 255]
}

impl Validate for EmbedParams {
    fn validate(&self) -> Result<(), ImageError> {
        if self.width == 0 || self.height == 0 {
             return Err(ImageError::InvalidDimensions(
                "Width and height must be > 0".to_string(),
            ));
        }
        Ok(())
    }
}

/// Parameters for format conversion.
/// - format: target format (e.g., "png", "jpeg")
/// - quality: optional, 0-100
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default, Archive, RkyvDeserialize, RkyvSerialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct FormatConversionParams {
    #[serde(default = "default_format")]
    pub format: String,
    #[serde(default)]
    pub quality: Option<u8>,
}

fn default_format() -> String {
    "png".to_string()
}

impl Validate for FormatConversionParams {
    fn validate(&self) -> Result<(), ImageError> {
        if let Some(quality) = self.quality {
            if quality > 100 {
                return Err(ImageError::InvalidQuality(
                    "Quality must be between 0 and 100.".to_string(),
                ));
            }
        }
        Ok(())
    }
}

/// Parameters for smart cropping.
/// - width, height: target size (must be > 0)
/// - quality: optional
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default, Archive, RkyvDeserialize, RkyvSerialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct SmartCropParams {
    #[serde(default = "default_dimension")]
    pub width: u32,
    #[serde(default = "default_dimension")]
    pub height: u32,
    #[serde(default)]
    pub quality: Option<u8>,
}

impl Validate for SmartCropParams {
    fn validate(&self) -> Result<(), ImageError> {
        if self.width == 0 || self.height == 0 {
            return Err(ImageError::InvalidDimensions(
                "Width and height must be > 0".to_string(),
            ));
        }
        Ok(())
    }
}

/// Parameters for brightness adjustment.
/// - value: brightness delta
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default, Archive, RkyvDeserialize, RkyvSerialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct AdjustBrightnessParams {
    #[serde(default)]
    pub value: i32,
}

impl Validate for AdjustBrightnessParams {
    fn validate(&self) -> Result<(), ImageError> {
        Ok(())
    }
}

/// Parameters for contrast adjustment.
/// - value: contrast delta
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default, Archive, RkyvDeserialize, RkyvSerialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct AdjustContrastParams {
    #[serde(default)]
    pub value: f32,
}

impl Validate for AdjustContrastParams {
    fn validate(&self) -> Result<(), ImageError> {
        Ok(())
    }
}

/// Parameters for Gaussian blur.
/// - sigma: blur radius (> 0)
/// - minampl: optional, minimum amplitude
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default, Archive, RkyvDeserialize, RkyvSerialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct BlurParams {
    pub sigma: f32,
    #[serde(default)]
    pub minampl: Option<f32>,
}

impl Validate for BlurParams {
    fn validate(&self) -> Result<(), ImageError> {
        if self.sigma <= 0.0 {
            return Err(ImageError::InvalidParameters(
                "Blur sigma must be greater than 0".to_string(),
            ));
        }
        if let Some(minampl_val) = self.minampl {
            if minampl_val < 0.0 {
                return Err(ImageError::InvalidParameters(
                    "Blur minampl cannot be negative".to_string(),
                ));
            }
        }
        Ok(())
    }
}

/// Parameters for thumbnail creation.
/// - width, height: target size (must be > 0)
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default, Archive, RkyvDeserialize, RkyvSerialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct ThumbnailParams {
    #[serde(default = "default_dimension")]
    pub width: u32,
    #[serde(default = "default_dimension")]
    pub height: u32,
}

impl Validate for ThumbnailParams {
    fn validate(&self) -> Result<(), ImageError> {
        if self.width == 0 || self.height == 0 {
            return Err(ImageError::InvalidDimensions(
                "Width and height must be > 0".to_string(),
            ));
        }
        Ok(())
    }
}

/// Parameters for extracting a subregion.
/// - x, y: top-left
/// - width, height: region size (must be > 0)
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default, Archive, RkyvDeserialize, RkyvSerialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct ExtractParams {
    #[serde(default)]
    pub x: u32,
    #[serde(default)]
    pub y: u32,
    #[serde(default = "default_dimension")]
    pub width: u32,
    #[serde(default = "default_dimension")]
    pub height: u32,
}

impl Validate for ExtractParams {
    fn validate(&self) -> Result<(), ImageError> {
        if self.width == 0 || self.height == 0 {
            return Err(ImageError::InvalidDimensions(
                "Width and height must be > 0".to_string(),
            ));
        }
        Ok(())
    }
}

/// Parameters for zooming.
/// - factor: zoom factor (> 0)
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default, Archive, RkyvDeserialize, RkyvSerialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct ZoomParams {
    #[serde(default = "default_zoom_factor")]
    pub factor: f32,
}

fn default_zoom_factor() -> f32 {
    1.0
}

impl Validate for ZoomParams {
    fn validate(&self) -> Result<(), ImageError> {
        if self.factor <= 0.0 {
            return Err(ImageError::InvalidParameters(
                "Zoom factor must be > 0".to_string(),
            ));
        }
        Ok(())
    }
}

/// Parameters for image watermarking.
/// - opacity: 0.0-1.0
/// - position: WatermarkPosition
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default, Archive, RkyvDeserialize, RkyvSerialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct WatermarkImageParams {
    #[serde(default = "default_opacity")]
    pub opacity: f32,
    #[serde(default)]
    pub position: WatermarkPosition,
    // In a real implementation, you would also have a field for the watermark image itself (e.g., as a path or bytes)
}

impl Validate for WatermarkImageParams {
    fn validate(&self) -> Result<(), ImageError> {
        if self.opacity < 0.0 || self.opacity > 1.0 {
            return Err(ImageError::InvalidOpacity(
                "Opacity must be between 0.0 and 1.0".to_string(),
            ));
        }
        Ok(())
    }
}

/// Parameters for fitting an image within dimensions (preserving aspect ratio).
/// - width, height: target bounding box (must be > 0)
/// - filter: filter algorithm
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default, Archive, RkyvDeserialize, RkyvSerialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct FitParams {
    #[serde(default = "default_dimension")]
    pub width: u32,
    #[serde(default = "default_dimension")]
    pub height: u32,
    #[serde(default)]
    pub filter: ResizeFilter,
}

impl Validate for FitParams {
    fn validate(&self) -> Result<(), ImageError> {
        if self.width == 0 || self.height == 0 {
            return Err(ImageError::InvalidDimensions(
                "Width and height must be > 0".to_string(),
            ));
        }
        Ok(())
    }
}

/// Parameters for filling dimensions (resize to cover + crop).
/// - width, height: target size (must be > 0)
/// - filter: filter algorithm
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default, Archive, RkyvDeserialize, RkyvSerialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct FillParams {
    #[serde(default = "default_dimension")]
    pub width: u32,
    #[serde(default = "default_dimension")]
    pub height: u32,
    #[serde(default)]
    pub filter: ResizeFilter,
}

impl Validate for FillParams {
    fn validate(&self) -> Result<(), ImageError> {
        if self.width == 0 || self.height == 0 {
            return Err(ImageError::InvalidDimensions(
                "Width and height must be > 0".to_string(),
            ));
        }
        Ok(())
    }
}

/// Parameters for gamma correction.
/// - value: gamma value (must be > 0)
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default, Archive, RkyvDeserialize, RkyvSerialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct GammaParams {
    pub value: f32,
}

impl Validate for GammaParams {
    fn validate(&self) -> Result<(), ImageError> {
        if self.value <= 0.0 {
            return Err(ImageError::InvalidParameters(
                "Gamma value must be > 0".to_string(),
            ));
        }
        Ok(())
    }
}
