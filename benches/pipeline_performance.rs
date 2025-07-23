use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use imaginary::image::pipeline_executor::execute_pipeline;
use imaginary::image::pipeline_types::{PipelineOperationSpec, SupportedOperation};
use image::{DynamicImage, ImageBuffer, RgbImage};
use serde_json::json;
use std::thread;
use std::sync::Arc;

// Create test image data for benchmarking
fn create_test_image(width: u32, height: u32) -> DynamicImage {
    let img: RgbImage = ImageBuffer::from_fn(width, height, |x, y| {
        image::Rgb([
            (x % 256) as u8,
            (y % 256) as u8,
            ((x + y) % 256) as u8,
        ])
    });
    DynamicImage::ImageRgb8(img)
}

// Benchmark pipeline processing with different operation counts
fn bench_pipeline_operations_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("pipeline_operations_count");
    
    let test_image = create_test_image(800, 600);
    
    // Different complexity levels
    let operation_sets = vec![
        (1, "single_operation", vec![
            PipelineOperationSpec {
                operation: SupportedOperation::Resize,
                params: json!({"width": 400, "height": 300}),
                ignore_failure: false,
            },
        ]),
        (3, "three_operations", vec![
            PipelineOperationSpec {
                operation: SupportedOperation::Resize,
                params: json!({"width": 400, "height": 300}),
                ignore_failure: false,
            },
            PipelineOperationSpec {
                operation: SupportedOperation::Grayscale,
                params: json!({}),
                ignore_failure: false,
            },
            PipelineOperationSpec {
                operation: SupportedOperation::Blur,
                params: json!({"sigma": 1.0}),
                ignore_failure: false,
            },
        ]),
        (5, "five_operations", vec![
            PipelineOperationSpec {
                operation: SupportedOperation::Resize,
                params: json!({"width": 600, "height": 400}),
                ignore_failure: false,
            },
            PipelineOperationSpec {
                operation: SupportedOperation::Crop,
                params: json!({"x": 50, "y": 50, "width": 500, "height": 300}),
                ignore_failure: false,
            },
            PipelineOperationSpec {
                operation: SupportedOperation::Rotate,
                params: json!({"degrees": 90.0}),
                ignore_failure: false,
            },
            PipelineOperationSpec {
                operation: SupportedOperation::AdjustBrightness,
                params: json!({"value": 10}),
                ignore_failure: false,
            },
            PipelineOperationSpec {
                operation: SupportedOperation::Sharpen,
                params: json!({}),
                ignore_failure: false,
            },
        ]),
    ];
    
    for (_count, name, operations) in operation_sets {
        group.bench_with_input(
            BenchmarkId::new("pipeline_processing", name),
            &operations,
            |b, ops| {
                b.iter(|| {
                    black_box(execute_pipeline(
                        black_box(test_image.clone()),
                        black_box(ops.clone()),
                    ))
                })
            },
        );
    }
    
    group.finish();
}

// Benchmark memory usage patterns
fn bench_memory_usage_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage_patterns");
    
    // Test with different image sizes to understand memory scaling
    let image_sizes = vec![
        (200, 150, "tiny"),
        (800, 600, "small"),
        (1920, 1080, "medium"),
        (3840, 2160, "large"),
    ];
    
    let operations = vec![
        PipelineOperationSpec {
            operation: SupportedOperation::Resize,
            params: json!({"width": 400, "height": 300}),
            ignore_failure: false,
        },
        PipelineOperationSpec {
            operation: SupportedOperation::Grayscale,
            params: json!({}),
            ignore_failure: false,
        },
        PipelineOperationSpec {
            operation: SupportedOperation::Blur,
            params: json!({"sigma": 2.0}),
            ignore_failure: false,
        },
    ];
    
    for (width, height, size_name) in image_sizes {
        let test_image = create_test_image(width, height);
        
        group.bench_with_input(
            BenchmarkId::new("memory_scaling", size_name),
            &test_image,
            |b, img| {
                b.iter(|| {
                    black_box(execute_pipeline(
                        black_box(img.clone()),
                        black_box(operations.clone()),
                    ))
                })
            },
        );
    }
    
    group.finish();
}

// Benchmark concurrent pipeline processing
fn bench_concurrent_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_processing");
    
    let test_image = Arc::new(create_test_image(800, 600));
    let operations = Arc::new(vec![
        PipelineOperationSpec {
            operation: SupportedOperation::Resize,
            params: json!({"width": 400, "height": 300}),
            ignore_failure: false,
        },
        PipelineOperationSpec {
            operation: SupportedOperation::Grayscale,
            params: json!({}),
            ignore_failure: false,
        },
        PipelineOperationSpec {
            operation: SupportedOperation::Blur,
            params: json!({"sigma": 1.0}),
            ignore_failure: false,
        },
    ]);
    
    let concurrency_levels = vec![1, 2, 4, 8];
    
    for concurrency in concurrency_levels {
        group.bench_with_input(
            BenchmarkId::new("concurrent_requests", concurrency),
            &concurrency,
            |b, &concurrency| {
                b.iter(|| {
                    let handles: Vec<_> = (0..concurrency)
                        .map(|_| {
                            let img = test_image.clone();
                            let ops = operations.clone();
                            
                            thread::spawn(move || {
                                execute_pipeline((*img).clone(), (*ops).clone())
                            })
                        })
                        .collect();
                    
                    let results: Vec<_> = handles
                        .into_iter()
                        .map(|handle| handle.join().unwrap())
                        .collect();
                    
                    black_box(results)
                })
            },
        );
    }
    
    group.finish();
}

// Benchmark format conversion performance
fn bench_format_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("format_performance");
    
    let test_image = create_test_image(800, 600);
    
    let format_operations = vec![
        ("jpeg_high_quality", json!({"format": "jpeg", "quality": 95})),
        ("jpeg_medium_quality", json!({"format": "jpeg", "quality": 80})),
        ("jpeg_low_quality", json!({"format": "jpeg", "quality": 50})),
        ("png", json!({"format": "png"})),
        ("webp_high_quality", json!({"format": "webp", "quality": 95})),
        ("webp_low_quality", json!({"format": "webp", "quality": 50})),
    ];
    
    for (format_name, params) in format_operations {
        let operations = vec![
            PipelineOperationSpec {
                operation: SupportedOperation::Convert,
                params,
                ignore_failure: false,
            },
        ];
        
        group.bench_with_input(
            BenchmarkId::new("format_conversion", format_name),
            &operations,
            |b, ops| {
                b.iter(|| {
                    black_box(execute_pipeline(
                        black_box(test_image.clone()),
                        black_box(ops.clone()),
                    ))
                })
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_pipeline_operations_count,
    bench_memory_usage_patterns,
    bench_concurrent_processing,
    bench_format_performance
);
criterion_main!(benches);