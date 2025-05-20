use super::operations;
use super::params::{self, Validate};
use super::pipeline_types::{PipelineOperationSpec, SupportedOperation};
use crate::http::errors::{AppError, ImageError};
use image::DynamicImage;
use serde_json::Value;

pub fn execute_pipeline(
    mut image: DynamicImage,
    operations_spec: Vec<PipelineOperationSpec>,
) -> Result<DynamicImage, AppError> {
    for spec in operations_spec {
        let operation_name = spec.operation.clone(); // For logging/error messages
        match execute_single_operation(image.clone(), &spec) {
            Ok(processed_image) => {
                image = processed_image;
            }
            Err(e) => {
                if spec.ignore_failure {
                    tracing::warn!(
                        "Operation {:?} failed but was ignored: {:?}",
                        operation_name,
                        e
                    );
                } else {
                    return Err(match e {
                        // If it's already a well-formed AppError from deeper, pass it up.
                        // Otherwise, wrap it with context.
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
    Ok(image)
}

fn execute_single_operation(
    image: DynamicImage,
    spec: &PipelineOperationSpec,
) -> Result<DynamicImage, AppError> {
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

        // Explicitly mark unimplemented operations from h2non/imaginary spec
        op @ SupportedOperation::SmartCrop |
        op @ SupportedOperation::Enlarge |
        op @ SupportedOperation::Extract |
        op @ SupportedOperation::Autorotate |
        op @ SupportedOperation::Thumbnail |
        op @ SupportedOperation::Zoom |
        op @ SupportedOperation::Watermark |
        op @ SupportedOperation::WatermarkImage => {
            Err(AppError::InvalidOperation(format!(
                "Operation {:?} is not yet implemented.",
                op
            )))
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