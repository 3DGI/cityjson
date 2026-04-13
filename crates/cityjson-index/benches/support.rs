#![allow(clippy::explicit_iter_loop, clippy::missing_panics_doc)]

use std::env;
use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use cityjson_index::realistic_workload::{QUERY_BATCH_COUNT, build_realistic_workload};
use cityjson_index::{BBox, CityIndex, StorageLayout};
use cityjson_lib::{Error, Result};
use criterion::{BatchSize, Criterion};
use serde_json::Value;

#[allow(dead_code)]
#[path = "../tests/common/data_prep.rs"]
mod data_prep;

const QUERY_VALIDATION_SAMPLE_COUNT: usize = 32;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum LayoutKind {
    FeatureFiles,
    CityJson,
    Ndjson,
}

pub fn bench_layout(c: &mut Criterion, kind: LayoutKind) {
    let fixtures = fixtures();
    let layout_root = fixtures.layout_root(kind).to_path_buf();
    let populated_index = build_index(kind, &layout_root);
    let label = kind.label();

    c.bench_function(&format!("{label}_reindex"), |b| {
        let reindex_root = layout_root.clone();
        b.iter_batched_ref(
            || empty_index(kind, &reindex_root),
            |index| {
                index.reindex().expect("reindex should succeed");
                black_box(index.metadata().expect("metadata should load"));
            },
            BatchSize::LargeInput,
        );
    });

    c.bench_function(&format!("{label}_get"), |b| {
        b.iter(|| {
            for feature_id in fixtures.get_ids.iter() {
                let model = populated_index
                    .get(black_box(feature_id.as_str()))
                    .expect("get should succeed")
                    .expect("feature should exist");
                black_box(model);
            }
        });
    });

    c.bench_function(&format!("{label}_query"), |b| {
        let mut query_cursor = 0usize;
        b.iter(|| {
            for _ in 0..QUERY_BATCH_COUNT {
                let bbox = &fixtures.query_bboxes[query_cursor % fixtures.query_bboxes.len()];
                query_cursor += 1;
                let models = populated_index
                    .query(black_box(bbox))
                    .expect("query should succeed");
                black_box(models);
            }
        });
    });

    c.bench_function(&format!("{label}_query_iter"), |b| {
        let mut query_iter_cursor = 0usize;
        b.iter(|| {
            for _ in 0..QUERY_BATCH_COUNT {
                let bbox = &fixtures.query_bboxes[query_iter_cursor % fixtures.query_bboxes.len()];
                query_iter_cursor += 1;
                let models = populated_index
                    .query_iter(black_box(bbox))
                    .expect("query_iter should build")
                    .collect::<std::result::Result<Vec<_>, _>>()
                    .expect("query_iter should succeed");
                black_box(models);
            }
        });
    });

    c.bench_function(&format!("{label}_metadata"), |b| {
        b.iter(|| {
            let metadata = populated_index.metadata().expect("metadata should succeed");
            black_box(metadata);
        });
    });
}

impl LayoutKind {
    fn label(self) -> &'static str {
        match self {
            Self::FeatureFiles => "feature_files",
            Self::CityJson => "cityjson",
            Self::Ndjson => "ndjson",
        }
    }

    fn storage_layout(self, root: &Path) -> StorageLayout {
        match self {
            Self::FeatureFiles => StorageLayout::FeatureFiles {
                root: root.to_path_buf(),
                metadata_glob: "**/metadata.json".to_owned(),
                feature_glob: "**/*.city.jsonl".to_owned(),
            },
            Self::CityJson => StorageLayout::CityJson {
                paths: vec![root.to_path_buf()],
            },
            Self::Ndjson => StorageLayout::Ndjson {
                paths: vec![root.to_path_buf()],
            },
        }
    }
}

struct BenchFixtures {
    datasets: data_prep::PreparedDatasets,
    get_ids: Vec<String>,
    query_bboxes: Vec<BBox>,
}

impl BenchFixtures {
    fn layout_root(&self, kind: LayoutKind) -> &Path {
        match kind {
            LayoutKind::FeatureFiles => self.datasets.feature_files.as_path(),
            LayoutKind::CityJson => self.datasets.cityjson.as_path(),
            LayoutKind::Ndjson => self.datasets.ndjson.as_path(),
        }
    }
}

fn fixtures() -> &'static BenchFixtures {
    static FIXTURES: OnceLock<BenchFixtures> = OnceLock::new();
    FIXTURES.get_or_init(|| prepare_bench_fixtures().expect("benchmark fixtures should prepare"))
}

fn prepare_bench_fixtures() -> Result<BenchFixtures> {
    let datasets = prepared_datasets()?;
    let workload = build_realistic_workload(&datasets.feature_files)?;

    validate_workloads(&datasets, &workload.get_ids, &workload.query_bboxes)?;

    Ok(BenchFixtures {
        datasets,
        get_ids: workload.get_ids,
        query_bboxes: workload.query_bboxes,
    })
}

fn prepared_datasets() -> Result<data_prep::PreparedDatasets> {
    let output_root = bench_root();
    let feature_files_root = output_root.join("feature-files");
    let cityjson_root = output_root.join("cityjson");
    let ndjson_root = output_root.join("ndjson");

    if feature_files_root.exists()
        && cityjson_root.exists()
        && ndjson_root.exists()
        && manifest_matches(&output_root)
    {
        return Ok(data_prep::PreparedDatasets {
            feature_files: feature_files_root,
            cityjson: cityjson_root,
            ndjson: ndjson_root,
        });
    }

    Err(Error::Import(format!(
        "benchmark dataset is missing or stale under {}; run `just prep-test-data` first",
        output_root.display()
    )))
}

fn manifest_matches(output_root: &Path) -> bool {
    let manifest_path = output_root.join("manifest.json");
    let Ok(bytes) = fs::read(&manifest_path) else {
        return false;
    };
    let manifest: Value = match serde_json::from_slice(&bytes) {
        Ok(manifest) => manifest,
        Err(_) => return false,
    };
    manifest
        .get("tile_index_url")
        .and_then(|value| value.as_str())
        == Some(data_prep::DEFAULT_TILE_INDEX_URL)
}

fn bench_root() -> PathBuf {
    env::var_os("CJINDEX_BENCH_ROOT").map_or_else(
        || PathBuf::from(data_prep::DEFAULT_OUTPUT_ROOT),
        PathBuf::from,
    )
}

fn build_index(kind: LayoutKind, root: &Path) -> CityIndex {
    let index_path = unique_temp_file(
        &format!("cityjson-index-bench-{}-build", kind.label()),
        "sqlite",
    );
    let mut index = CityIndex::open(kind.storage_layout(root), &index_path)
        .expect("benchmark index should open");
    index.reindex().expect("benchmark index should reindex");
    index
}

fn empty_index(kind: LayoutKind, root: &Path) -> CityIndex {
    let index_path = unique_temp_file(
        &format!("cityjson-index-bench-{}-empty", kind.label()),
        "sqlite",
    );
    CityIndex::open(kind.storage_layout(root), &index_path).expect("benchmark index should open")
}

fn validate_workloads(
    datasets: &data_prep::PreparedDatasets,
    get_ids: &[String],
    query_bboxes: &[BBox],
) -> Result<()> {
    let layouts = [
        (
            LayoutKind::FeatureFiles,
            datasets.feature_files.as_path().to_path_buf(),
        ),
        (
            LayoutKind::CityJson,
            datasets.cityjson.as_path().to_path_buf(),
        ),
        (LayoutKind::Ndjson, datasets.ndjson.as_path().to_path_buf()),
    ];

    for (kind, root) in layouts {
        let index = build_index(kind, &root);

        for id in get_ids {
            let model = index
                .get(id)?
                .ok_or_else(|| Error::Import(format!("feature {id} should be indexed")))?;
            black_box(model);
        }

        for bbox in query_bboxes
            .iter()
            .take(QUERY_VALIDATION_SAMPLE_COUNT.min(query_bboxes.len()))
        {
            let query_hits = index.query(bbox)?;
            if query_hits.is_empty() {
                return Err(Error::Import(format!(
                    "query workload bbox produced no hits for {}",
                    kind.label()
                )));
            }

            let iter_hits = index
                .query_iter(bbox)?
                .collect::<std::result::Result<Vec<_>, _>>()?;
            if iter_hits.is_empty() {
                return Err(Error::Import(format!(
                    "query_iter workload bbox produced no hits for {}",
                    kind.label()
                )));
            }
        }
    }

    Ok(())
}
fn unique_temp_file(label: &str, suffix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time must be after the unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("cityjson-index-{label}-{unique}.{suffix}"));
    if path.exists() {
        fs::remove_file(&path).expect("benchmark temp file should be removable");
    }
    path
}
