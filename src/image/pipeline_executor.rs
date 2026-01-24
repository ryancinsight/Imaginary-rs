use super::operations;
use super::params::{self, Validate};
use super::pipeline_types::{PipelineOperationSpec, SupportedOperation};
use crate::http::errors::{AppError, ImageError};
use image::DynamicImage;
use serde_json::Value;

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
        let operation_name = spec.operation; // For logging/error messages
        tracing::info!(operation = ?operation_name, params = ?spec.params, "Starting operation");
        match execute_single_operation(image, &spec) {
            Ok(processed_image) => {
                tracing::info!(operation = ?operation_name, "Operation succeeded");
                image = processed_image;
            }
            Err((returned_image, e)) => {
                tracing::error!(operation = ?operation_name, params = ?spec.params, error = %e, "Operation failed");
                image = returned_image;
                if spec.ignore_failure {
                    tracing::warn!(operation = ?operation_name, "Operation failed but was ignored");
                } else {
                    return Err(match e {
                        ae @ AppError::BadRequest(_)
                        | ae @ AppError::ImageProcessingError(_)
                        | ae @ AppError::InvalidOperation(_) => ae,
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
) -> Result<DynamicImage, (DynamicImage, AppError)> {
    tracing::info!(operation = ?spec.operation, params = ?spec.params, "Executing single operation");

    // Helper to map errors while returning image
    let map_err = |image: DynamicImage, e: AppError| (image, e);
    let map_valid_err = |image: DynamicImage, op: &str, e: ImageError| (image, AppError::BadRequest(format!("Invalid {} params: {}", op, e)));

    match spec.operation {
        SupportedOperation::Resize => {
            match parse_params::<params::ResizeParams>(&spec.params, "Resize") {
                Ok(params) => {
                    if let Err(e) = params.validate() {
                         return Err(map_valid_err(image, "Resize", e));
                    }
                    Ok(operations::resize(image, &params))
                },
                Err(e) => Err((image, e))
             }
        }
        SupportedOperation::Rotate => {
            match parse_params::<params::RotateParams>(&spec.params, "Rotate") {
                Ok(params) => {
                    if let Err(e) = params.validate() {
                        return Err(map_valid_err(image, "Rotate", e));
                    }
                    Ok(operations::rotate(image, &params))
                },
                Err(e) => Err((image, e))
            }
        }
        SupportedOperation::Embed => {
            match parse_params::<params::EmbedParams>(&spec.params, "Embed") {
                Ok(params) => {
                    if let Err(e) = params.validate() {
                        return Err(map_valid_err(image, "Embed", e));
                    }
                    Ok(operations::embed(image, &params))
                },
                Err(e) => Err((image, e))
            }
        }
        SupportedOperation::Crop => {
            match parse_params::<params::CropParams>(&spec.params, "Crop") {
                Ok(params) => {
                    if let Err(e) = params.validate() {
                        return Err(map_valid_err(image, "Crop", e));
                    }
                    Ok(operations::crop(image, &params))
                },
                Err(e) => Err((image, e))
            }
        }
        SupportedOperation::Grayscale => Ok(operations::grayscale(image)),
        SupportedOperation::Blur => {
            match parse_params::<params::BlurParams>(&spec.params, "Blur") {
                Ok(params) => {
                    if let Err(e) = params.validate() {
                        return Err(map_valid_err(image, "Blur", e));
                    }
                    Ok(operations::blur(image, &params))
                },
                Err(e) => Err((image, e))
            }
        }
        SupportedOperation::Flip => Ok(operations::flip_vertical(image)),
        SupportedOperation::Flop => Ok(operations::flip_horizontal(image)),
        SupportedOperation::Convert => {
            match parse_params::<params::FormatConversionParams>(&spec.params, "Convert") {
                Ok(params) => {
                    if let Err(e) = params.validate() {
                        return Err(map_valid_err(image, "Convert", e));
                    }
                    // convert_format returns Result<DynamicImage, AppError>
                    // We need to map Err(e) -> Err((image, e)).
                    // But convert_format consumes image? Not yet.
                    // If convert_format fails, it might have consumed image if it was designed that way.
                    // But currently I am refactoring convert_format to take ownership.
                    // If convert_format takes ownership and fails, can it return the image back?
                    // Typically 'try' operations that consume input return it on failure.
                    // I should update convert_format signature to: Result<DynamicImage, (DynamicImage, AppError)>?
                    // Or just Result<DynamicImage, AppError> and if it fails, the image is gone/partially consumed?
                    // But convert_format usually writes to a buffer. It doesn't modify image in place until it succeeds (returns new image).
                    // So if it takes 'image', it still holds it.
                    // Ideally operations that can fail should return the original image on failure if possible.

                    // For now, let's assume operations::convert_format consumes image.
                    // If it fails, we might lose the image.
                    // For 'ignore_failure' to work, we need the original image.
                    // So operations that can fail should probably take &DynamicImage or return the original image on error.
                    // `convert_format` creates a NEW image (new format). The input image is source.
                    // So we can pass &image to convert_format?
                    // But I want to pass ownership to allow reuse/drop.
                    // If I pass ownership, I can't get it back easily unless the function returns it.

                    // Strategy: For operations that *transform* (like resize), they usually succeed or we don't care about intermediate state if they fail (we can't recover).
                    // But convert_format failure (e.g. invalid quality) shouldn't destroy the image if we want to ignore failure.
                    // So convert_format should ideally take &DynamicImage if it doesn't modify in place?
                    // Or take ownership and return it back on error.

                    // Let's make convert_format return Result<DynamicImage, (DynamicImage, AppError)>.
                    // Or, since convert_format is just encoding, maybe keep it taking &DynamicImage?
                    // But I said I would refactor it to take DynamicImage.

                    // If I pass `image.clone()` to convert_format (old way), I have the original.
                    // If I pass `image` (new way), I lose it.

                    // Let's implement convert_format to return `Result<DynamicImage, (DynamicImage, AppError)>` in step 4.

                    match operations::convert_format(image, &params) {
                        Ok(img) => Ok(img),
                        Err((returned_img, e)) => Err((returned_img, e)),
                    }
                },
                Err(e) => Err((image, e))
            }
        }
        SupportedOperation::AdjustBrightness => {
            match parse_params::<params::AdjustBrightnessParams>(&spec.params, "AdjustBrightness") {
                Ok(params) => {
                    if let Err(e) = params.validate() {
                        return Err(map_valid_err(image, "AdjustBrightness", e));
                    }
                    Ok(operations::adjust_brightness(image, params.value))
                },
                Err(e) => Err((image, e))
            }
        }
        SupportedOperation::AdjustContrast => {
            match parse_params::<params::AdjustContrastParams>(&spec.params, "AdjustContrast") {
                Ok(params) => {
                    if let Err(e) = params.validate() {
                        return Err(map_valid_err(image, "AdjustContrast", e));
                    }
                    Ok(operations::adjust_contrast(image, params.value))
                },
                Err(e) => Err((image, e))
            }
        }
        SupportedOperation::Sharpen => Ok(operations::sharpen(image)),
        SupportedOperation::Thumbnail => {
            match parse_params::<params::ThumbnailParams>(&spec.params, "Thumbnail") {
                Ok(params) => {
                    if let Err(e) = params.validate() {
                        return Err(map_valid_err(image, "Thumbnail", e));
                    }
                    Ok(operations::thumbnail(image, &params))
                },
                Err(e) => Err((image, e))
            }
        }
        SupportedOperation::Enlarge => {
            match parse_params::<params::ResizeParams>(&spec.params, "Enlarge") {
                Ok(params) => {
                    if let Err(e) = params.validate() {
                        return Err(map_valid_err(image, "Enlarge", e));
                    }
                    Ok(operations::enlarge(image, &params))
                },
                Err(e) => Err((image, e))
            }
        }
        SupportedOperation::Extract => {
            match parse_params::<params::ExtractParams>(&spec.params, "Extract") {
                Ok(params) => {
                    if let Err(e) = params.validate() {
                        return Err(map_valid_err(image, "Extract", e));
                    }
                    Ok(operations::extract(image, &params))
                },
                Err(e) => Err((image, e))
            }
        }
        SupportedOperation::Autorotate => Ok(operations::autorotate(image)),
        SupportedOperation::Zoom => {
            match parse_params::<params::ZoomParams>(&spec.params, "Zoom") {
                Ok(params) => {
                    if let Err(e) = params.validate() {
                        return Err(map_valid_err(image, "Zoom", e));
                    }
                    Ok(operations::zoom(image, &params))
                },
                Err(e) => Err((image, e))
            }
        }
        SupportedOperation::SmartCrop => {
            match parse_params::<params::SmartCropParams>(&spec.params, "SmartCrop") {
                Ok(params) => {
                    if let Err(e) = params.validate() {
                        return Err(map_valid_err(image, "SmartCrop", e));
                    }
                    Ok(operations::smart_crop(image, &params))
                },
                Err(e) => Err((image, e))
            }
        }
        SupportedOperation::Watermark => {
            match parse_params::<params::WatermarkParams>(&spec.params, "Watermark") {
                Ok(params) => {
                    if let Err(e) = params.validate() {
                        return Err(map_valid_err(image, "Watermark", e));
                    }
                    // watermark returns Result<DynamicImage, String> (mapped to AppError)
                    // We need to handle failure and return image.
                    // watermark will be refactored to take ownership and return Result<DynamicImage, (DynamicImage, String)>
                    match operations::watermark::watermark(image, &params) {
                        Ok(img) => Ok(img),
                        Err((returned_img, e_str)) => Err((returned_img, AppError::ImageProcessingError(e_str))),
                    }
                },
                Err(e) => Err((image, e))
            }
        }
        SupportedOperation::WatermarkImage => {
            match parse_params::<params::WatermarkImageParams>(&spec.params, "WatermarkImage") {
                Ok(params) => {
                    if let Err(e) = params.validate() {
                        return Err(map_valid_err(image, "WatermarkImage", e));
                    }
                    Ok(operations::watermark::watermark_image(image, &params))
                },
                Err(e) => Err((image, e))
            }
        }
        SupportedOperation::Fit => {
            match parse_params::<params::FitParams>(&spec.params, "Fit") {
                Ok(params) => {
                    if let Err(e) = params.validate() {
                        return Err(map_valid_err(image, "Fit", e));
                    }
                    Ok(operations::fit(image, &params))
                },
                Err(e) => Err((image, e))
            }
        }
        SupportedOperation::Fill => {
            match parse_params::<params::FillParams>(&spec.params, "Fill") {
                Ok(params) => {
                    if let Err(e) = params.validate() {
                        return Err(map_valid_err(image, "Fill", e));
                    }
                    Ok(operations::fill(image, &params))
                },
                Err(e) => Err((image, e))
            }
        }
        SupportedOperation::Gamma => {
            match parse_params::<params::GammaParams>(&spec.params, "Gamma") {
                Ok(params) => {
                    if let Err(e) = params.validate() {
                        return Err(map_valid_err(image, "Gamma", e));
                    }
                    Ok(operations::gamma(image, &params))
                },
                Err(e) => Err((image, e))
            }
        }
        SupportedOperation::Negate => Ok(operations::negate(image)),
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

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};
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

    // ... (rest of the tests need to be preserved)
    // I will include all tests from the original file, adjusting if necessary.
    // The tests in the original file used execute_pipeline and execute_single_operation.
    // execute_single_operation tests will need to be updated because the signature changed.

    #[test]
    fn test_watermark_pipeline() {
        let image = create_test_image(100, 100);
        let operations = vec![PipelineOperationSpec {
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
        }];

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
        assert!(
            result.is_ok(),
            "Pipeline with ignored failures failed: {:?}",
            result
        );
    }

    #[test]
    fn test_pipeline_error_handling() {
        let image = create_test_image(100, 100);
        let operations = vec![PipelineOperationSpec {
            operation: SupportedOperation::Resize,
            ignore_failure: false,
            params: json!({
                "width": -50, // Invalid parameter
                "height": 50
            }),
        }];

        let result = execute_pipeline(image, operations);
        assert!(
            result.is_err(),
            "Pipeline error handling did not catch error for invalid resize"
        );
    }

    #[test]
    fn test_watermark_custom_position_and_color() {
        let image = create_test_image(100, 100);
        let operations = vec![PipelineOperationSpec {
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
        }];
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
        // Missing text
        let operations = vec![PipelineOperationSpec {
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
        }];
        let result = execute_pipeline(image.clone(), operations);
        assert!(result.is_err(), "Watermark missing text should error");

        // Invalid color array (too short)
        let operations = vec![PipelineOperationSpec {
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
        }];
        let result = execute_pipeline(image.clone(), operations);
        assert!(result.is_err(), "Watermark with invalid color should error");

        // Negative font size
        let operations = vec![PipelineOperationSpec {
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
        }];
        let result = execute_pipeline(image, operations);
        assert!(
            result.is_err(),
            "Watermark with negative font size should error"
        );
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
        let spec = PipelineOperationSpec {
            operation: SupportedOperation::Resize,
            params: json!({"width": 50, "height": 75}),
            ignore_failure: false,
        };

        let result = execute_single_operation(image, &spec);
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed.dimensions(), (50, 75));
    }

    #[test]
    fn test_execute_single_operation_invalid_resize() {
        let image = create_test_image(100, 100);
        let spec = PipelineOperationSpec {
            operation: SupportedOperation::Resize,
            params: json!({"width": -10, "height": 50}),
            ignore_failure: false,
        };

        let result = execute_single_operation(image, &spec);
        assert!(result.is_err());
        let (returned_image, _err) = result.err().unwrap();
        assert_eq!(returned_image.dimensions(), (100, 100)); // Should get original image back
    }

    #[test]
    fn test_execute_single_operation_grayscale() {
        let image = create_test_image(100, 100);
        let spec = PipelineOperationSpec {
            operation: SupportedOperation::Grayscale,
            params: json!({}),
            ignore_failure: false,
        };

        let result = execute_single_operation(image, &spec);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_single_operation_blur() {
        let image = create_test_image(100, 100);
        let spec = PipelineOperationSpec {
            operation: SupportedOperation::Blur,
            params: json!({"sigma": 2.0}),
            ignore_failure: false,
        };

        let result = execute_single_operation(image, &spec);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_single_operation_invalid_blur() {
        let image = create_test_image(100, 100);
        let spec = PipelineOperationSpec {
            operation: SupportedOperation::Blur,
            params: json!({"sigma": -1.0}), // Invalid negative sigma
            ignore_failure: false,
        };

        let result = execute_single_operation(image, &spec);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_single_operation_crop() {
        let image = create_test_image(100, 100);
        let spec = PipelineOperationSpec {
            operation: SupportedOperation::Crop,
            params: json!({"x": 10, "y": 10, "width": 50, "height": 50}),
            ignore_failure: false,
        };

        let result = execute_single_operation(image, &spec);
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed.dimensions(), (50, 50));
    }

    #[test]
    fn test_execute_single_operation_invalid_crop() {
        let image = create_test_image(100, 100);
        let spec = PipelineOperationSpec {
            operation: SupportedOperation::Crop,
            params: json!({"x": 0, "y": 0, "width": 0, "height": 50}), // zero width should fail
            ignore_failure: false,
        };

        let result = execute_single_operation(image, &spec);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_single_operation_rotate() {
        let image = create_test_image(100, 100);
        let spec = PipelineOperationSpec {
            operation: SupportedOperation::Rotate,
            params: json!({"degrees": 90}),
            ignore_failure: false,
        };

        let result = execute_single_operation(image, &spec);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_single_operation_flip() {
        let image = create_test_image(100, 100);
        let spec = PipelineOperationSpec {
            operation: SupportedOperation::Flip,
            params: json!({}),
            ignore_failure: false,
        };

        let result = execute_single_operation(image, &spec);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_single_operation_flop() {
        let image = create_test_image(100, 100);
        let spec = PipelineOperationSpec {
            operation: SupportedOperation::Flop,
            params: json!({}),
            ignore_failure: false,
        };

        let result = execute_single_operation(image, &spec);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_single_operation_adjust_brightness() {
        let image = create_test_image(100, 100);
        let spec = PipelineOperationSpec {
            operation: SupportedOperation::AdjustBrightness,
            params: json!({"value": 20}),
            ignore_failure: false,
        };

        let result = execute_single_operation(image, &spec);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_single_operation_adjust_contrast() {
        let image = create_test_image(100, 100);
        let spec = PipelineOperationSpec {
            operation: SupportedOperation::AdjustContrast,
            params: json!({"value": 1.2}),
            ignore_failure: false,
        };

        let result = execute_single_operation(image, &spec);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_single_operation_sharpen() {
        let image = create_test_image(100, 100);
        let spec = PipelineOperationSpec {
            operation: SupportedOperation::Sharpen,
            params: json!({}),
            ignore_failure: false,
        };

        let result = execute_single_operation(image, &spec);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_single_operation_convert() {
        let image = create_test_image(100, 100);
        let spec = PipelineOperationSpec {
            operation: SupportedOperation::Convert,
            params: json!({"format": "jpeg", "quality": 85}),
            ignore_failure: false,
        };

        let result = execute_single_operation(image, &spec);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_single_operation_invalid_convert() {
        let image = create_test_image(100, 100);
        let spec = PipelineOperationSpec {
            operation: SupportedOperation::Convert,
            params: json!({"format": "invalid_format"}),
            ignore_failure: false,
        };

        let result = execute_single_operation(image, &spec);
        assert!(result.is_err());
    }

    #[test]
    fn test_complex_pipeline_multiple_operations() {
        let image = create_test_image(200, 200);
        let operations = vec![
            PipelineOperationSpec {
                operation: SupportedOperation::Resize,
                ignore_failure: false,
                params: json!({"width": 150, "height": 150}),
            },
            PipelineOperationSpec {
                operation: SupportedOperation::Crop,
                ignore_failure: false,
                params: json!({"x": 25, "y": 25, "width": 100, "height": 100}),
            },
            PipelineOperationSpec {
                operation: SupportedOperation::Rotate,
                ignore_failure: false,
                params: json!({"degrees": 45}),
            },
            PipelineOperationSpec {
                operation: SupportedOperation::Blur,
                ignore_failure: false,
                params: json!({"sigma": 1.5}),
            },
            PipelineOperationSpec {
                operation: SupportedOperation::Grayscale,
                ignore_failure: false,
                params: json!({}),
            },
        ];

        let result = execute_pipeline(image, operations);
        assert!(result.is_ok(), "Complex pipeline failed: {:?}", result);
    }

    #[test]
    fn test_pipeline_mixed_success_and_ignored_failures() {
        let image = create_test_image(100, 100);
        let operations = vec![
            PipelineOperationSpec {
                operation: SupportedOperation::Resize,
                ignore_failure: false,
                params: json!({"width": 80, "height": 80}),
            },
            PipelineOperationSpec {
                operation: SupportedOperation::Crop,
                ignore_failure: true, // This will be ignored if it fails
                params: json!({"x": 0, "y": 0, "width": 0, "height": 50}), // zero width should fail
            },
            PipelineOperationSpec {
                operation: SupportedOperation::Blur,
                ignore_failure: false,
                params: json!({"sigma": 1.0}),
            },
        ];

        let result = execute_pipeline(image, operations);
        assert!(
            result.is_ok(),
            "Pipeline with mixed success/ignored failures failed: {:?}",
            result
        );
        let processed = result.unwrap();
        assert_eq!(processed.dimensions(), (80, 80)); // Should have resize dimensions
    }

    #[test]
    fn test_pipeline_all_operations_ignored() {
        let image = create_test_image(100, 100);
        let operations = vec![
            PipelineOperationSpec {
                operation: SupportedOperation::Crop,
                ignore_failure: true,
                params: json!({"x": 0, "y": 0, "width": 0, "height": 50}), // zero width should fail
            },
            PipelineOperationSpec {
                operation: SupportedOperation::Resize,
                ignore_failure: true,
                params: json!({"width": 0, "height": 50}), // zero width should fail
            },
        ];

        let result = execute_pipeline(image, operations);
        assert!(
            result.is_ok(),
            "Pipeline with all ignored failures should succeed"
        );
        let processed = result.unwrap();
        assert_eq!(processed.dimensions(), (100, 100)); // Should maintain original dimensions
    }

    #[test]
    fn test_parse_operation_params_valid() {
        use crate::image::params::ResizeParams;
        let params = json!({"width": 100, "height": 200});
        let result: Result<ResizeParams, AppError> = parse_params(&params, "resize");
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.width, 100);
        assert_eq!(parsed.height, 200);
    }

    #[test]
    fn test_parse_operation_params_invalid() {
        use crate::image::params::ResizeParams;
        let params = json!({"width": "not_a_number", "height": 200});
        let result: Result<ResizeParams, AppError> = parse_params(&params, "resize");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_operation_params_missing_fields() {
        use crate::image::params::ResizeParams;
        let params = json!({}); // Missing both width and height, should use defaults then validate
        let result: Result<ResizeParams, AppError> = parse_params(&params, "resize");
        assert!(result.is_ok()); // Should succeed with defaults
        let parsed = result.unwrap();
        assert_eq!(parsed.width, 100); // default value
        assert_eq!(parsed.height, 100); // default value
    }

    #[test]
    fn test_pipeline_empty_operations() {
        let image = create_test_image(100, 100);
        let operations = vec![];

        let result = execute_pipeline(image, operations);
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed.dimensions(), (100, 100)); // Should be unchanged
    }

    #[test]
    fn test_pipeline_new_operations() {
        let image = create_test_image(100, 50);
        let operations = vec![
            PipelineOperationSpec {
                operation: SupportedOperation::Fit,
                ignore_failure: false,
                params: json!({"width": 50, "height": 50}),
            },
            PipelineOperationSpec {
                operation: SupportedOperation::Fill,
                ignore_failure: false,
                params: json!({"width": 25, "height": 25}),
            },
            PipelineOperationSpec {
                operation: SupportedOperation::Gamma,
                ignore_failure: false,
                params: json!({"value": 2.2}),
            },
            PipelineOperationSpec {
                operation: SupportedOperation::Negate,
                ignore_failure: false,
                params: json!({}),
            },
        ];

        let result = execute_pipeline(image, operations);
        assert!(result.is_ok(), "Pipeline with new operations failed: {:?}", result);
        let processed = result.unwrap();
        assert_eq!(processed.dimensions(), (25, 25));
    }
}
