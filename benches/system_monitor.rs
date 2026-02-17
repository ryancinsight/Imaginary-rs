use criterion::{criterion_group, criterion_main, Criterion};
use std::sync::{Mutex, OnceLock};
use sysinfo::System;

fn bench_system_new(c: &mut Criterion) {
    c.bench_function("system_new_refresh", |b| {
        b.iter(|| {
            let mut system = System::new();
            system.refresh_memory();
            let _ = system.total_memory();
            let _ = system.used_memory();
        })
    });
}

fn bench_system_reuse(c: &mut Criterion) {
    static SYSTEM: OnceLock<Mutex<System>> = OnceLock::new();

    // Initialize once outside the loop to simulate the global state
    SYSTEM.get_or_init(|| Mutex::new(System::new()));

    c.bench_function("system_reuse_refresh", |b| {
        b.iter(|| {
            let mut system = SYSTEM.get().unwrap().lock().unwrap();
            system.refresh_memory();
            let _ = system.total_memory();
            let _ = system.used_memory();
        })
    });
}

criterion_group!(benches, bench_system_new, bench_system_reuse);
criterion_main!(benches);
