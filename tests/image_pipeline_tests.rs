mod helpers;

use helpers::{create_test_image, load_test_image, save_test_image};
use image::GenericImageView;
use imaginary::image::pipeline_executor::execute_pipeline;
use imaginary::image::pipeline_types::{PipelineOperationSpec, SupportedOperation};
use serde_json::json;

#[test]
fn test_complete_pipeline_with_real_image() {
    let image = load_test_image("balloons.png");
    let original_dimensions = image.dimensions();

    let operations = vec![
        PipelineOperationSpec {
            operation: SupportedOperation::Resize,
            ignore_failure: false,
            params: json!({
                "width": original_dimensions.0 / 2,
                "height": original_dimensions.1 / 2
            }),
        },
        PipelineOperationSpec {
            operation: SupportedOperation::Grayscale,
            ignore_failure: false,
            params: json!({}),
        },
        PipelineOperationSpec {
            operation: SupportedOperation::Watermark,
            ignore_failure: false,
            params: json!({
                "text": "Test Watermark",
                "opacity": 0.5,
                "position": "Center",
                "font_size": 24,
                "color": [255, 255, 255]
            }),
        },
    ];

    let result = execute_pipeline(image, operations);
    assert!(result.is_ok());
    let processed = result.unwrap();
    assert_eq!(
        processed.dimensions(),
        (original_dimensions.0 / 2, original_dimensions.1 / 2)
    );
}

#[test]
fn test_format_conversion_pipeline() {
    let image = load_test_image("balloons.png");

    let operations = vec![PipelineOperationSpec {
        operation: SupportedOperation::Convert,
        ignore_failure: false,
        params: json!({
            "format": "jpeg",
            "quality": 85
        }),
    }];

    let result = execute_pipeline(image, operations);
    assert!(result.is_ok());
}

#[test]
fn test_complex_pipeline_with_error_handling() {
    let image = load_test_image("balloons.png");
    let original_dimensions = image.dimensions();

    let operations = vec![
        // This operation should fail but be ignored
        PipelineOperationSpec {
            operation: SupportedOperation::Resize,
            ignore_failure: true,
            params: json!({
                "width": 0,  // Invalid width
                "height": original_dimensions.1 / 2
            }),
        },
        // This operation should succeed
        PipelineOperationSpec {
            operation: SupportedOperation::Grayscale,
            ignore_failure: false,
            params: json!({}),
        },
        // This operation should succeed
        PipelineOperationSpec {
            operation: SupportedOperation::Blur,
            ignore_failure: false,
            params: json!({
                "sigma": 1.0
            }),
        },
    ];

    let result = execute_pipeline(image, operations);
    assert!(result.is_ok());
    let processed = result.unwrap();
    // Image should maintain original dimensions since resize failed but was ignored
    assert_eq!(processed.dimensions(), original_dimensions);
}

#[test]
fn test_pipeline_with_different_image_formats() {
    // Test with TIFF image
    let tiff_image = load_test_image("body1.tif");
    let operations = vec![
        PipelineOperationSpec {
            operation: SupportedOperation::Resize,
            ignore_failure: false,
            params: json!({
                "width": 100,
                "height": 100
            }),
        },
        PipelineOperationSpec {
            operation: SupportedOperation::Convert,
            ignore_failure: false,
            params: json!({
                "format": "png",
                "quality": 90
            }),
        },
    ];

    let result = execute_pipeline(tiff_image, operations);
    assert!(result.is_ok());
    let processed = result.unwrap();
    assert_eq!(processed.dimensions(), (100, 100));
}

#[test]
fn test_pipeline_with_rotation_and_blur() {
    let image = load_test_image("balloons.png");
    let original_dimensions = image.dimensions();

    let operations = vec![
        PipelineOperationSpec {
            operation: SupportedOperation::Rotate,
            ignore_failure: false,
            params: json!({
                "degrees": 90
            }),
        },
        PipelineOperationSpec {
            operation: SupportedOperation::Blur,
            ignore_failure: false,
            params: json!({
                "sigma": 2.0
            }),
        },
    ];

    let result = execute_pipeline(image, operations);
    assert!(result.is_ok());
    let processed = result.unwrap();
    // After 90-degree rotation, dimensions should be swapped
    assert_eq!(
        processed.dimensions(),
        (original_dimensions.1, original_dimensions.0)
    );
}

#[test]
fn test_resize_pipeline() {
    let image = create_test_image(100, 100);
    let operations = vec![PipelineOperationSpec {
        operation: SupportedOperation::Resize,
        ignore_failure: false,
        params: json!({
            "width": 50,
            "height": 50
        }),
    }];

    let result = execute_pipeline(image, operations);
    assert!(result.is_ok());

    let processed = result.unwrap();
    assert_eq!(processed.dimensions(), (50, 50));
}

#[test]
fn test_blur_pipeline() {
    let image = create_test_image(100, 100);
    let operations = vec![PipelineOperationSpec {
        operation: SupportedOperation::Blur,
        ignore_failure: false,
        params: json!({
            "sigma": 1.0
        }),
    }];

    let result = execute_pipeline(image, operations);
    assert!(result.is_ok());
}

#[test]
fn test_complex_pipeline() {
    let image = load_test_image("balloons.png");
    let original_dimensions = image.dimensions();

    let operations = vec![
        PipelineOperationSpec {
            operation: SupportedOperation::Resize,
            ignore_failure: false,
            params: json!({
                "width": original_dimensions.0 / 2,
                "height": original_dimensions.1 / 2
            }),
        },
        PipelineOperationSpec {
            operation: SupportedOperation::Blur,
            ignore_failure: false,
            params: json!({
                "sigma": 0.5
            }),
        },
        PipelineOperationSpec {
            operation: SupportedOperation::Rotate,
            ignore_failure: false,
            params: json!({
                "degrees": 90.0
            }),
        },
    ];

    let result = execute_pipeline(image, operations);
    assert!(result.is_ok());

    let processed = result.unwrap();
    // After 90-degree rotation, dimensions should be swapped
    assert_eq!(
        processed.dimensions(),
        (original_dimensions.1 / 2, original_dimensions.0 / 2)
    );

    // Save the result for manual inspection if needed
    save_test_image(&processed, "complex_pipeline_result.png").unwrap();
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
                "sigma": 1.0
            }),
        },
    ];

    let result = execute_pipeline(image, operations);
    assert!(result.is_ok()); // Should succeed because first failure is ignored
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
    assert!(result.is_err());
}
