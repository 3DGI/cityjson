mod common;

use std::hint::black_box;
use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion, SamplingMode, Throughput};

use common::{write_cases, write_write_suite_metadata};

fn configure_group(group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>) {
    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(3));
    group.measurement_time(Duration::from_secs(10));
    group.sampling_mode(SamplingMode::Flat);
}

fn bench_write(c: &mut Criterion) {
    let prepared_cases: Vec<_> = write_cases()
        .into_iter()
        .map(|case| case.prepare_write())
        .collect();
    write_write_suite_metadata(&prepared_cases);

    for prepared in &prepared_cases {
        let mut group = c.benchmark_group(prepared.name.as_str());
        group.throughput(Throughput::Bytes(prepared.output_bytes));
        configure_group(&mut group);

        group.bench_function("serde_cityjson/to_string", |b| {
            b.iter_with_large_drop(|| {
                serde_cityjson::to_string(black_box(&prepared.model)).unwrap()
            });
        });

        group.bench_function("serde_cityjson/to_string_validated", |b| {
            b.iter_with_large_drop(|| {
                serde_cityjson::to_string_validated(black_box(&prepared.model)).unwrap()
            });
        });

        group.bench_function("serde_json::to_string", |b| {
            b.iter_with_large_drop(|| serde_json::to_string(black_box(&prepared.value)).unwrap());
        });

        group.finish();
    }
}

criterion_group!(benches, bench_write);
criterion_main!(benches);
