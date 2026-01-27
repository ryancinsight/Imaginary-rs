//! Types and enums for describing image processing pipelines and supported operations.
//!
//! This module defines the data structures used to specify a sequence of image operations (pipeline)
//! and the set of operations supported by the pipeline executor.

use serde::{Deserialize, Serialize};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use super::params::*;

/// Specification for a single operation in an image processing pipeline.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Archive, RkyvDeserialize, RkyvSerialize)]
#[serde(rename_all = "camelCase")]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct PipelineOperationSpec {
    /// The operation to perform, including its parameters.
    #[serde(flatten)]
    pub operation: PipelineOperation,
    /// If true, ignore failure of this operation and continue the pipeline.
    #[serde(default)]
    pub ignore_failure: bool,
}

/// Enum of all supported image operations for the pipeline, including their parameters.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Archive, RkyvDeserialize, RkyvSerialize)]
#[serde(tag = "operation", content = "params")]
#[serde(rename_all = "camelCase")]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub enum PipelineOperation {
    Resize(ResizeParams),
    Crop(CropParams),
    Rotate(RotateParams),
    Blur(BlurParams),
    Watermark(WatermarkParams),
    WatermarkImage(WatermarkImageParams),
    Embed(EmbedParams),
    Extend(ExtendParams),
    Convert(FormatConversionParams),
    SmartCrop(SmartCropParams),
    AdjustBrightness(AdjustBrightnessParams),
    AdjustContrast(AdjustContrastParams),
    Thumbnail(ThumbnailParams),
    Extract(ExtractParams),
    Zoom(ZoomParams),
    Fit(FitParams),
    Fill(FillParams),
    Gamma(GammaParams),
    Enlarge(ResizeParams),

    // Operations without parameters
    Grayscale,
    Flip,
    Flop,
    Sharpen,
    Autorotate,
    Negate,
}
