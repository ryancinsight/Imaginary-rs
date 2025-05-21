//! Types and enums for describing image processing pipelines and supported operations.
//!
//! This module defines the data structures used to specify a sequence of image operations (pipeline)
//! and the set of operations supported by the pipeline executor.

use serde::Deserialize;
use serde_json::Value;

// Add other necessary imports if/when they become clear.
// For now, params.rs might be needed for actual parameter structs,
// but we\'ll handle dynamic dispatch first.
// use super::params::*; // Example if params were directly embedded

/// Specification for a single operation in an image processing pipeline.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PipelineOperationSpec {
    /// The operation to perform.
    pub operation: SupportedOperation,
    /// If true, ignore failure of this operation and continue the pipeline.
    #[serde(default)]
    pub ignore_failure: bool,
    /// Parameters for the operation (operation-specific, dynamic).
    #[serde(default)]
    pub params: Value, // Using serde_json::Value for dynamic params
}

/// Enum of all supported image operations for the pipeline.
#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum SupportedOperation {
    Crop,
    SmartCrop,
    Resize,
    Enlarge,
    Extract,
    Rotate,
    Autorotate,
    Flip,
    Flop,
    Thumbnail,
    Zoom,
    Convert,
    Watermark,
    WatermarkImage,
    Blur,
    Grayscale, // Added from existing imaginary-rs operations
    AdjustBrightness, // Added from existing imaginary-rs operations
    AdjustContrast, // Added from existing imaginary-rs operations
    Sharpen, // Added from existing imaginary-rs operations
    // Add other operations as they are implemented and supported in pipeline
}

// Consider adding a method to PipelineOperationSpec to try and parse `params`
// into a specific operation\'s parameter struct.
// e.g., impl PipelineOperationSpec {
//     pub fn try_into_resize_params(&self) -> Result<ResizeParams, serde_json::Error> {
//         serde_json::from_value(self.params.clone())
//     }
// }
// This would require specific knowledge of param structs here, or a more generic approach. 