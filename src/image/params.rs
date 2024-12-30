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