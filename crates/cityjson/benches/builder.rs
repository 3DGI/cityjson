//! Benchmarks that build objects
use cityjson::prelude::*;
use cityjson::v2_0::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

fn prepare_data(_i: usize) -> usize {
    let _ = CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);
    todo!()
}

fn build_cityobjects(conf: (usize, bool)) {
    let _nr_objects = conf.0;
    let _with_geometry = conf.1;
    todo!()
}

fn bench_build_cityobjects_without_geometry(c: &mut Criterion) {
    let mut group = c.benchmark_group("builder");

    // Set throughput for better reporting
    group.throughput(Throughput::Elements(10000));

    group.bench_function("build_10000_cityobjects_without_geometry", |b| {
        let data = prepare_data(10000);
        b.iter(|| build_cityobjects(black_box((data, false))));
    });

    group.finish();
}

fn bench_build_cityobjects_with_geometry(c: &mut Criterion) {
    let mut group = c.benchmark_group("builder");

    // Set throughput for better reporting
    group.throughput(Throughput::Elements(10000));

    group.bench_function("build_10000_cityobjects_with_geometry", |b| {
        let data = prepare_data(10000);
        b.iter(|| build_cityobjects(black_box((data, true))));
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_build_cityobjects_without_geometry,
    bench_build_cityobjects_with_geometry
);
criterion_main!(benches);
