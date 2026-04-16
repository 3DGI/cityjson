#[path = "../bench_support/mod.rs"]
mod support;

use std::fs;
use std::io::Cursor;
use std::time::Duration;

use cityjson_lib::{arrow, json};
use criterion::{Criterion, SamplingMode, Throughput, criterion_group, criterion_main};
use support::{BenchmarkCase, benchmark_cases};

fn configure_group(group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>) {
    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(3));
    group.measurement_time(Duration::from_secs(10));
    group.sampling_mode(SamplingMode::Flat);
}

struct PreparedDiagnosticCase {
    case: BenchmarkCase,
    model: cityjson_lib::CityModel,
    batches: cityjson_lib::arrow::ArrowBatches,
    stream_bytes: Vec<u8>,
}

impl PreparedDiagnosticCase {
    fn new(case: BenchmarkCase) -> Self {
        let json_bytes = fs::read(&case.json_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", case.json_path.display()));
        let model = json::read_model(&json_bytes, &json::JsonReadOptions::default())
            .unwrap_or_else(|error| {
                panic!("failed to decode {}: {error}", case.json_path.display())
            });
        let batches = arrow::export_batches(&model)
            .unwrap_or_else(|error| panic!("failed to export batches for {}: {error}", case.id));
        let stream_bytes = fs::read(&case.cityarrow_path).unwrap_or_else(|error| {
            panic!("failed to read {}: {error}", case.cityarrow_path.display())
        });

        Self {
            case,
            model,
            batches,
            stream_bytes,
        }
    }
}

fn bench_diagnostics(c: &mut Criterion) {
    let cases = benchmark_cases();

    for case in cases {
        let prepared = PreparedDiagnosticCase::new(case);

        let mut convert_group = c.benchmark_group(format!("diagnose_convert/{}", prepared.case.id));
        configure_group(&mut convert_group);
        convert_group.bench_function("cityarrow/export_batches", |b| {
            b.iter(|| {
                let _ = arrow::export_batches(&prepared.model).expect("export batches");
            });
        });
        convert_group.bench_function("cityarrow/import_batches", |b| {
            b.iter(|| {
                let _ = arrow::import_batches(&prepared.batches).expect("import batches");
            });
        });
        convert_group.finish();

        let mut stream_group = c.benchmark_group(format!("diagnose_stream/{}", prepared.case.id));
        configure_group(&mut stream_group);
        stream_group.throughput(Throughput::Bytes(prepared.case.cityarrow_bytes));
        stream_group.bench_function("cityarrow/write_stream", |b| {
            b.iter(|| {
                let mut bytes = Vec::new();
                arrow::write_stream(
                    &mut bytes,
                    &prepared.model,
                    &arrow::ExportOptions::default(),
                )
                .expect("write stream");
            });
        });
        stream_group.bench_function("cityarrow/read_stream", |b| {
            b.iter(|| {
                let _ = arrow::read_stream(
                    Cursor::new(prepared.stream_bytes.as_slice()),
                    &arrow::ImportOptions::default(),
                )
                .expect("read stream");
            });
        });
        stream_group.finish();
    }
}

criterion_group!(benches, bench_diagnostics);
criterion_main!(benches);
