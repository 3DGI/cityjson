mod common;

use std::hint::black_box;
use std::time::Duration;

use cityjson_arrow::{ImportOptions, read_stream};
use cityjson_json::v2_0::{ReadOptions, WriteOptions, read_model, to_vec};
use cityjson_parquet::{PackageReader, PackageWriter};
use criterion::{Criterion, SamplingMode, Throughput, criterion_group, criterion_main};
use tempfile::NamedTempFile;

use common::{
    READ_BENCH_JSON, READ_BENCH_PACKAGE, READ_BENCH_STREAM, WRITE_BENCH_JSON, WRITE_BENCH_PACKAGE,
    WRITE_BENCH_STREAM, read_cases, write_cases, write_read_suite_metadata,
    write_write_suite_metadata,
};

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
        let mut group = c.benchmark_group(format!("read/{}", prepared.name));
        configure_group(&mut group);

        group.throughput(Throughput::Bytes(prepared.package_bytes));
        group.bench_function(READ_BENCH_PACKAGE, |b| {
            b.iter_with_large_drop(|| {
                PackageReader::default()
                    .read_file(black_box(&prepared.package_path))
                    .unwrap()
            });
        });

        group.throughput(Throughput::Bytes(prepared.stream_bytes_len));
        group.bench_function(READ_BENCH_STREAM, |b| {
            b.iter_with_large_drop(|| {
                read_stream(
                    black_box(prepared.stream_bytes.as_slice()),
                    &ImportOptions::default(),
                )
                .unwrap()
            });
        });

        group.throughput(Throughput::Bytes(prepared.json_bytes_len));
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

fn bench_write(c: &mut Criterion) {
    let prepared_cases: Vec<_> = write_cases()
        .into_iter()
        .map(|case| case.prepare_write())
        .collect();
    write_write_suite_metadata(&prepared_cases);

    for prepared in &prepared_cases {
        let mut group = c.benchmark_group(format!("write/{}", prepared.name));
        configure_group(&mut group);

        group.throughput(Throughput::Bytes(
            prepared.benchmark_bytes(WRITE_BENCH_PACKAGE),
        ));
        group.bench_function(WRITE_BENCH_PACKAGE, |b| {
            b.iter(|| {
                let tmp = NamedTempFile::new().unwrap();
                PackageWriter::default()
                    .write_file(tmp.path(), black_box(&prepared.model))
                    .unwrap();
            });
        });

        group.throughput(Throughput::Bytes(
            prepared.benchmark_bytes(WRITE_BENCH_STREAM),
        ));
        group.bench_function(WRITE_BENCH_STREAM, |b| {
            b.iter_with_large_drop(|| {
                let mut bytes = Vec::new();
                cityjson_arrow::write_stream(
                    &mut bytes,
                    black_box(&prepared.model),
                    &cityjson_arrow::ExportOptions::default(),
                )
                .unwrap();
                bytes
            });
        });

        group.throughput(Throughput::Bytes(prepared.benchmark_bytes(WRITE_BENCH_JSON)));
        group.bench_function(WRITE_BENCH_JSON, |b| {
            b.iter_with_large_drop(|| {
                to_vec(black_box(&prepared.model), &WriteOptions::default()).unwrap()
            });
        });

        group.finish();
    }
}

criterion_group!(benches, bench_read, bench_write);
criterion_main!(benches);
