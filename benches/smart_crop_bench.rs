use criterion::{black_box, criterion_group, criterion_main, Criterion};
use image::{DynamicImage, ImageBuffer, Rgba};
use imaginary::image::operations::transform::smart_crop;
use imaginary::image::params::SmartCropParams;

fn create_large_test_image(width: u32, height: u32) -> DynamicImage {
    let img = ImageBuffer::from_fn(width, height, |x, y| {
        Rgba([(x % 256) as u8, (y % 256) as u8, ((x + y) % 256) as u8, 255])
    });
    DynamicImage::ImageRgba8(img)
}

fn bench_smart_crop_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("smart_crop_allocation");

    // 200x200 image, crop to 100x100
    // Range 100x100 = 10,000 candidates.
    let width = 200;
    let height = 200;
    let img = create_large_test_image(width, height);

    let params = SmartCropParams {
        width: 100,
        height: 100,
        quality: Some(100),
    };

    group.sample_size(10);
    group.bench_function("smart_crop_large_q100", |b| {
        b.iter(|| black_box(smart_crop(black_box(img.clone()), black_box(&params))))
    });

    group.finish();
}

criterion_group!(benches, bench_smart_crop_allocation);
criterion_main!(benches);
