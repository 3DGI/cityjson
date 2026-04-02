use std::collections::HashMap;
use std::fs;
use std::hint::black_box;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

use cjindex::realistic_workload::{seeded_shuffle, WORKLOAD_SHUFFLE_SEED};
use cjindex::{CityIndex, IndexedFeatureRef, StorageLayout};
use cjlib::json::staged;
use cjlib::Result;
use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};

const WORKER_COUNT: usize = 6;
/// Deterministic number of adjacents per building (median from real data).
const ADJACENTS_PER_BUILDING: usize = 4;

/// A single work item: a target building plus its adjacent buildings.
struct BuildingWork {
    target_ref: IndexedFeatureRef,
    adjacent_refs: Vec<IndexedFeatureRef>,
}

/// Baseline equivalent of BuildingWork using file paths.
struct BaselineBuildingWork {
    target_path: PathBuf,
    target_metadata_dir: PathBuf,
    adjacent_paths: Vec<(PathBuf, PathBuf)>,
}

struct ParallelBenchFixture {
    baseline_work: Vec<BaselineBuildingWork>,
    cjindex_work: Vec<BuildingWork>,
    metadata_cache: Arc<HashMap<PathBuf, Vec<u8>>>,
    ndjson_index_path: PathBuf,
    ndjson_root: PathBuf,
}

fn collect_all_refs(index: &CityIndex) -> Result<Vec<IndexedFeatureRef>> {
    let mut all_refs = Vec::new();
    let page_size = 1000;
    let mut offset = 0;
    loop {
        let page = index.feature_ref_page(offset, page_size)?;
        if page.is_empty() {
            break;
        }
        offset += page.len();
        all_refs.extend(page);
    }
    Ok(all_refs)
}

fn temp_index_path(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "cjindex-parallel-{label}-{}.sqlite",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn prepare_fixture() -> Result<ParallelBenchFixture> {
    let feature_files_root = PathBuf::from("tests/data/feature-files");
    let ndjson_root = PathBuf::from("tests/data/ndjson");

    if !feature_files_root.exists() || !ndjson_root.exists() {
        return Err(cjlib::Error::Import(
            "tests/data/{feature-files,ndjson} must exist; run `just prep-test-data` first".into(),
        ));
    }

    // Build feature-files index for the baseline (to collect refs and file paths)
    let ff_index_path = temp_index_path("ff");
    let mut ff_index = CityIndex::open(
        StorageLayout::FeatureFiles {
            root: feature_files_root.clone(),
            metadata_glob: "**/metadata.json".to_owned(),
            feature_glob: "**/*.city.jsonl".to_owned(),
        },
        &ff_index_path,
    )?;
    ff_index.reindex()?;
    let ff_refs = collect_all_refs(&ff_index)?;

    // Build ndjson index for the cjindex case
    let ndjson_index_path = temp_index_path("ndjson");
    let mut ndjson_index = CityIndex::open(
        StorageLayout::Ndjson {
            paths: vec![ndjson_root.clone()],
        },
        &ndjson_index_path,
    )?;
    ndjson_index.reindex()?;
    let ndjson_refs = collect_all_refs(&ndjson_index)?;

    // Build id -> ref maps for both layouts
    let ff_ref_by_id: HashMap<String, IndexedFeatureRef> = ff_refs
        .iter()
        .map(|r| (r.feature_id.clone(), r.clone()))
        .collect();
    let ndjson_ref_by_id: HashMap<String, IndexedFeatureRef> = ndjson_refs
        .iter()
        .map(|r| (r.feature_id.clone(), r.clone()))
        .collect();

    // Use ALL IDs present in both layouts as targets
    let mut target_ids: Vec<String> = ff_ref_by_id
        .keys()
        .filter(|id| ndjson_ref_by_id.contains_key(*id))
        .cloned()
        .collect();
    target_ids.sort();
    seeded_shuffle(&mut target_ids, WORKLOAD_SHUFFLE_SEED);

    // Build deterministic adjacency in O(N): shuffle once, then pick adjacents
    // by strided offset into the shuffled list.
    let n = target_ids.len();
    let mut adj_order = target_ids.clone();
    seeded_shuffle(&mut adj_order, WORKLOAD_SHUFFLE_SEED ^ 0xdead_beef_cafe_babe);

    let mut cjindex_work = Vec::with_capacity(n);
    let mut baseline_work = Vec::with_capacity(n);
    let mut metadata_cache: HashMap<PathBuf, Vec<u8>> = HashMap::new();

    for (i, target_id) in target_ids.iter().enumerate() {
        let ff_target_ref = ff_ref_by_id[target_id].clone();
        let ndjson_target_ref = ndjson_ref_by_id[target_id].clone();

        // Pick ADJACENTS_PER_BUILDING neighbors from the pre-shuffled list,
        // skipping self.
        let mut ndjson_adjacent_refs = Vec::with_capacity(ADJACENTS_PER_BUILDING);
        let mut adjacent_ids_for_baseline = Vec::with_capacity(ADJACENTS_PER_BUILDING);
        let mut j = (i + 1) % n;
        while ndjson_adjacent_refs.len() < ADJACENTS_PER_BUILDING && j != i {
            let adj_id = &adj_order[j];
            if adj_id != target_id {
                ndjson_adjacent_refs.push(ndjson_ref_by_id[adj_id].clone());
                adjacent_ids_for_baseline.push(adj_id.clone());
            }
            j = (j + 1) % n;
        }

        // Baseline: resolve file paths from feature-files layout
        let target_dir = ff_target_ref.source_path.parent().unwrap().to_path_buf();
        if !metadata_cache.contains_key(&target_dir) {
            let md_path = target_dir.join("metadata.json");
            if md_path.exists() {
                metadata_cache.insert(target_dir.clone(), fs::read(&md_path)?);
            }
        }

        let mut adjacent_paths = Vec::with_capacity(ADJACENTS_PER_BUILDING);
        for adj_id in &adjacent_ids_for_baseline {
            let adj_ref = &ff_ref_by_id[adj_id];
            let adj_dir = adj_ref.source_path.parent().unwrap().to_path_buf();
            if !metadata_cache.contains_key(&adj_dir) {
                let md_path = adj_dir.join("metadata.json");
                if md_path.exists() {
                    metadata_cache.insert(adj_dir.clone(), fs::read(&md_path)?);
                }
            }
            adjacent_paths.push((adj_ref.source_path.clone(), adj_dir));
        }

        baseline_work.push(BaselineBuildingWork {
            target_path: ff_target_ref.source_path.clone(),
            target_metadata_dir: target_dir,
            adjacent_paths,
        });

        cjindex_work.push(BuildingWork {
            target_ref: ndjson_target_ref,
            adjacent_refs: ndjson_adjacent_refs,
        });
    }

    // Clean up the temporary feature-files index
    drop(ff_index);
    let _ = fs::remove_file(&ff_index_path);

    Ok(ParallelBenchFixture {
        baseline_work,
        cjindex_work,
        metadata_cache: Arc::new(metadata_cache),
        ndjson_index_path,
        ndjson_root,
    })
}

fn bench_parallel_get(c: &mut Criterion) {
    let fixture = prepare_fixture().expect("fixture should prepare");

    c.bench_function("parallel_get_baseline", |b| {
        let metadata_cache = &fixture.metadata_cache;
        let work = &fixture.baseline_work;

        let chunk_size = (work.len() + WORKER_COUNT - 1) / WORKER_COUNT;
        let chunks: Vec<&[BaselineBuildingWork]> = work.chunks(chunk_size).collect();

        b.iter(|| {
            thread::scope(|s| {
                for chunk in &chunks {
                    s.spawn(|| {
                        for item in *chunk {
                            // Load target (direct file read, mirrors read_feature_json)
                            let target_bytes = fs::read(&item.target_path)
                                .expect("target file should be readable");
                            let target_meta = metadata_cache
                                .get(&item.target_metadata_dir)
                                .expect("target metadata should exist");
                            let target_model =
                                staged::from_feature_slice_with_base(&target_bytes, target_meta)
                                    .expect("target deserialization should succeed");
                            black_box(&target_model);

                            // Load adjacents (direct file read, mirrors
                            // read_feature_json(adj_ref))
                            for (adj_path, adj_meta_dir) in &item.adjacent_paths {
                                let adj_bytes =
                                    fs::read(adj_path).expect("adjacent file should be readable");
                                let adj_meta = metadata_cache
                                    .get(adj_meta_dir)
                                    .expect("adjacent metadata should exist");
                                let adj_model =
                                    staged::from_feature_slice_with_base(&adj_bytes, adj_meta)
                                        .expect("adjacent deserialization should succeed");
                                black_box(adj_model);
                            }
                        }
                    });
                }
            });
        });
    });

    c.bench_function("parallel_get_cjindex_ndjson", |b| {
        let index_path = &fixture.ndjson_index_path;
        let ndjson_root = &fixture.ndjson_root;
        let work = &fixture.cjindex_work;

        let chunk_size = (work.len() + WORKER_COUNT - 1) / WORKER_COUNT;
        let chunks: Vec<&[BuildingWork]> = work.chunks(chunk_size).collect();

        b.iter(|| {
            thread::scope(|s| {
                for chunk in &chunks {
                    s.spawn(|| {
                        // Each worker opens the index once (mirrors _init_worker)
                        let index = CityIndex::open(
                            StorageLayout::Ndjson {
                                paths: vec![ndjson_root.clone()],
                            },
                            index_path,
                        )
                        .expect("index should open");

                        for item in *chunk {
                            // Load target via read_feature (no SQLite lookup)
                            let target_model = index
                                .read_feature(black_box(&item.target_ref))
                                .expect("read_feature should succeed");
                            black_box(&target_model);

                            // Load adjacents via read_feature (no SQLite lookup,
                            // mirrors read_feature_json(adj_ref))
                            for adj_ref in &item.adjacent_refs {
                                let adj_model = index
                                    .read_feature(black_box(adj_ref))
                                    .expect("read_feature should succeed");
                                black_box(adj_model);
                            }
                        }
                    });
                }
            });
        });
    });
}

fn bench_parallel_io_only(c: &mut Criterion) {
    let fixture = prepare_fixture().expect("fixture should prepare");

    c.bench_function("parallel_io_baseline", |b| {
        let metadata_cache = &fixture.metadata_cache;
        let work = &fixture.baseline_work;

        let chunk_size = (work.len() + WORKER_COUNT - 1) / WORKER_COUNT;
        let chunks: Vec<&[BaselineBuildingWork]> = work.chunks(chunk_size).collect();

        b.iter(|| {
            thread::scope(|s| {
                for chunk in &chunks {
                    s.spawn(|| {
                        for item in *chunk {
                            let target_bytes = fs::read(&item.target_path)
                                .expect("target file should be readable");
                            black_box(&target_bytes);
                            let target_meta = metadata_cache
                                .get(&item.target_metadata_dir)
                                .expect("target metadata should exist");
                            black_box(target_meta);

                            for (adj_path, adj_meta_dir) in &item.adjacent_paths {
                                let adj_bytes =
                                    fs::read(adj_path).expect("adjacent file should be readable");
                                black_box(&adj_bytes);
                                let adj_meta = metadata_cache
                                    .get(adj_meta_dir)
                                    .expect("adjacent metadata should exist");
                                black_box(adj_meta);
                            }
                        }
                    });
                }
            });
        });
    });

    c.bench_function("parallel_io_cjindex_ndjson", |b| {
        let index_path = &fixture.ndjson_index_path;
        let ndjson_root = &fixture.ndjson_root;
        let work = &fixture.cjindex_work;

        let chunk_size = (work.len() + WORKER_COUNT - 1) / WORKER_COUNT;
        let chunks: Vec<&[BuildingWork]> = work.chunks(chunk_size).collect();

        b.iter(|| {
            thread::scope(|s| {
                for chunk in &chunks {
                    s.spawn(|| {
                        let index = CityIndex::open(
                            StorageLayout::Ndjson {
                                paths: vec![ndjson_root.clone()],
                            },
                            index_path,
                        )
                        .expect("index should open");

                        for item in *chunk {
                            let target_bytes = index
                                .read_feature_bytes(black_box(&item.target_ref))
                                .expect("read_feature_bytes should succeed");
                            black_box(&target_bytes);

                            for adj_ref in &item.adjacent_refs {
                                let adj_bytes = index
                                    .read_feature_bytes(black_box(adj_ref))
                                    .expect("read_feature_bytes should succeed");
                                black_box(&adj_bytes);
                            }
                        }
                    });
                }
            });
        });
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10).measurement_time(Duration::from_secs(120));
    targets = bench_parallel_get, bench_parallel_io_only
}
criterion_main!(benches);
