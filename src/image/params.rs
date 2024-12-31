use serde::Deserialize;
use crate::http::errors::ImageError;

pub trait Validate {
    fn validate(&self) -> Result<(), ImageError>;
}

#[derive(Debug, Deserialize, Default)]
pub struct ResizeParams {
    #[serde(default = "default_dimension")]
    pub width: u32,
    #[serde(default = "default_dimension")]
    pub height: u32,
}

fn default_dimension() -> u32 { 100 }

impl Validate for ResizeParams {
    fn validate(&self) -> Result<(), ImageError> {
        if self.width == 0 || self.height == 0 {
            Err(ImageError::InvalidDimensions("Width and height must be greater than zero.".to_string()))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct RotateParams {
    #[serde(default = "default_degrees")]
    pub degrees: f32,
}

fn default_degrees() -> f32 { 90.0 }

impl Validate for RotateParams {
    fn validate(&self) -> Result<(), ImageError> {
        if self.degrees < 0.0 || self.degrees >= 360.0 {
            Err(ImageError::InvalidDegrees("Degrees must be between 0 and 360.".to_string()))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Deserialize, Default)]
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
            Err(ImageError::InvalidDimensions("Width and height must be greater than zero.".to_string()))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct WatermarkParams {
    #[serde(default = "default_opacity")]
    pub opacity: f32,
    #[serde(default)]
    pub position: WatermarkPosition,
}

fn default_opacity() -> f32 { 0.5 }

impl Validate for WatermarkParams {
    fn validate(&self) -> Result<(), ImageError> {
        if self.opacity < 0.0 || self.opacity > 1.0 {
            Err(ImageError::InvalidOpacity("Opacity must be between 0.0 and 1.0.".to_string()))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub enum WatermarkPosition {
    #[default]
    Center,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Debug, Deserialize, Default)]
pub struct FormatConversionParams {
    #[serde(default = "default_format")]
    pub format: String,
    #[serde(default)]
    pub quality: Option<u8>,
}

fn default_format() -> String { "png".to_string() }

impl Validate for FormatConversionParams {
    fn validate(&self) -> Result<(), ImageError> {
        if let Some(quality) = self.quality {
            if quality > 100 {
                return Err(ImageError::InvalidQuality("Quality must be between 0 and 100.".to_string()));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, Default)]
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
            Err(ImageError::InvalidDimensions("Width and height must be greater than zero.".to_string()))
        } else if let Some(quality) = self.quality {
            if quality > 100 {
                Err(ImageError::InvalidQuality("Quality must be between 0 and 100.".to_string()))
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}