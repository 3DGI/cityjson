use std::env;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use cityjson_index::{BBox, CityIndex, realistic_workload, resolve_dataset};
use cityjson_lib::Result;
use rusqlite::Connection;

fn main() -> Result<()> {
    let dataset_dir = parse_dataset_dir();

    println!("Loading dataset from: {}", dataset_dir.display());

    let dataset = resolve_dataset(&dataset_dir, None)?;
    println!("Storage layout: {}", dataset.layout.as_str());
    println!("Index path: {}", dataset.index_path.display());

    let mut index = CityIndex::open(dataset.storage_layout(), &dataset.index_path)?;
    let reindex_duration = reindex(&mut index)?;
    println!("Reindex took: {:.2}s", reindex_duration.as_secs_f64());

    let workload = load_workload(&dataset.index_path, dataset.dataset_root.as_path())?;
    println!(
        "Test workload has {} get IDs and {} bbox queries",
        workload.get_ids.len(),
        workload.query_bboxes.len()
    );

    if workload.get_ids.is_empty() || workload.query_bboxes.is_empty() {
        eprintln!("No features found in dataset!");
        std::process::exit(1);
    }

    let get_stats = run_get_benchmark(&index, &workload.get_ids)?;
    let query_stats = run_query_benchmark(&index, &workload.query_bboxes)?;

    println!("\n--- Summary ---");
    println!(
        "GET:   {:.4}ms/op ({:.0} ops/sec)",
        per_operation_ms(get_stats.duration, get_stats.count),
        throughput(get_stats.duration, get_stats.count)
    );
    println!(
        "QUERY: {:.4}ms/op ({:.0} ops/sec)",
        per_operation_ms(query_stats.duration, query_stats.count),
        throughput(query_stats.duration, query_stats.count)
    );

    Ok(())
}

fn parse_dataset_dir() -> PathBuf {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: perf-test <DATASET_DIR>");
        eprintln!("  DATASET_DIR: Path to the cjindex dataset (auto-detects layout)");
        std::process::exit(1);
    }
    PathBuf::from(&args[1])
}

fn reindex(index: &mut CityIndex) -> Result<Duration> {
    println!("\nReindexing...");
    let reindex_start = Instant::now();
    index.reindex()?;
    Ok(reindex_start.elapsed())
}

fn load_workload(
    index_path: &Path,
    dataset_root: &Path,
) -> Result<realistic_workload::RealisticWorkload> {
    println!("\nBuilding test workload...");

    if let Ok(workload) = realistic_workload::build_realistic_workload(dataset_root) {
        println!("Using realistic workload from feature-files layout");
        Ok(workload)
    } else {
        println!("Sampling IDs and bboxes from SQLite index...");
        sample_ids_and_bboxes(index_path)
    }
}

struct BenchmarkStats {
    duration: Duration,
    count: usize,
}

struct QueryBenchmarkStats {
    duration: Duration,
    count: usize,
}

fn run_get_benchmark(index: &CityIndex, test_ids: &[String]) -> Result<BenchmarkStats> {
    println!("\n--- GET Performance Test ---");
    println!("Testing {} get operations...", test_ids.len());

    let warmup_count = test_ids.len().min(10);
    let warmup_start = Instant::now();
    for id in &test_ids[..warmup_count] {
        let _ = index.get(id)?;
    }
    let warmup_duration = warmup_start.elapsed();
    println!(
        "Warmup ({warmup_count} ops): {:.4}s ({:.4}ms/op)",
        warmup_duration.as_secs_f64(),
        per_operation_ms(warmup_duration, warmup_count)
    );

    let get_start = Instant::now();
    for id in test_ids {
        let _ = index.get(id)?;
    }
    let get_duration = get_start.elapsed();
    let get_count = test_ids.len();

    println!(
        "Measured ({get_count} ops): {:.4}s",
        get_duration.as_secs_f64()
    );
    println!("  - Total: {:.2}s", get_duration.as_secs_f64());
    println!(
        "  - Per operation: {:.4}ms",
        per_operation_ms(get_duration, get_count)
    );
    println!(
        "  - Throughput: {:.0} ops/sec",
        throughput(get_duration, get_count)
    );

    Ok(BenchmarkStats {
        duration: get_duration,
        count: get_count,
    })
}

fn run_query_benchmark(index: &CityIndex, test_bboxes: &[BBox]) -> Result<QueryBenchmarkStats> {
    println!("\n--- QUERY (BBox) Performance Test ---");
    println!("Testing {} query operations...", test_bboxes.len());

    let warmup_count = test_bboxes.len().min(3);
    let query_warmup_start = Instant::now();
    for bbox in &test_bboxes[..warmup_count] {
        let results = index.query(bbox)?;
        let _ = results.len();
    }
    let query_warmup_duration = query_warmup_start.elapsed();
    println!(
        "Warmup ({warmup_count} ops): {:.4}s ({:.4}ms/op)",
        query_warmup_duration.as_secs_f64(),
        per_operation_ms(query_warmup_duration, warmup_count)
    );

    let query_start = Instant::now();
    let mut total_results = 0;
    for bbox in test_bboxes {
        let results = index.query(bbox)?;
        total_results += results.len();
    }
    let query_duration = query_start.elapsed();
    let query_count = test_bboxes.len();

    println!(
        "Measured ({query_count} ops): {:.4}s",
        query_duration.as_secs_f64()
    );
    println!("  - Total: {:.2}s", query_duration.as_secs_f64());
    println!(
        "  - Per operation: {:.4}ms",
        per_operation_ms(query_duration, query_count)
    );
    println!(
        "  - Throughput: {:.0} ops/sec",
        throughput(query_duration, query_count)
    );
    println!("  - Total results returned: {total_results}");
    println!(
        "  - Avg results per query: {:.1}",
        count_to_f64(total_results) / count_to_f64(query_count)
    );

    Ok(QueryBenchmarkStats {
        duration: query_duration,
        count: query_count,
    })
}

fn per_operation_ms(duration: Duration, count: usize) -> f64 {
    duration.as_secs_f64() / count_to_f64(count) * 1000.0
}

fn throughput(duration: Duration, count: usize) -> f64 {
    count_to_f64(count) / duration.as_secs_f64()
}

fn count_to_f64(count: usize) -> f64 {
    f64::from(u32::try_from(count).expect("benchmark count fits in u32"))
}

fn sample_ids_and_bboxes(index_path: &Path) -> Result<realistic_workload::RealisticWorkload> {
    let conn = Connection::open(index_path)
        .map_err(|error| cityjson_lib::Error::Import(format!("Failed to open index: {error}")))?;

    let mut stmt = conn
        .prepare(
            "SELECT feature_id, min_x, max_x, min_y, max_y FROM feature_bbox JOIN bbox_map ON feature_rowid = feature_rowid LIMIT 1000",
        )
        .map_err(|error| cityjson_lib::Error::Import(format!("Failed to prepare query: {error}")))?;

    let mut ids = Vec::new();
    let mut bboxes = Vec::new();

    let rows = stmt
        .query_map([], |row| {
            let id: String = row.get(0)?;
            let min_x: f64 = row.get(1)?;
            let max_x: f64 = row.get(2)?;
            let min_y: f64 = row.get(3)?;
            let max_y: f64 = row.get(4)?;
            Ok((
                id,
                BBox {
                    min_x,
                    max_x,
                    min_y,
                    max_y,
                },
            ))
        })
        .map_err(|error| cityjson_lib::Error::Import(format!("Failed to query rows: {error}")))?;

    for row in rows {
        let (id, bbox) = row
            .map_err(|error| cityjson_lib::Error::Import(format!("Failed to read row: {error}")))?;
        ids.push(id);
        bboxes.push(bbox);
    }

    if bboxes.len() < 100 && !bboxes.is_empty() {
        let base_bbox = bboxes[0];
        let width = (base_bbox.max_x - base_bbox.min_x) / 2.0;
        let height = (base_bbox.max_y - base_bbox.min_y) / 2.0;

        for i in 0_u32..100 {
            let offset_x = (f64::from(i) * 0.5) * width;
            let offset_y = ((f64::from(i) * 0.3) % 10.0) * height;
            bboxes.push(BBox {
                min_x: (base_bbox.min_x + offset_x).max(-180.0),
                max_x: (base_bbox.max_x + offset_x).min(180.0),
                min_y: (base_bbox.min_y + offset_y).max(-90.0),
                max_y: (base_bbox.max_y + offset_y).min(90.0),
            });
        }
    }

    Ok(realistic_workload::RealisticWorkload {
        get_ids: ids,
        query_bboxes: bboxes,
    })
}
