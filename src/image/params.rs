use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ResizeParams {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Deserialize)]
pub struct RotateParams {
    pub degrees: f32,
}

#[derive(Debug, Deserialize)]
pub struct CropParams {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Deserialize)]
pub struct WatermarkParams {
    pub opacity: f32,
    pub position: WatermarkPosition,
}

#[derive(Debug, Deserialize)]
pub enum WatermarkPosition {
    Center,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Debug, Deserialize)]
pub struct FormatConversionParams {
    pub format: String,
    pub quality: Option<u8>,
}

#[derive(Debug, Deserialize)]
pub struct SmartCropParams {
    pub width: u32,
    pub height: u32,
    pub quality: Option<u8>,
}