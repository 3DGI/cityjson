mod common;

use std::hint::black_box;
use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion, SamplingMode, Throughput};

use common::{
    write_cases, write_write_suite_metadata, WRITE_BENCH_CITYJSON_JSON_AS_JSON_TO_VALUE,
    WRITE_BENCH_CITYJSON_JSON_TO_STRING, WRITE_BENCH_CITYJSON_JSON_TO_STRING_VALIDATED,
    WRITE_BENCH_SERDE_JSON_TO_STRING,
};

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
        configure_group(&mut group);

        group.throughput(Throughput::Bytes(
            prepared.benchmark_bytes(WRITE_BENCH_CITYJSON_JSON_AS_JSON_TO_VALUE),
        ));
        group.bench_function(WRITE_BENCH_CITYJSON_JSON_AS_JSON_TO_VALUE, |b| {
            b.iter_with_large_drop(|| {
                serde_json::to_value(cityjson_json::as_json(black_box(&prepared.model))).unwrap()
            });
        });

        group.throughput(Throughput::Bytes(
            prepared.benchmark_bytes(WRITE_BENCH_CITYJSON_JSON_TO_STRING),
        ));
        group.bench_function(WRITE_BENCH_CITYJSON_JSON_TO_STRING, |b| {
            b.iter_with_large_drop(|| {
                cityjson_json::to_string(black_box(&prepared.model)).unwrap()
            });
        });

        group.throughput(Throughput::Bytes(
            prepared.benchmark_bytes(WRITE_BENCH_CITYJSON_JSON_TO_STRING_VALIDATED),
        ));
        group.bench_function(WRITE_BENCH_CITYJSON_JSON_TO_STRING_VALIDATED, |b| {
            b.iter_with_large_drop(|| {
                cityjson_json::to_string_validated(black_box(&prepared.model)).unwrap()
            });
        });

        group.throughput(Throughput::Bytes(
            prepared.benchmark_bytes(WRITE_BENCH_SERDE_JSON_TO_STRING),
        ));
        group.bench_function(WRITE_BENCH_SERDE_JSON_TO_STRING, |b| {
            b.iter_with_large_drop(|| {
                serde_json::to_string(black_box(&prepared.canonical_value)).unwrap()
            });
        });

        group.finish();
    }
}

criterion_group!(benches, bench_write);
criterion_main!(benches);
