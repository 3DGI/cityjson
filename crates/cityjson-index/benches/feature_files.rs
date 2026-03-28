use std::fs;
use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

fn bench_feature_files(c: &mut Criterion) {
    let sample = find_first(
        "/home/balazs/Data/3DBAG_3dtiles_test/cjindex/feature-files/features",
        "city.jsonl",
    );
    c.bench_function("feature_files_parse", |b| {
        b.iter(|| {
            let bytes = fs::read(black_box(&sample)).expect("sample feature file");
            let value: serde_json::Value = serde_json::from_slice(&bytes).expect("valid JSON");
            black_box(value);
        });
    });
}

fn find_first(root: &str, suffix: &str) -> std::path::PathBuf {
    for entry in walkdir::WalkDir::new(root) {
        let entry = entry.expect("directory entry");
        if entry.file_type().is_file()
            && entry.path().to_string_lossy().ends_with(suffix)
            && entry.metadata().map(|meta| meta.len() > 0).unwrap_or(false)
        {
            return entry.path().to_path_buf();
        }
    }
    panic!("no {suffix} file found in {root}");
}

criterion_group!(benches, bench_feature_files);
criterion_main!(benches);
