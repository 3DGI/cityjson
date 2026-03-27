mod common;

use std::hint::black_box;
use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion, SamplingMode, Throughput};

use common::{read_cases, write_suite_metadata};

fn configure_group(group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>) {
    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(3));
    group.measurement_time(Duration::from_secs(10));
    group.sampling_mode(SamplingMode::Flat);
}

fn bench_read(c: &mut Criterion) {
    let prepared_cases: Vec<_> = read_cases()
        .into_iter()
        .map(|case| case.prepare())
        .collect();
    write_suite_metadata("read", &prepared_cases);

    for prepared in &prepared_cases {
        let mut group = c.benchmark_group(prepared.name);
        group.throughput(Throughput::Bytes(prepared.input_bytes));
        configure_group(&mut group);

        group.bench_function("serde_cityjson/owned", |b| {
            b.iter_with_large_drop(|| {
                serde_cityjson::from_str_owned(black_box(&prepared.input_json)).unwrap()
            });
        });

        if prepared.borrowed {
            group.bench_function("serde_cityjson/borrowed", |b| {
                b.iter_with_large_drop(|| {
                    serde_cityjson::from_str_borrowed(black_box(&prepared.input_json)).unwrap()
                });
            });
        }

        group.bench_function("serde_json::Value", |b| {
            b.iter_with_large_drop(|| {
                serde_json::from_str::<serde_json::Value>(black_box(&prepared.input_json)).unwrap()
            });
        });

        group.finish();
    }
}

criterion_group!(benches, bench_read);
criterion_main!(benches);
