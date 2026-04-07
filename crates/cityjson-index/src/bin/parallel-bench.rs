use std::collections::HashMap;
use std::fs;
use std::hint::black_box;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use cjindex::realistic_workload::{WORKLOAD_SHUFFLE_SEED, seeded_shuffle};
use cjindex::{CityIndex, IndexedFeatureRef, StorageLayout};
use cjlib::Result;
use cjlib::json::staged;

const WORKER_COUNT: usize = 6;
const ADJACENTS_PER_BUILDING: usize = 4;

struct BuildingWork {
    target_ref: IndexedFeatureRef,
    adjacent_refs: Vec<IndexedFeatureRef>,
}

struct BaselineBuildingWork {
    target_path: PathBuf,
    target_metadata_dir: PathBuf,
    adjacent_paths: Vec<(PathBuf, PathBuf)>,
}

fn collect_all_refs(index: &CityIndex) -> Result<Vec<IndexedFeatureRef>> {
    let mut all_refs = Vec::new();
    let mut offset = 0;
    loop {
        let page = index.feature_ref_page(offset, 1000)?;
        if page.is_empty() {
            break;
        }
        offset += page.len();
        all_refs.extend(page);
    }
    Ok(all_refs)
}

fn reads_as_f64(total_reads: usize) -> f64 {
    f64::from(u32::try_from(total_reads).expect("bench total reads should fit in u32"))
}

fn main() -> Result<()> {
    let feature_files_root = PathBuf::from("tests/data/feature-files");
    let ndjson_root = PathBuf::from("tests/data/ndjson");

    if !feature_files_root.exists() || !ndjson_root.exists() {
        eprintln!(
            "tests/data/{{feature-files,ndjson}} must exist; run `just prep-test-data` first"
        );
        std::process::exit(1);
    }

    println!("=== Setup ===");

    let setup_start = Instant::now();

    // Build feature-files index
    let ff_index_path = std::env::temp_dir().join("cjindex-parallel-bench-ff.sqlite");
    let _ = fs::remove_file(&ff_index_path);
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

    // Build ndjson index
    let ndjson_index_path = std::env::temp_dir().join("cjindex-parallel-bench-ndjson.sqlite");
    let _ = fs::remove_file(&ndjson_index_path);
    let mut ndjson_index = CityIndex::open(
        StorageLayout::Ndjson {
            paths: vec![ndjson_root.clone()],
        },
        &ndjson_index_path,
    )?;
    ndjson_index.reindex()?;
    let ndjson_refs = collect_all_refs(&ndjson_index)?;
    drop(ndjson_index);

    // Build id -> ref maps
    let ff_ref_by_id: HashMap<String, IndexedFeatureRef> = ff_refs
        .iter()
        .map(|r| (r.feature_id.clone(), r.clone()))
        .collect();
    let ndjson_ref_by_id: HashMap<String, IndexedFeatureRef> = ndjson_refs
        .iter()
        .map(|r| (r.feature_id.clone(), r.clone()))
        .collect();

    // All IDs present in both layouts
    let mut target_ids: Vec<String> = ff_ref_by_id
        .keys()
        .filter(|id| ndjson_ref_by_id.contains_key(*id))
        .cloned()
        .collect();
    target_ids.sort();
    seeded_shuffle(&mut target_ids, WORKLOAD_SHUFFLE_SEED);

    let n = target_ids.len();
    let mut adj_order = target_ids.clone();
    seeded_shuffle(
        &mut adj_order,
        WORKLOAD_SHUFFLE_SEED ^ 0xdead_beef_cafe_babe,
    );

    // Build work items
    let mut cjindex_work = Vec::with_capacity(n);
    let mut baseline_work = Vec::with_capacity(n);
    let mut metadata_cache: HashMap<PathBuf, Vec<u8>> = HashMap::new();

    for (i, target_id) in target_ids.iter().enumerate() {
        let ff_target_ref = ff_ref_by_id[target_id].clone();
        let ndjson_target_ref = ndjson_ref_by_id[target_id].clone();

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

    drop(ff_index);
    let _ = fs::remove_file(&ff_index_path);

    let metadata_cache = Arc::new(metadata_cache);
    let total_reads = n * (1 + ADJACENTS_PER_BUILDING);
    let total_reads_f64 = reads_as_f64(total_reads);

    println!(
        "Setup: {:.2}s ({} buildings, {} reads/building, {} total reads, {} workers)",
        setup_start.elapsed().as_secs_f64(),
        n,
        1 + ADJACENTS_PER_BUILDING,
        total_reads,
        WORKER_COUNT,
    );

    // --- Baseline ---
    println!("\n=== Baseline (feature-files, direct fs::read + deser) ===");
    {
        let chunk_size = baseline_work.len().div_ceil(WORKER_COUNT);
        let chunks: Vec<&[BaselineBuildingWork]> = baseline_work.chunks(chunk_size).collect();

        let start = Instant::now();
        thread::scope(|s| {
            for chunk in &chunks {
                let metadata_cache = metadata_cache.clone();
                s.spawn(move || {
                    for item in *chunk {
                        let target_bytes = fs::read(&item.target_path).expect("target read failed");
                        let target_meta = metadata_cache
                            .get(&item.target_metadata_dir)
                            .expect("target metadata missing");
                        let target_model =
                            staged::from_feature_slice_with_base(&target_bytes, target_meta)
                                .expect("target deser failed");
                        black_box(&target_model);

                        for (adj_path, adj_meta_dir) in &item.adjacent_paths {
                            let adj_bytes = fs::read(adj_path).expect("adj read failed");
                            let adj_meta = metadata_cache
                                .get(adj_meta_dir)
                                .expect("adj metadata missing");
                            let adj_model =
                                staged::from_feature_slice_with_base(&adj_bytes, adj_meta)
                                    .expect("adj deser failed");
                            black_box(adj_model);
                        }
                    }
                });
            }
        });
        let elapsed = start.elapsed();
        println!(
            "Time: {:.3}s ({:.1} reads/sec, {:.4}ms/read)",
            elapsed.as_secs_f64(),
            total_reads_f64 / elapsed.as_secs_f64(),
            elapsed.as_secs_f64() / total_reads_f64 * 1000.0,
        );
    }

    // --- cjindex ndjson ---
    println!("\n=== cjindex (ndjson, read_feature via refs) ===");
    {
        let chunk_size = cjindex_work.len().div_ceil(WORKER_COUNT);
        let chunks: Vec<&[BuildingWork]> = cjindex_work.chunks(chunk_size).collect();

        let start = Instant::now();
        thread::scope(|s| {
            for chunk in &chunks {
                s.spawn(|| {
                    let index = CityIndex::open(
                        StorageLayout::Ndjson {
                            paths: vec![ndjson_root.clone()],
                        },
                        &ndjson_index_path,
                    )
                    .expect("index open failed");

                    for item in *chunk {
                        let target_model = index
                            .read_feature(black_box(&item.target_ref))
                            .expect("read_feature failed");
                        black_box(&target_model);

                        for adj_ref in &item.adjacent_refs {
                            let adj_model = index
                                .read_feature(black_box(adj_ref))
                                .expect("read_feature failed");
                            black_box(adj_model);
                        }
                    }
                });
            }
        });
        let elapsed = start.elapsed();
        println!(
            "Time: {:.3}s ({:.1} reads/sec, {:.4}ms/read)",
            elapsed.as_secs_f64(),
            total_reads_f64 / elapsed.as_secs_f64(),
            elapsed.as_secs_f64() / total_reads_f64 * 1000.0,
        );
    }

    let _ = fs::remove_file(&ndjson_index_path);

    Ok(())
}
