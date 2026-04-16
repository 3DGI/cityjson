#[path = "../bench_support/mod.rs"]
mod support;

use std::fs;
use std::time::Duration;

use cityjson_arrow::internal::{decode_parts, encode_parts, read_stream_parts, write_stream_parts};
use cityjson_lib::json;
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
    parts: cityjson_arrow::schema::CityModelArrowParts,
    stream_bytes: Vec<u8>,
}

impl PreparedDiagnosticCase {
    fn new(case: BenchmarkCase) -> Self {
        let model = json::from_file(&case.json_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", case.json_path.display()));
        let parts = encode_parts(&model)
            .unwrap_or_else(|error| panic!("failed to encode parts for {}: {error}", case.id));
        let stream_bytes = fs::read(&case.cityarrow_path).unwrap_or_else(|error| {
            panic!("failed to read {}: {error}", case.cityarrow_path.display())
        });

        Self {
            case,
            model,
            parts,
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
        convert_group.bench_function("cityarrow/encode_parts", |b| {
            b.iter(|| {
                let _ = encode_parts(&prepared.model).expect("encode parts");
            });
        });
        convert_group.bench_function("cityarrow/decode_parts", |b| {
            b.iter(|| {
                let _ = decode_parts(&prepared.parts).expect("decode parts");
            });
        });
        convert_group.finish();

        let mut stream_group = c.benchmark_group(format!("diagnose_stream/{}", prepared.case.id));
        configure_group(&mut stream_group);
        stream_group.throughput(Throughput::Bytes(prepared.case.cityarrow_bytes));
        stream_group.bench_function("cityarrow/write_parts", |b| {
            b.iter(|| {
                let mut bytes = Vec::new();
                write_stream_parts(&prepared.parts, &mut bytes).expect("write stream parts");
            });
        });
        stream_group.bench_function("cityarrow/read_parts", |b| {
            b.iter(|| {
                let _ =
                    read_stream_parts(prepared.stream_bytes.as_slice()).expect("read stream parts");
            });
        });
        stream_group.finish();
    }
}

criterion_group!(benches, bench_diagnostics);
criterion_main!(benches);
