//! Benchmark the execution speed with criterion.rs.
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use criterion::{black_box, criterion_group, criterion_main, Criterion, SamplingMode};

/// Deserialize into a serde_json::Value.
fn serde_value<P: AsRef<Path>>(path: P) -> serde_json::Result<serde_json::Value> {
    let mut file = File::open(path.as_ref()).unwrap();
    let mut json_str = String::new();
    file.read_to_string(&mut json_str).unwrap();
    let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    Ok(val)
}

/// Benchmark with real data. Run 'just download' first to download the data files.
fn real_data(c: &mut Criterion) {
    let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("benches")
        .join("data");
    let sampling_mode = SamplingMode::Flat;

    let mut group = c.benchmark_group("all");
    let p_json = data_dir.join("all.json");
    group.sampling_mode(sampling_mode);
    group.bench_function("strongly_typed", |b| {
        b.iter_with_large_drop(|| {
            let mut file = File::open(&p_json).unwrap();
            let mut json_str = String::new();
            file.read_to_string(&mut json_str).unwrap();
            let ms: nested::Model = serde_json::from_str(&json_str).unwrap();
            black_box(ms);
        })
    });
    group.bench_function("serde_json::Value", |b| {
        b.iter_with_large_drop(|| {
            let val = serde_value(black_box(&p_json)).unwrap();
            black_box(val);
        })
    });
    group.finish();
}

criterion_group!(benches, real_data);
criterion_main!(benches);
