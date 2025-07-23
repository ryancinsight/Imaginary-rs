use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use imaginary::image::operations::*;
use imaginary::image::pipeline_executor::execute_pipeline;
use imaginary::image::pipeline_types::{PipelineOperationSpec, SupportedOperation};
use imaginary::image::params::{ResizeParams, CropParams, RotateParams, BlurParams, FormatConversionParams};
use image::{DynamicImage, ImageBuffer, RgbImage};
use serde_json::json;

// Create test images of different sizes for benchmarking
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

// Benchmark resize operations
fn bench_resize(c: &mut Criterion) {
    let mut group = c.benchmark_group("resize_operations");
    
    let sizes = vec![
        (100, 100, "small"),
        (800, 600, "medium"),
        (1920, 1080, "large"),
        (4000, 3000, "xlarge"),
    ];
    
    for (width, height, size_name) in sizes {
        let img = create_test_image(width, height);
        
        group.bench_with_input(
            BenchmarkId::new("resize_to_thumbnail", size_name),
            &img,
            |b, img| {
                let params = ResizeParams { width: 200, height: 200 };
                b.iter(|| {
                    black_box(resize(
                        black_box(img.clone()),
                        black_box(&params),
                    ))
                })
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("resize_upscale", size_name),
            &img,
            |b, img| {
                let params = ResizeParams { width: width * 2, height: height * 2 };
                b.iter(|| {
                    black_box(resize(
                        black_box(img.clone()),
                        black_box(&params),
                    ))
                })
            },
        );
    }
    
    group.finish();
}

// Benchmark crop operations
fn bench_crop(c: &mut Criterion) {
    let mut group = c.benchmark_group("crop_operations");
    
    let img = create_test_image(1000, 1000);
    
    let crop_sizes = vec![
        (100, 100, "small_crop"),
        (500, 500, "medium_crop"),
        (800, 800, "large_crop"),
    ];
    
    for (crop_width, crop_height, crop_name) in crop_sizes {
        group.bench_with_input(
            BenchmarkId::new("crop", crop_name),
            &img,
            |b, img| {
                let params = CropParams { x: 0, y: 0, width: crop_width, height: crop_height };
                b.iter(|| {
                    black_box(crop(
                        black_box(img.clone()),
                        black_box(&params),
                    ))
                })
            },
        );
    }
    
    group.finish();
}

// Benchmark rotation operations
fn bench_rotate(c: &mut Criterion) {
    let mut group = c.benchmark_group("rotate_operations");
    
    let img = create_test_image(800, 600);
    
    let angles = vec![90.0, 180.0, 270.0, 45.0];
    
    for angle in angles {
        group.bench_with_input(
            BenchmarkId::new("rotate", format!("{}_degrees", angle)),
            &img,
            |b, img| {
                let params = RotateParams { degrees: angle };
                b.iter(|| {
                    black_box(rotate(
                        black_box(img.clone()),
                        black_box(&params),
                    ))
                })
            },
        );
    }
    
    group.finish();
}

// Benchmark color operations
fn bench_color_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("color_operations");
    
    let img = create_test_image(800, 600);
    
    group.bench_function("grayscale", |b| {
        b.iter(|| {
            black_box(grayscale(black_box(img.clone())))
        })
    });
    
    group.bench_function("adjust_brightness", |b| {
        b.iter(|| {
            black_box(adjust_brightness(
                black_box(img.clone()),
                black_box(20),
            ))
        })
    });
    
    group.bench_function("adjust_contrast", |b| {
        b.iter(|| {
            black_box(adjust_contrast(
                black_box(img.clone()),
                black_box(1.2),
            ))
        })
    });
    
    group.finish();
}

// Benchmark filter operations
fn bench_filters(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_operations");
    
    let img = create_test_image(800, 600);
    
    group.bench_function("blur", |b| {
        let params = BlurParams { sigma: 2.0, minampl: None };
        b.iter(|| {
            black_box(blur(
                black_box(img.clone()),
                black_box(&params),
            ))
        })
    });
    
    group.bench_function("sharpen", |b| {
        b.iter(|| {
            black_box(sharpen(black_box(img.clone())))
        })
    });
    
    group.bench_function("flip_vertical", |b| {
        b.iter(|| {
            black_box(flip_vertical(black_box(img.clone())))
        })
    });
    
    group.bench_function("flip_horizontal", |b| {
        b.iter(|| {
            black_box(flip_horizontal(black_box(img.clone())))
        })
    });
    
    group.finish();
}

// Benchmark format conversion
fn bench_format_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("format_conversion");
    
    let img = create_test_image(800, 600);
    
    let formats = vec!["jpeg", "png", "webp"];
    let qualities = vec![50, 80, 95];
    
    for format in formats {
        for quality in &qualities {
            group.bench_with_input(
                BenchmarkId::new("convert", format!("{}_{}", format, quality)),
                &img,
                |b, img| {
                    let params = FormatConversionParams { 
                        format: format.to_string(), 
                        quality: Some(*quality) 
                    };
                    b.iter(|| {
                        black_box(convert_format(
                            black_box(img.clone()),
                            black_box(&params),
                        ))
                    })
                },
            );
        }
    }
    
    group.finish();
}

// Benchmark complete pipeline operations
fn bench_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("pipeline_operations");
    
    let img = create_test_image(1200, 800);
    
    // Simple pipeline: resize + grayscale
    let simple_ops = vec![
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
    
    // Complex pipeline: resize + crop + rotate + blur + adjust brightness
    let complex_ops = vec![
        PipelineOperationSpec {
            operation: SupportedOperation::Resize,
            params: json!({"width": 800, "height": 600}),
            ignore_failure: false,
        },
        PipelineOperationSpec {
            operation: SupportedOperation::Crop,
            params: json!({"x": 100, "y": 100, "width": 600, "height": 400}),
            ignore_failure: false,
        },
        PipelineOperationSpec {
            operation: SupportedOperation::Rotate,
            params: json!({"degrees": 90.0}),
            ignore_failure: false,
        },
        PipelineOperationSpec {
            operation: SupportedOperation::Blur,
            params: json!({"sigma": 1.5}),
            ignore_failure: false,
        },
        PipelineOperationSpec {
            operation: SupportedOperation::AdjustBrightness,
            params: json!({"value": 10}),
            ignore_failure: false,
        },
    ];
    
    group.bench_function("simple_pipeline", |b| {
        b.iter(|| {
            black_box(execute_pipeline(
                black_box(img.clone()),
                black_box(simple_ops.clone()),
            ))
        })
    });
    
    group.bench_function("complex_pipeline", |b| {
        b.iter(|| {
            black_box(execute_pipeline(
                black_box(img.clone()),
                black_box(complex_ops.clone()),
            ))
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_resize,
    bench_crop,
    bench_rotate,
    bench_color_operations,
    bench_filters,
    bench_format_conversion,
    bench_pipeline
);
criterion_main!(benches);