mod helpers;

use helpers::{create_test_image, load_test_image, save_test_image};
use image::GenericImageView;
use imaginary::image::pipeline_executor::execute_pipeline;
use imaginary::image::pipeline_types::PipelineOperationSpec;
use serde_json::json;

// Helper to convert JSON params to specific operation spec via deserialization
fn create_op_spec(json: serde_json::Value) -> PipelineOperationSpec {
    serde_json::from_value(json).expect("Failed to create PipelineOperationSpec from JSON")
}

#[test]
fn test_complete_pipeline_with_real_image() {
    let image = load_test_image("balloons.png");
    let original_dimensions = image.dimensions();

    let operations = vec![
        create_op_spec(json!({
            "operation": "resize",
            "params": {
                "width": original_dimensions.0 / 2,
                "height": original_dimensions.1 / 2
            },
            "ignoreFailure": false
        })),
        create_op_spec(json!({
            "operation": "grayscale",
            "ignoreFailure": false
        })),
        create_op_spec(json!({
            "operation": "watermark",
            "params": {
                "text": "Test Watermark",
                "opacity": 0.5,
                "position": "Center",
                "font_size": 24,
                "color": [255, 255, 255]
            },
            "ignoreFailure": false
        })),
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

    let operations = vec![create_op_spec(json!({
        "operation": "convert",
        "params": {
            "format": "jpeg",
            "quality": 85
        },
        "ignoreFailure": false
    }))];

    let result = execute_pipeline(image, operations);
    assert!(result.is_ok());
}

#[test]
fn test_complex_pipeline_with_error_handling() {
    let image = load_test_image("balloons.png");
    let original_dimensions = image.dimensions();

    let operations = vec![
        // This operation should fail but be ignored
        create_op_spec(json!({
            "operation": "resize",
            "params": {
                "width": 0,  // Invalid width
                "height": original_dimensions.1 / 2
            },
            "ignoreFailure": true
        })),
        // This operation should succeed
        create_op_spec(json!({
            "operation": "grayscale",
            "ignoreFailure": false
        })),
        // This operation should succeed
        create_op_spec(json!({
            "operation": "blur",
            "params": {
                "sigma": 1.0
            },
            "ignoreFailure": false
        })),
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
        create_op_spec(json!({
            "operation": "resize",
            "params": {
                "width": 100,
                "height": 100
            },
            "ignoreFailure": false
        })),
        create_op_spec(json!({
            "operation": "convert",
            "params": {
                "format": "png",
                "quality": 90
            },
            "ignoreFailure": false
        })),
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
        create_op_spec(json!({
            "operation": "rotate",
            "params": {
                "degrees": 90
            },
            "ignoreFailure": false
        })),
        create_op_spec(json!({
            "operation": "blur",
            "params": {
                "sigma": 2.0
            },
            "ignoreFailure": false
        })),
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
    let operations = vec![create_op_spec(json!({
        "operation": "resize",
        "params": {
            "width": 50,
            "height": 50
        },
        "ignoreFailure": false
    }))];

    let result = execute_pipeline(image, operations);
    assert!(result.is_ok());

    let processed = result.unwrap();
    assert_eq!(processed.dimensions(), (50, 50));
}

#[test]
fn test_blur_pipeline() {
    let image = create_test_image(100, 100);
    let operations = vec![create_op_spec(json!({
        "operation": "blur",
        "params": {
            "sigma": 1.0
        },
        "ignoreFailure": false
    }))];

    let result = execute_pipeline(image, operations);
    assert!(result.is_ok());
}

#[test]
fn test_complex_pipeline() {
    let image = load_test_image("balloons.png");
    let original_dimensions = image.dimensions();

    let operations = vec![
        create_op_spec(json!({
            "operation": "resize",
            "params": {
                "width": original_dimensions.0 / 2,
                "height": original_dimensions.1 / 2
            },
            "ignoreFailure": false
        })),
        create_op_spec(json!({
            "operation": "blur",
            "params": {
                "sigma": 0.5
            },
            "ignoreFailure": false
        })),
        create_op_spec(json!({
            "operation": "rotate",
            "params": {
                "degrees": 90.0
            },
            "ignoreFailure": false
        })),
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
        create_op_spec(json!({
            "operation": "resize",
            "params": {
                "width": 0, // Invalid parameter (0 is invalid for width)
                "height": 50
            },
            "ignoreFailure": true
        })),
        create_op_spec(json!({
            "operation": "blur",
            "params": {
                "sigma": 1.0
            },
            "ignoreFailure": false
        })),
    ];

    let result = execute_pipeline(image, operations);
    assert!(result.is_ok()); // Should succeed because first failure is ignored
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
    assert!(result.is_err());
}
