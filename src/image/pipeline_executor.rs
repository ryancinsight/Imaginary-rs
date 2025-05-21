use super::operations;
use super::params::{self, Validate};
use super::pipeline_types::{PipelineOperationSpec, SupportedOperation};
use crate::http::errors::{AppError, ImageError};
use image::DynamicImage;
use serde_json::Value;
use image::GenericImageView;
use image::{ImageBuffer, Rgba};

/// Executes a sequence of image operations (pipeline) on the given image.
///
/// # Arguments
/// * `image` - The input image to process.
/// * `operations_spec` - A vector of pipeline operation specifications.
///
/// # Returns
/// * `Ok(DynamicImage)` with the processed image if all operations succeed (or failures are ignored).
/// * `Err(AppError)` if a non-ignored operation fails.
pub fn execute_pipeline(
    mut image: DynamicImage,
    operations_spec: Vec<PipelineOperationSpec>,
) -> Result<DynamicImage, AppError> {
    for spec in operations_spec {
        let operation_name = spec.operation.clone(); // For logging/error messages
        tracing::info!(operation = ?operation_name, params = ?spec.params, "Starting operation");
        match execute_single_operation(image.clone(), &spec) {
            Ok(processed_image) => {
                tracing::info!(operation = ?operation_name, "Operation succeeded");
                image = processed_image;
            }
            Err(e) => {
                tracing::error!(operation = ?operation_name, params = ?spec.params, error = %e, "Operation failed");
                if spec.ignore_failure {
                    tracing::warn!(operation = ?operation_name, "Operation failed but was ignored");
                } else {
                    return Err(match e {
                        ae @ AppError::BadRequest(_) |
                        ae @ AppError::ImageProcessingError(_) |
                        ae @ AppError::InvalidOperation(_) => ae,
                        _ => AppError::ImageProcessingError(format!(
                            "Error in operation {:?}: {}",
                            operation_name, e
                        )),
                    });
                }
            }
        }
    }
    tracing::info!("Pipeline execution complete");
    Ok(image)
}

fn execute_single_operation(
    image: DynamicImage,
    spec: &PipelineOperationSpec,
) -> Result<DynamicImage, AppError> {
    tracing::info!(operation = ?spec.operation, params = ?spec.params, "Executing single operation");
    match spec.operation {
        SupportedOperation::Resize => {
            let params: params::ResizeParams = parse_params(&spec.params, "Resize")?;
            params.validate().map_err(|e: ImageError| {
                AppError::BadRequest(format!("Invalid Resize params: {}", e))
            })?;
            Ok(operations::resize(image, &params))
        }
        SupportedOperation::Rotate => {
            let params: params::RotateParams = parse_params(&spec.params, "Rotate")?;
            params.validate().map_err(|e: ImageError| {
                AppError::BadRequest(format!("Invalid Rotate params: {}", e))
            })?;
            Ok(operations::rotate(image, &params))
        }
        SupportedOperation::Crop => {
            let params: params::CropParams = parse_params(&spec.params, "Crop")?;
            params.validate().map_err(|e: ImageError| {
                AppError::BadRequest(format!("Invalid Crop params: {}", e))
            })?;
            Ok(operations::crop(image, &params))
        }
        SupportedOperation::Grayscale => Ok(operations::grayscale(image)),
        SupportedOperation::Blur => {
            let params: params::BlurParams = parse_params(&spec.params, "Blur")?;
            params.validate().map_err(|e: ImageError| {
                AppError::BadRequest(format!("Invalid Blur params: {}", e))
            })?;
            Ok(operations::blur(image, &params))
        }
        SupportedOperation::Flip => Ok(operations::flip_vertical(image)),
        SupportedOperation::Flop => Ok(operations::flip_horizontal(image)),
        SupportedOperation::Convert => {
            let params: params::FormatConversionParams = parse_params(&spec.params, "Convert")?;
            params.validate().map_err(|e: ImageError| {
                AppError::BadRequest(format!("Invalid Convert params: {}", e))
            })?;
            operations::convert_format(image, &params) // Returns Result<DynamicImage, AppError>
        }
        SupportedOperation::AdjustBrightness => {
            let params: params::AdjustBrightnessParams = parse_params(&spec.params, "AdjustBrightness")?;
            params.validate().map_err(|e: ImageError| AppError::BadRequest(format!("Invalid AdjustBrightness params: {}", e)))?;
            Ok(operations::adjust_brightness(image, params.value))
        }
        SupportedOperation::AdjustContrast => {
            let params: params::AdjustContrastParams = parse_params(&spec.params, "AdjustContrast")?;
            params.validate().map_err(|e: ImageError| AppError::BadRequest(format!("Invalid AdjustContrast params: {}", e)))?;
            Ok(operations::adjust_contrast(image, params.value))
        }
        SupportedOperation::Sharpen => Ok(operations::sharpen(image)),
        SupportedOperation::Thumbnail => {
            let params: params::ThumbnailParams = parse_params(&spec.params, "Thumbnail")?;
            params.validate().map_err(|e: ImageError| AppError::BadRequest(format!("Invalid Thumbnail params: {}", e)))?;
            Ok(operations::thumbnail(image, &params))
        }
        SupportedOperation::Enlarge => {
            // Enlarge uses ResizeParams, but only allows upscaling
            let params: params::ResizeParams = parse_params(&spec.params, "Enlarge")?;
            params.validate().map_err(|e: ImageError| AppError::BadRequest(format!("Invalid Enlarge params: {}", e)))?;
            Ok(operations::enlarge(image, &params))
        }
        SupportedOperation::Extract => {
            let params: params::ExtractParams = parse_params(&spec.params, "Extract")?;
            params.validate().map_err(|e: ImageError| AppError::BadRequest(format!("Invalid Extract params: {}", e)))?;
            Ok(operations::extract(image, &params))
        }
        SupportedOperation::Autorotate => {
            Ok(operations::autorotate(image))
        }
        SupportedOperation::Zoom => {
            let params: params::ZoomParams = parse_params(&spec.params, "Zoom")?;
            params.validate().map_err(|e: ImageError| AppError::BadRequest(format!("Invalid Zoom params: {}", e)))?;
            Ok(operations::zoom(image, &params))
        }
        SupportedOperation::SmartCrop => {
            let params: params::SmartCropParams = parse_params(&spec.params, "SmartCrop")?;
            params.validate().map_err(|e: ImageError| AppError::BadRequest(format!("Invalid SmartCrop params: {}", e)))?;
            Ok(operations::smart_crop(image, &params))
        }
        SupportedOperation::Watermark => {
            let params: params::WatermarkParams = parse_params(&spec.params, "Watermark")?;
            params.validate().map_err(|e: ImageError| {
                AppError::BadRequest(format!("Invalid Watermark params: {}", e))
            })?;
            operations::watermark::watermark(&image, &params)
                .map_err(|e| AppError::ImageProcessingError(e))
        }
        SupportedOperation::WatermarkImage => {
            let params: params::WatermarkImageParams = parse_params(&spec.params, "WatermarkImage")?;
            params.validate().map_err(|e: ImageError| AppError::BadRequest(format!("Invalid WatermarkImage params: {}", e)))?;
            Ok(operations::watermark::watermark_image(image, &params))
        }
        // Catch any other future variants if SupportedOperation enum expands beyond these
        // _ => Err(AppError::InvalidOperation(format!(
        //     "Unknown or unsupported operation: {:?}.",
        //     spec.operation
        // ))),
    }
}

fn parse_params<T: serde::de::DeserializeOwned>(
    value: &Value,
    op_name: &str,
) -> Result<T, AppError> {
    serde_json::from_value(value.clone()).map_err(|e| {
        AppError::BadRequest(format!(
            "Failed to parse parameters for {} operation: {}. Value: {}",
            op_name, e, value
        ))
    })
}

// TODO: Add unit tests for execute_pipeline and execute_single_operation
// - Test successful pipeline with multiple operations
// - Test pipeline with an operation that fails (with and without ignore_failure)
// - Test parsing of valid and invalid params for each supported operation
// - Test unimplemented operations

#[cfg(test)]
mod tests {
    use super::*;
    use image::DynamicImage;
    use serde_json::json;

    fn create_test_image(width: u32, height: u32) -> DynamicImage {
        DynamicImage::ImageRgba8(ImageBuffer::from_pixel(
            width,
            height,
            Rgba([255u8, 0u8, 0u8, 255u8]), // Red image
        ))
    }

    #[test]
    fn test_successful_pipeline() {
        let image = create_test_image(100, 100);
        let operations = vec![
            PipelineOperationSpec {
                operation: SupportedOperation::Resize,
                ignore_failure: false,
                params: json!({
                    "width": 50,
                    "height": 50
                }),
            },
            PipelineOperationSpec {
                operation: SupportedOperation::Blur,
                ignore_failure: false,
                params: json!({
                    "sigma": 1.0,
                    "minampl": 0.1
                }),
            },
        ];

        let result = execute_pipeline(image, operations);
        assert!(result.is_ok(), "Pipeline failed at resize or blur: {:?}", result);
        let processed = result.unwrap();
        assert_eq!(processed.dimensions(), (50, 50), "Resize did not produce expected dimensions");
    }

    #[test]
    fn test_watermark_pipeline() {
        let image = create_test_image(100, 100);
        let operations = vec![
            PipelineOperationSpec {
                operation: SupportedOperation::Watermark,
                ignore_failure: false,
                params: json!({
                    "text": "Test",
                    "opacity": 0.5,
                    "position": "Center",
                    "font_size": 24,
                    "color": [255, 255, 255],
                    "x": null,
                    "y": null
                }),
            },
        ];

        let result = execute_pipeline(image, operations);
        if result.is_err() {
            println!("Watermark pipeline error: {:?}", result.as_ref().err());
        }
        assert!(result.is_ok(), "Watermark pipeline failed: {:?}", result);
        let processed = result.unwrap();
        assert_eq!(processed.dimensions(), (100, 100), "Watermark did not preserve dimensions");
    }

    #[test]
    fn test_pipeline_with_ignored_failures() {
        let image = create_test_image(100, 100);
        let operations = vec![
            PipelineOperationSpec {
                operation: SupportedOperation::Resize,
                ignore_failure: true,
                params: json!({
                    "width": -50, // Invalid parameter
                    "height": 50
                }),
            },
            PipelineOperationSpec {
                operation: SupportedOperation::Blur,
                ignore_failure: false,
                params: json!({
                    "sigma": 1.0,
                    "minampl": 0.1
                }),
            },
        ];

        let result = execute_pipeline(image, operations);
        assert!(result.is_ok(), "Pipeline with ignored failures failed: {:?}", result);
    }

    #[test]
    fn test_pipeline_error_handling() {
        let image = create_test_image(100, 100);
        let operations = vec![
            PipelineOperationSpec {
                operation: SupportedOperation::Resize,
                ignore_failure: false,
                params: json!({
                    "width": -50, // Invalid parameter
                    "height": 50
                }),
            },
        ];

        let result = execute_pipeline(image, operations);
        assert!(result.is_err(), "Pipeline error handling did not catch error for invalid resize");
    }

    #[test]
    fn test_watermark_custom_position_and_color() {
        let image = create_test_image(100, 100);
        let operations = vec![
            PipelineOperationSpec {
                operation: SupportedOperation::Watermark,
                ignore_failure: false,
                params: json!({
                    "text": "Custom",
                    "opacity": 1.0,
                    "position": "TopLeft",
                    "font_size": 16,
                    "color": [0, 255, 0],
                    "x": 5,
                    "y": 5
                }),
            },
        ];
        let result = execute_pipeline(image, operations);
        assert!(result.is_ok(), "Watermark with custom position and color failed: {:?}", result);
        let processed = result.unwrap();
        assert_eq!(processed.dimensions(), (100, 100), "Custom watermark did not preserve dimensions");
    }

    #[test]
    fn test_watermark_invalid_params() {
        let image = create_test_image(100, 100);
        // Missing text
        let operations = vec![
            PipelineOperationSpec {
                operation: SupportedOperation::Watermark,
                ignore_failure: false,
                params: json!({
                    "opacity": 1.0,
                    "position": "TopLeft",
                    "font_size": 16,
                    "color": [0, 255, 0],
                    "x": 5,
                    "y": 5
                }),
            },
        ];
        let result = execute_pipeline(image.clone(), operations);
        assert!(result.is_err(), "Watermark missing text should error");

        // Invalid color array (too short)
        let operations = vec![
            PipelineOperationSpec {
                operation: SupportedOperation::Watermark,
                ignore_failure: false,
                params: json!({
                    "text": "BadColor",
                    "opacity": 1.0,
                    "position": "TopLeft",
                    "font_size": 16,
                    "color": [0, 255],
                    "x": 5,
                    "y": 5
                }),
            },
        ];
        let result = execute_pipeline(image.clone(), operations);
        assert!(result.is_err(), "Watermark with invalid color should error");

        // Negative font size
        let operations = vec![
            PipelineOperationSpec {
                operation: SupportedOperation::Watermark,
                ignore_failure: false,
                params: json!({
                    "text": "NegativeFont",
                    "opacity": 1.0,
                    "position": "TopLeft",
                    "font_size": -10,
                    "color": [0, 255, 0],
                    "x": 5,
                    "y": 5
                }),
            },
        ];
        let result = execute_pipeline(image, operations);
        assert!(result.is_err(), "Watermark with negative font size should error");
    }

    #[test]
    fn test_pipeline_grayscale_watermark_convert() {
        let image = create_test_image(100, 100);
        let operations = vec![
            PipelineOperationSpec {
                operation: SupportedOperation::Grayscale,
                ignore_failure: false,
                params: json!({}),
            },
            PipelineOperationSpec {
                operation: SupportedOperation::Watermark,
                ignore_failure: false,
                params: json!({
                    "text": "GrayWM",
                    "opacity": 0.8,
                    "position": "Center",
                    "font_size": 18,
                    "color": [255, 0, 0],
                }),
            },
            PipelineOperationSpec {
                operation: SupportedOperation::Convert,
                ignore_failure: false,
                params: json!({
                    "format": "jpeg",
                    "quality": 80
                }),
            },
        ];
        let result = execute_pipeline(image, operations);
        assert!(result.is_ok(), "Pipeline grayscale->watermark->convert failed: {:?}", result);
        let processed = result.unwrap();
        assert_eq!(processed.dimensions(), (100, 100), "Pipeline grayscale->watermark->convert did not preserve dimensions");
    }
}