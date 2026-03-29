use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};

use cjindex::{BBox, CityIndex, StorageLayout};

const NDJSON_ROOT: &str = "/home/balazs/Data/3DBAG_3dtiles_test/cjindex/ndjson";

fn bench_ndjson(c: &mut Criterion) {
    let root = Path::new(NDJSON_ROOT);
    let sample_source = find_first_jsonl_file(root);
    let sample_fixture = derive_small_ndjson_fixture(&sample_source);
    let feature_id = "ndjson-test-feature".to_owned();

    c.bench_function("ndjson_reindex", |b| {
        b.iter_batched_ref(
            || build_index(&sample_fixture),
            |index| {
                index.reindex().expect("reindex should succeed");
                black_box(index.metadata().expect("metadata should load"));
            },
            BatchSize::LargeInput,
        );
    });

    let index = build_index(&sample_fixture);
    let bbox = BBox {
        min_x: -1.0e12,
        max_x: 1.0e12,
        min_y: -1.0e12,
        max_y: 1.0e12,
    };

    c.bench_function("ndjson_get", |b| {
        b.iter(|| {
            let model = index
                .get(black_box(&feature_id))
                .expect("get should succeed")
                .expect("feature should exist");
            black_box(model);
        });
    });

    c.bench_function("ndjson_query", |b| {
        b.iter(|| {
            let models = index.query(black_box(&bbox)).expect("query should succeed");
            black_box(models);
        });
    });

    c.bench_function("ndjson_query_iter", |b| {
        b.iter(|| {
            let models = index
                .query_iter(black_box(&bbox))
                .expect("query_iter should build")
                .collect::<std::result::Result<Vec<_>, _>>()
                .expect("query_iter should succeed");
            black_box(models);
        });
    });

    c.bench_function("ndjson_metadata", |b| {
        b.iter(|| {
            let metadata = index.metadata().expect("metadata should succeed");
            black_box(metadata);
        });
    });
}

fn build_index(path: &Path) -> CityIndex {
    let index_path = temp_index_path();
    let mut index = CityIndex::open(
        StorageLayout::Ndjson {
            paths: vec![path.to_path_buf()],
        },
        &index_path,
    )
    .expect("index should open");
    index.reindex().expect("reindex should succeed");
    index
}

fn find_first_jsonl_file(root: &Path) -> PathBuf {
    for entry in walkdir::WalkDir::new(root) {
        let entry = entry.expect("directory entry");
        if entry.file_type().is_file()
            && entry.path().extension().and_then(|ext| ext.to_str()) == Some("jsonl")
        {
            return entry.path().to_path_buf();
        }
    }
    panic!("no NDJSON file found in benchmark fixtures");
}

fn derive_small_ndjson_fixture(source: &Path) -> PathBuf {
    let contents = fs::read_to_string(source).expect("sample ndjson tile must be readable");
    let mut lines = contents.lines();
    let metadata = lines.next().expect("sample tile must contain metadata");
    let path = std::env::temp_dir().join(format!(
        "cjindex-ndjson-bench-{}.jsonl",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after the unix epoch")
            .as_nanos()
    ));
    let feature = serde_json::json!({
        "type": "CityJSONFeature",
        "id": "ndjson-test-feature",
        "CityObjects": {
            "ndjson-test-feature": {
                "type": "Building",
                "geometry": [{
                    "type": "MultiSurface",
                    "lod": "1.0",
                    "boundaries": [[[0, 1, 2]]]
                }]
            }
        },
        "vertices": [
            [0, 0, 0],
            [1, 0, 0],
            [0, 1, 0]
        ]
    });

    fs::write(
        &path,
        format!(
            "{metadata}\n{}\n",
            serde_json::to_string(&feature).expect("feature JSON")
        ),
    )
    .expect("derived NDJSON fixture must be writable");
    path
}

fn temp_index_path() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("cjindex-ndjson-bench-{unique}.sqlite"));
    if path.exists() {
        fs::remove_file(&path).expect("benchmark index path should be removable");
    }
    path
}

criterion_group!(benches, bench_ndjson);
criterion_main!(benches);
