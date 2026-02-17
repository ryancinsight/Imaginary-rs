use criterion::{criterion_group, criterion_main, Criterion};
use imaginary::storage::generate_operation_hash;
use std::io::Write;
use tempfile::NamedTempFile;
use tokio::runtime::Runtime;

fn benchmark_hashing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut temp_file = NamedTempFile::new().unwrap();
    let data = vec![0u8; 1024 * 1024]; // 1MB
    temp_file.write_all(&data).unwrap();
    let path = temp_file.path().to_owned();

    c.bench_function("generate_operation_hash_1mb", |b| {
        b.to_async(&rt).iter(|| async {
            generate_operation_hash(&path, "resize", "width=100,height=100").await.unwrap();
        })
    });
}

criterion_group!(benches, benchmark_hashing);
criterion_main!(benches);
