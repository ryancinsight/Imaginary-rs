use criterion::{black_box, criterion_group, criterion_main, Criterion};
use imaginary::image::pipeline_types::PipelineOperationSpec;
use serde::Deserialize;
use serde_json::json;

fn bench_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("param_parsing");

    // Simulate a complex operation spec as Value
    // Note: PipelineOperationSpec uses serde(flatten) for 'operation',
    // but the input JSON for it must match the structure.
    // The structure expected by PipelineOperationSpec (which flattens PipelineOperation)
    // depends on PipelineOperation enum representation.
    // PipelineOperation uses: #[serde(tag = "operation", content = "params")]
    // AND #[serde(rename_all = "camelCase")]

    // So the JSON should look like:
    // {
    //   "operation": "resize",
    //   "params": { "width": 800, "height": 600, "filter": "Lanczos3" },
    //   "ignoreFailure": false
    // }

    let json_value = json!({
        "operation": "resize",
        "params": {
            "width": 800,
            "height": 600,
            "filter": "Lanczos3"
        },
        "ignoreFailure": false
    });

    group.bench_function("parse_from_value_clone", |b| {
        b.iter(|| {
            let val = black_box(&json_value);
            // Simulate the inefficiency: clone the value then deserialize
            let spec: PipelineOperationSpec = serde_json::from_value(val.clone()).unwrap();
            black_box(spec);
        })
    });

    group.bench_function("parse_from_value_ref", |b| {
        b.iter(|| {
            let val = black_box(&json_value);
            // Optimized: deserialize directly from reference
            let spec = PipelineOperationSpec::deserialize(val).unwrap();
            black_box(spec);
        })
    });

    group.finish();
}

criterion_group!(benches, bench_parsing);
criterion_main!(benches);
