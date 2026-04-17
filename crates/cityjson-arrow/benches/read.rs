mod common;

use std::hint::black_box;
use std::time::Duration;

use cityjson_arrow::{ImportOptions, read_stream};
use cityjson_json::v2_0::{ReadOptions, read_model};
use criterion::{Criterion, SamplingMode, Throughput, criterion_group, criterion_main};

use common::{READ_BENCH_JSON, READ_BENCH_STREAM, read_cases, write_read_suite_metadata};

fn configure_group(group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>) {
    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(3));
    group.measurement_time(Duration::from_secs(10));
    group.sampling_mode(SamplingMode::Flat);
}

fn bench_read(c: &mut Criterion) {
    let prepared_cases: Vec<_> = read_cases()
        .into_iter()
        .map(|case| case.prepare_read())
        .collect();
    write_read_suite_metadata(&prepared_cases);

    for prepared in &prepared_cases {
        let mut group = c.benchmark_group(prepared.name.as_str());
        configure_group(&mut group);

        group.throughput(Throughput::Bytes(prepared.stream_input_bytes));
        group.bench_function(READ_BENCH_STREAM, |b| {
            b.iter_with_large_drop(|| {
                read_stream(
                    black_box(prepared.stream_bytes.as_slice()),
                    &ImportOptions::default(),
                )
                .unwrap()
            });
        });

        group.throughput(Throughput::Bytes(prepared.json_input_bytes));
        group.bench_function(READ_BENCH_JSON, |b| {
            b.iter_with_large_drop(|| {
                read_model(
                    black_box(prepared.json_bytes.as_bytes()),
                    &ReadOptions::default(),
                )
                .unwrap()
            });
        });

        group.finish();
    }
}

criterion_group!(benches, bench_read);
criterion_main!(benches);
