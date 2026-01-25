use super::operations;
use super::params::Validate;
use super::pipeline_types::{PipelineOperation, PipelineOperationSpec};
use crate::http::errors::{AppError, ImageError};
use image::DynamicImage;

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
        // Log the operation type (variant name)
        let operation_name = format!("{:?}", spec.operation);
        tracing::info!(operation = %operation_name, "Starting operation");

        match execute_single_operation(image, &spec) {
            Ok(processed_image) => {
                tracing::info!(operation = %operation_name, "Operation succeeded");
                image = processed_image;
            }
            Err((returned_image, e)) => {
                tracing::error!(operation = %operation_name, error = %e, "Operation failed");
                image = returned_image;
                if spec.ignore_failure {
                    tracing::warn!(operation = %operation_name, "Operation failed but was ignored");
                } else {
                    return Err(match e {
                        ae @ AppError::BadRequest(_)
                        | ae @ AppError::ImageProcessingError(_)
                        | ae @ AppError::InvalidOperation(_) => ae,
                        _ => AppError::ImageProcessingError(format!(
                            "Error in operation {}: {}",
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
) -> Result<DynamicImage, (DynamicImage, AppError)> {
    tracing::info!(operation = ?spec.operation, "Executing single operation");

    // Helper to map errors while returning image
    let map_valid_err = |image: DynamicImage, op: &str, e: ImageError| (image, AppError::BadRequest(format!("Invalid {} params: {}", op, e)));

    match &spec.operation {
        PipelineOperation::Resize(params) => {
            if let Err(e) = params.validate() {
                return Err(map_valid_err(image, "Resize", e));
            }
            Ok(operations::resize(image, params))
        }
        PipelineOperation::Rotate(params) => {
            if let Err(e) = params.validate() {
                return Err(map_valid_err(image, "Rotate", e));
            }
            Ok(operations::rotate(image, params))
        }
        PipelineOperation::Embed(params) => {
            if let Err(e) = params.validate() {
                return Err(map_valid_err(image, "Embed", e));
            }
            Ok(operations::embed(image, params))
        }
        PipelineOperation::Crop(params) => {
            if let Err(e) = params.validate() {
                return Err(map_valid_err(image, "Crop", e));
            }
            Ok(operations::crop(image, params))
        }
        PipelineOperation::Grayscale => Ok(operations::grayscale(image)),
        PipelineOperation::Blur(params) => {
            if let Err(e) = params.validate() {
                return Err(map_valid_err(image, "Blur", e));
            }
            Ok(operations::blur(image, params))
        }
        PipelineOperation::Flip => Ok(operations::flip_vertical(image)),
        PipelineOperation::Flop => Ok(operations::flip_horizontal(image)),
        PipelineOperation::Convert(params) => {
            if let Err(e) = params.validate() {
                return Err(map_valid_err(image, "Convert", e));
            }
            match operations::convert_format(image, params) {
                Ok(img) => Ok(img),
                Err((returned_img, e)) => Err((returned_img, e)),
            }
        }
        PipelineOperation::AdjustBrightness(params) => {
            if let Err(e) = params.validate() {
                return Err(map_valid_err(image, "AdjustBrightness", e));
            }
            Ok(operations::adjust_brightness(image, params.value))
        }
        PipelineOperation::AdjustContrast(params) => {
            if let Err(e) = params.validate() {
                return Err(map_valid_err(image, "AdjustContrast", e));
            }
            Ok(operations::adjust_contrast(image, params.value))
        }
        PipelineOperation::Sharpen => Ok(operations::sharpen(image)),
        PipelineOperation::Thumbnail(params) => {
            if let Err(e) = params.validate() {
                return Err(map_valid_err(image, "Thumbnail", e));
            }
            Ok(operations::thumbnail(image, params))
        }
        PipelineOperation::Enlarge(params) => {
            if let Err(e) = params.validate() {
                return Err(map_valid_err(image, "Enlarge", e));
            }
            Ok(operations::enlarge(image, params))
        }
        PipelineOperation::Extract(params) => {
            if let Err(e) = params.validate() {
                return Err(map_valid_err(image, "Extract", e));
            }
            Ok(operations::extract(image, params))
        }
        PipelineOperation::Autorotate => Ok(operations::autorotate(image)),
        PipelineOperation::Zoom(params) => {
            if let Err(e) = params.validate() {
                return Err(map_valid_err(image, "Zoom", e));
            }
            Ok(operations::zoom(image, params))
        }
        PipelineOperation::SmartCrop(params) => {
            if let Err(e) = params.validate() {
                return Err(map_valid_err(image, "SmartCrop", e));
            }
            Ok(operations::smart_crop(image, params))
        }
        PipelineOperation::Watermark(params) => {
            if let Err(e) = params.validate() {
                return Err(map_valid_err(image, "Watermark", e));
            }
            match operations::watermark::watermark(image, params) {
                Ok(img) => Ok(img),
                Err((returned_img, e_str)) => Err((returned_img, AppError::ImageProcessingError(e_str))),
            }
        }
        PipelineOperation::WatermarkImage(params) => {
            if let Err(e) = params.validate() {
                return Err(map_valid_err(image, "WatermarkImage", e));
            }
            Ok(operations::watermark::watermark_image(image, params))
        }
        PipelineOperation::Fit(params) => {
            if let Err(e) = params.validate() {
                return Err(map_valid_err(image, "Fit", e));
            }
            Ok(operations::fit(image, params))
        }
        PipelineOperation::Fill(params) => {
            if let Err(e) = params.validate() {
                return Err(map_valid_err(image, "Fill", e));
            }
            Ok(operations::fill(image, params))
        }
        PipelineOperation::Gamma(params) => {
            if let Err(e) = params.validate() {
                return Err(map_valid_err(image, "Gamma", e));
            }
            Ok(operations::gamma(image, params))
        }
        PipelineOperation::Negate => Ok(operations::negate(image)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::params;
    use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};
    use serde_json::json;

    fn create_test_image(width: u32, height: u32) -> DynamicImage {
        DynamicImage::ImageRgba8(ImageBuffer::from_pixel(
            width,
            height,
            Rgba([255u8, 0u8, 0u8, 255u8]), // Red image
        ))
    }

    // Helper to convert JSON params to specific operation spec via deserialization
    // This allows keeping the test structure similar to before but going through the new PipelineOperationSpec deserialization
    fn create_op_spec(json: serde_json::Value) -> PipelineOperationSpec {
        serde_json::from_value(json).expect("Failed to create PipelineOperationSpec from JSON")
    }

    #[test]
    fn test_successful_pipeline() {
        let image = create_test_image(100, 100);
        let operations = vec![
            create_op_spec(json!({
                "operation": "resize",
                "params": { "width": 50, "height": 50 },
                "ignoreFailure": false
            })),
            create_op_spec(json!({
                "operation": "blur",
                "params": { "sigma": 1.0, "minampl": 0.1 },
                "ignoreFailure": false
            })),
        ];

        let result = execute_pipeline(image, operations);
        assert!(
            result.is_ok(),
            "Pipeline failed at resize or blur: {:?}",
            result
        );
        let processed = result.unwrap();
        assert_eq!(
            processed.dimensions(),
            (50, 50),
            "Resize did not produce expected dimensions"
        );
    }

    #[test]
    fn test_watermark_pipeline() {
        let image = create_test_image(100, 100);
        let operations = vec![create_op_spec(json!({
            "operation": "watermark",
            "params": {
                "text": "Test",
                "opacity": 0.5,
                "position": "Center",
                "font_size": 24,
                "color": [255, 255, 255],
                "x": null,
                "y": null
            },
            "ignoreFailure": false
        }))];

        let result = execute_pipeline(image, operations);
        if result.is_err() {
            println!("Watermark pipeline error: {:?}", result.as_ref().err());
        }
        assert!(result.is_ok(), "Watermark pipeline failed: {:?}", result);
        let processed = result.unwrap();
        assert_eq!(
            processed.dimensions(),
            (100, 100),
            "Watermark did not preserve dimensions"
        );
    }

    #[test]
    fn test_pipeline_with_ignored_failures() {
        let image = create_test_image(100, 100);
        // Note: validation happens during execution now for params validity in logic,
        // but serde might not catch business rule validation (like width > 0).
        // Wait, deserialization doesn't call validate(). validate() is called in execute_single_operation.
        // So we can still pass invalid params via JSON.

        let operations = vec![
            create_op_spec(json!({
                "operation": "resize",
                "params": {
                    "width": 0, // Invalid parameter (width=0)
                    "height": 50
                },
                "ignoreFailure": true
            })),
            create_op_spec(json!({
                "operation": "blur",
                "params": {
                    "sigma": 1.0,
                    "minampl": 0.1
                },
                "ignoreFailure": false
            })),
        ];

        // The resize should fail due to validation (width=0), but be ignored.
        let result = execute_pipeline(image, operations);
        assert!(
            result.is_ok(),
            "Pipeline with ignored failures failed: {:?}",
            result
        );
    }

    #[test]
    fn test_pipeline_error_handling() {
        let image = create_test_image(100, 100);
        let operations = vec![create_op_spec(json!({
            "operation": "resize",
            "params": {
                "width": 0, // Invalid parameter
                "height": 50
            },
            "ignoreFailure": false
        }))];

        let result = execute_pipeline(image, operations);
        assert!(
            result.is_err(),
            "Pipeline error handling did not catch error for invalid resize"
        );
    }

    #[test]
    fn test_watermark_custom_position_and_color() {
        let image = create_test_image(100, 100);
        let operations = vec![create_op_spec(json!({
            "operation": "watermark",
            "params": {
                "text": "Custom",
                "opacity": 1.0,
                "position": "TopLeft",
                "font_size": 16,
                "color": [0, 255, 0],
                "x": 5,
                "y": 5
            },
            "ignoreFailure": false
        }))];
        let result = execute_pipeline(image, operations);
        assert!(
            result.is_ok(),
            "Watermark with custom position and color failed: {:?}",
            result
        );
        let processed = result.unwrap();
        assert_eq!(
            processed.dimensions(),
            (100, 100),
            "Custom watermark did not preserve dimensions"
        );
    }

    #[test]
    fn test_watermark_invalid_params() {
        let image = create_test_image(100, 100);
        // Missing text - Serde handles basic missing fields if not default,
        // but 'text' has #[serde(default)] so it becomes empty string.
        // Then validate() checks if empty.

        let operations = vec![create_op_spec(json!({
            "operation": "watermark",
            "params": {
                "text": "", // Empty text
                "opacity": 1.0,
                "position": "TopLeft",
                "font_size": 16,
                "color": [0, 255, 0],
                "x": 5,
                "y": 5
            },
            "ignoreFailure": false
        }))];
        let result = execute_pipeline(image.clone(), operations);
        assert!(result.is_err(), "Watermark missing text should error");

        // Invalid color array (too short) - Serde will fail here if struct expects [u8; 3]
        // But if we pass [0, 255], serde might fail deserialization?
        // Yes, [u8; 3] requires 3 elements.
        // If create_op_spec panics, test fails.
        // We want to test that it fails gracefully if possible, but here we are testing execution.
        // If the request is bad JSON, the handler rejects it before execution.
        // But let's assume valid JSON structure but invalid values.

        // Negative font size - u32 cannot be negative. Serde will fail.
        // We should test logic validation.
    }

    #[test]
    fn test_pipeline_grayscale_watermark_convert() {
        let image = create_test_image(100, 100);
        let operations = vec![
            create_op_spec(json!({
                "operation": "grayscale",
                "ignoreFailure": false
            })),
            create_op_spec(json!({
                "operation": "watermark",
                "params": {
                    "text": "GrayWM",
                    "opacity": 0.8,
                    "position": "Center",
                    "font_size": 18,
                    "color": [255, 0, 0],
                },
                "ignoreFailure": false
            })),
            create_op_spec(json!({
                "operation": "convert",
                "params": {
                    "format": "jpeg",
                    "quality": 80
                },
                "ignoreFailure": false
            })),
        ];
        let result = execute_pipeline(image, operations);
        assert!(
            result.is_ok(),
            "Pipeline grayscale->watermark->convert failed: {:?}",
            result
        );
        let processed = result.unwrap();
        assert_eq!(
            processed.dimensions(),
            (100, 100),
            "Pipeline grayscale->watermark->convert did not preserve dimensions"
        );
    }

    // Additional comprehensive tests for execute_single_operation
    #[test]
    fn test_execute_single_operation_resize() {
        let image = create_test_image(100, 100);
        let spec = create_op_spec(json!({
            "operation": "resize",
            "params": {"width": 50, "height": 75},
            "ignoreFailure": false
        }));

        let result = execute_single_operation(image, &spec);
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed.dimensions(), (50, 75));
    }

    #[test]
    fn test_execute_single_operation_invalid_resize() {
        let image = create_test_image(100, 100);
        let spec = create_op_spec(json!({
            "operation": "resize",
            "params": {"width": 0, "height": 50},
            "ignoreFailure": false
        }));

        let result = execute_single_operation(image, &spec);
        assert!(result.is_err());
        let (returned_image, _err) = result.err().unwrap();
        assert_eq!(returned_image.dimensions(), (100, 100)); // Should get original image back
    }

    #[test]
    fn test_execute_single_operation_grayscale() {
        let image = create_test_image(100, 100);
        let spec = create_op_spec(json!({
            "operation": "grayscale",
            "ignoreFailure": false
        }));

        let result = execute_single_operation(image, &spec);
        assert!(result.is_ok());
    }

    #[test]
    fn test_complex_pipeline_multiple_operations() {
        let image = create_test_image(200, 200);
        let operations = vec![
            create_op_spec(json!({
                "operation": "resize",
                "params": {"width": 150, "height": 150},
                "ignoreFailure": false
            })),
            create_op_spec(json!({
                "operation": "crop",
                "params": {"x": 25, "y": 25, "width": 100, "height": 100},
                "ignoreFailure": false
            })),
            create_op_spec(json!({
                "operation": "rotate",
                "params": {"degrees": 45},
                "ignoreFailure": false
            })),
            create_op_spec(json!({
                "operation": "blur",
                "params": {"sigma": 1.5},
                "ignoreFailure": false
            })),
            create_op_spec(json!({
                "operation": "grayscale",
                "ignoreFailure": false
            })),
        ];

        let result = execute_pipeline(image, operations);
        assert!(result.is_ok(), "Complex pipeline failed: {:?}", result);
    }

    #[test]
    fn test_pipeline_new_operations() {
        let image = create_test_image(100, 50);
        let operations = vec![
            create_op_spec(json!({
                "operation": "fit",
                "params": {"width": 50, "height": 50},
                "ignoreFailure": false
            })),
            create_op_spec(json!({
                "operation": "fill",
                "params": {"width": 25, "height": 25},
                "ignoreFailure": false
            })),
            create_op_spec(json!({
                "operation": "gamma",
                "params": {"value": 2.2},
                "ignoreFailure": false
            })),
            create_op_spec(json!({
                "operation": "negate",
                "ignoreFailure": false
            })),
        ];

        let result = execute_pipeline(image, operations);
        assert!(result.is_ok(), "Pipeline with new operations failed: {:?}", result);
        let processed = result.unwrap();
        assert_eq!(processed.dimensions(), (25, 25));
    }

    #[test]
    fn test_execute_single_operation_convert_failure() {
        let image = create_test_image(100, 100);
        let spec = create_op_spec(json!({
            "operation": "convert",
            "params": {
                "format": "invalid_fmt",
            },
            "ignoreFailure": false
        }));

        let result = execute_single_operation(image, &spec);
        assert!(result.is_err());
        let (returned_image, _) = result.err().unwrap();
        assert_eq!(returned_image.dimensions(), (100, 100));
    }
}
