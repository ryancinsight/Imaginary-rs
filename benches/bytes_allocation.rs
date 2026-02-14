use axum::body::Bytes;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rand::RngCore;

fn bench_bytes_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("bytes_allocation");

    // Test sizes: 100KB
    let sizes = [100 * 1024];

    for size in sizes {
        group.throughput(Throughput::Bytes(size as u64));

        // Create random data
        let mut data = vec![0u8; size];
        rand::thread_rng().fill_bytes(&mut data);

        // Benchmark: Scenario 1 (Old)
        // Clone the Vec<u8> (Deep Copy) then convert to Bytes
        group.bench_with_input(
            BenchmarkId::new("old_implementation", size),
            &size,
            |b, _| {
                b.iter_with_setup(
                    || data.clone(),
                    |vec_data| {
                        // Logic from original code:
                        // let bytes_for_cache = Bytes::from(final_image_bytes.clone());
                        // response... Body::from(final_image_bytes)

                        // We measure the cost of the clone + conversion
                        let _cache_data = Bytes::from(black_box(vec_data.clone()));
                        let _response_data = black_box(vec_data);
                    },
                )
            },
        );

        // Benchmark: Scenario 2 (New)
        // Convert Vec<u8> to Bytes (Move) then Clone Bytes (Shallow Copy)
        group.bench_with_input(
            BenchmarkId::new("new_implementation", size),
            &size,
            |b, _| {
                b.iter_with_setup(
                    || data.clone(),
                    |vec_data| {
                        // Logic for optimized code:
                        // let bytes = Bytes::from(final_image_bytes);
                        // let bytes_for_cache = bytes.clone();
                        // response... Body::from(bytes)

                        let bytes = Bytes::from(vec_data);
                        let _cache_data = black_box(bytes.clone());
                        let _response_data = black_box(bytes);
                    },
                )
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_bytes_allocation);
criterion_main!(benches);
