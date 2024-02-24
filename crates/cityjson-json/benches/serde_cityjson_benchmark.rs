use serde_cityjson::{deserialize_from_path, serde_value};
use std::path::PathBuf;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    // Download file from https://data.3dbag.nl/cityjson/v20231008/tiles/10/356/724/10-356-724.city.json.gz
    let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources").join("data");

    let mut group = c.benchmark_group("3DBAG");
    group.sample_size(30);
    let p_json = data_dir.join("10-356-724.city.json");
    group.bench_function("3DBAG serde_cityjson", |b| b.iter_with_large_drop(|| deserialize_from_path(black_box(&p_json))));
    group.bench_function("3DBAG serde_json::Value", |b| b.iter_with_large_drop(|| serde_value(black_box(&p_json))));
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);