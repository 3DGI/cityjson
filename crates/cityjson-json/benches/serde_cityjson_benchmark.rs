use serde_cityjson::{deserialize_from_path, serde_value};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("3DBAG");
    group.sample_size(30);
    let p_json = "/data/3DBAG/export/tiles/9/268/572/9-268-572.city.json";
    group.bench_function("3DBAG serde_cityjson", |b| b.iter_with_large_drop(|| deserialize_from_path(black_box(&p_json))));
    group.bench_function("3DBAG serde_json::Value", |b| b.iter_with_large_drop(|| serde_value(black_box(&p_json))));
    group.finish();

    let mut group = c.benchmark_group("3D Basisvoorziening");
    group.sample_size(30);
    let p_json = "/data/3D_basisvoorziening/32cz1_2020_volledig/32cz1_01.json";
    group.bench_function("3D Basisvoorziening serde_cityjson", |b| b.iter_with_large_drop(|| deserialize_from_path(black_box(&p_json))));
    group.bench_function("3D Basisvoorziening serde_json::Value", |b| b.iter_with_large_drop(|| serde_value(black_box(&p_json))));
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);