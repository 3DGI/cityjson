mod common;

use std::hint::black_box;
use std::time::Duration;

use cityjson_arrow::{ExportOptions, write_stream};
use cityjson_json::v2_0::{WriteOptions, to_vec};
use criterion::{Criterion, SamplingMode, Throughput, criterion_group, criterion_main};

use common::{WRITE_BENCH_JSON, WRITE_BENCH_STREAM, write_cases, write_write_suite_metadata};

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
            prepared.benchmark_bytes(WRITE_BENCH_STREAM),
        ));
        group.bench_function(WRITE_BENCH_STREAM, |b| {
            b.iter_with_large_drop(|| {
                let mut bytes = Vec::new();
                write_stream(
                    &mut bytes,
                    black_box(&prepared.model),
                    &ExportOptions::default(),
                )
                .unwrap();
                bytes
            });
        });

        group.throughput(Throughput::Bytes(
            prepared.benchmark_bytes(WRITE_BENCH_JSON),
        ));
        group.bench_function(WRITE_BENCH_JSON, |b| {
            b.iter_with_large_drop(|| {
                to_vec(black_box(&prepared.model), &WriteOptions::default()).unwrap()
            });
        });

        group.finish();
    }
}

criterion_group!(benches, bench_write);
criterion_main!(benches);
