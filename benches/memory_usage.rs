use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use imaginary::image::pipeline_executor::execute_pipeline;
use imaginary::image::pipeline_types::{PipelineOperationSpec, SupportedOperation};
use image::{DynamicImage, ImageBuffer, RgbImage};
use serde_json::json;
use std::sync::Arc;
use std::thread;

// Create test images with different characteristics
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

// Benchmark memory usage for different image sizes
fn bench_memory_by_image_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_by_image_size");
    
    let sizes = vec![
        (200, 150, "tiny"),
        (640, 480, "small"),
        (1280, 720, "medium"),
        (1920, 1080, "large"),
        (3840, 2160, "xlarge"),
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
    ];
    
    for (width, height, size_name) in sizes {
        let img = create_test_image(width, height);
        
        group.bench_with_input(
            BenchmarkId::new("pipeline_memory_usage", size_name),
            &img,
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

// Benchmark memory usage for different operation counts
fn bench_memory_by_operation_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_by_operation_count");
    
    let img = create_test_image(800, 600);
    
    let operation_sets = vec![
        (1, "single_op", vec![
            PipelineOperationSpec {
                operation: SupportedOperation::Resize,
                params: json!({"width": 400, "height": 300}),
                ignore_failure: false,
            },
        ]),
        (3, "three_ops", vec![
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
        (5, "five_ops", vec![
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
            BenchmarkId::new("operation_count_memory", name),
            &operations,
            |b, ops| {
                b.iter(|| {
                    black_box(execute_pipeline(
                        black_box(img.clone()),
                        black_box(ops.clone()),
                    ))
                })
            },
        );
    }
    
    group.finish();
}

// Benchmark memory usage patterns for different formats
fn bench_memory_by_format(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_by_format");
    
    let img = create_test_image(1000, 750);
    
    let format_operations = vec![
        ("jpeg_high", json!({"format": "jpeg", "quality": 95})),
        ("jpeg_medium", json!({"format": "jpeg", "quality": 80})),
        ("jpeg_low", json!({"format": "jpeg", "quality": 50})),
        ("png", json!({"format": "png"})),
        ("webp_high", json!({"format": "webp", "quality": 95})),
        ("webp_low", json!({"format": "webp", "quality": 50})),
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
            BenchmarkId::new("format_memory", format_name),
            &operations,
            |b, ops| {
                b.iter(|| {
                    black_box(execute_pipeline(
                        black_box(img.clone()),
                        black_box(ops.clone()),
                    ))
                })
            },
        );
    }
    
    group.finish();
}

// Benchmark memory efficiency of image cloning vs references
fn bench_memory_cloning_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_cloning_patterns");
    
    let img = create_test_image(800, 600);
    let img_arc = Arc::new(img.clone());
    
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
    ];
    
    // Test with direct cloning
    group.bench_function("direct_cloning", |b| {
        b.iter(|| {
            black_box(execute_pipeline(
                black_box(img.clone()),
                black_box(operations.clone()),
            ))
        })
    });
    
    // Test with Arc to demonstrate that it provides no benefit with the current
    // `execute_pipeline` API, which requires a full clone of the image data.
    group.bench_function("arc_reference_ineffective", |b| {
        b.iter(|| {
            // This is a deep clone of the image data, not a cheap reference count increment.
            let img_clone = (*img_arc).clone();
            black_box(execute_pipeline(
                black_box(img_clone),
                black_box(operations.clone()),
            ))
        })
    });
    
    group.finish();
}

// Benchmark memory usage under concurrent load
fn bench_memory_concurrent_load(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_concurrent_load");
    
    let img = create_test_image(600, 400);
    let operations = vec![
        PipelineOperationSpec {
            operation: SupportedOperation::Resize,
            params: json!({"width": 300, "height": 200}),
            ignore_failure: false,
        },
        PipelineOperationSpec {
            operation: SupportedOperation::Blur,
            params: json!({"sigma": 1.0}),
            ignore_failure: false,
        },
    ];
    
    let concurrency_levels = vec![1, 2, 4, 8];
    
    for concurrency in concurrency_levels {
        group.bench_with_input(
            BenchmarkId::new("concurrent_memory", concurrency),
            &concurrency,
            |b, &concurrency| {
                b.iter(|| {
                    let handles: Vec<_> = (0..concurrency)
                        .map(|_| {
                            let img = img.clone();
                            let ops = operations.clone();
                            
                            thread::spawn(move || {
                                execute_pipeline(img, ops)
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

criterion_group!(
    benches,
    bench_memory_by_image_size,
    bench_memory_by_operation_count,
    bench_memory_by_format,
    bench_memory_cloning_patterns,
    bench_memory_concurrent_load
);
criterion_main!(benches);