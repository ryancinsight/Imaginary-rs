use criterion::{criterion_group, criterion_main, Criterion};
use sysinfo::Disks;

fn bench_disk_refresh(c: &mut Criterion) {
    c.bench_function("disk_refresh", |b| {
        b.iter(|| {
            let _disks = Disks::new_with_refreshed_list();
        })
    });
}

criterion_group!(benches, bench_disk_refresh);
criterion_main!(benches);
