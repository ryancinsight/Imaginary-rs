use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use imaginary::image::operations::watermark;
use imaginary::image::params::{WatermarkParams, WatermarkPosition};
use image::{DynamicImage, ImageBuffer, Rgba};

fn create_test_image(width: u32, height: u32) -> DynamicImage {
    DynamicImage::ImageRgba8(ImageBuffer::from_pixel(
        width,
        height,
        Rgba([0u8, 0u8, 0u8, 255u8]),
    ))
}

fn bench_watermark(c: &mut Criterion) {
    let mut group = c.benchmark_group("watermark_operations");

    // We test with a small image to make the font loading overhead more apparent
    // relative to the image processing time.
    let small_img = create_test_image(100, 100);
    let large_img = create_test_image(1000, 1000);

    let params = WatermarkParams {
        text: "Benchmark".to_string(),
        opacity: 0.8,
        position: WatermarkPosition::BottomRight,
        font_size: 24,
        color: [255, 255, 255],
        x: None,
        y: None,
    };

    group.bench_with_input(
        BenchmarkId::new("watermark_text", "small_100x100"),
        &small_img,
        |b, img| {
            b.iter(|| {
                black_box(watermark(
                    black_box(img.clone()),
                    black_box(&params),
                ))
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("watermark_text", "large_1000x1000"),
        &large_img,
        |b, img| {
            b.iter(|| {
                black_box(watermark(
                    black_box(img.clone()),
                    black_box(&params),
                ))
            })
        },
    );

    group.finish();
}

criterion_group!(benches, bench_watermark);
criterion_main!(benches);
