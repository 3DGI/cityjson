use std::fs;
use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

fn bench_ndjson(c: &mut Criterion) {
    let sample = find_first(
        "/home/balazs/Data/3DBAG_3dtiles_test/cjindex/ndjson",
        "city.jsonl",
    );
    c.bench_function("ndjson_parse_second_line", |b| {
        b.iter(|| {
            let contents = fs::read_to_string(black_box(&sample)).expect("sample ndjson file");
            let mut lines = contents.lines();
            let _metadata: serde_json::Value =
                serde_json::from_str(lines.next().expect("metadata line")).expect("valid JSON");
            let feature: serde_json::Value =
                serde_json::from_str(lines.next().expect("feature line")).expect("valid JSON");
            black_box(feature);
        });
    });
}

fn find_first(root: &str, suffix: &str) -> std::path::PathBuf {
    for entry in walkdir::WalkDir::new(root) {
        let entry = entry.expect("directory entry");
        if entry.file_type().is_file() && entry.path().to_string_lossy().ends_with(suffix) {
            return entry.path().to_path_buf();
        }
    }
    panic!("no {suffix} file found in {root}");
}

criterion_group!(benches, bench_ndjson);
criterion_main!(benches);
