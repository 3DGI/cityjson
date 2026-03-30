use std::env;
use std::path::PathBuf;
use std::time::Instant;

use cjindex::{BBox, CityIndex, resolve_dataset, realistic_workload};
use cjlib::Result;
use rusqlite::Connection;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: perf-test <DATASET_DIR>");
        eprintln!("  DATASET_DIR: Path to the cjindex dataset (auto-detects layout)");
        std::process::exit(1);
    }

    let dataset_dir = PathBuf::from(&args[1]);

    println!("Loading dataset from: {}", dataset_dir.display());

    let dataset = resolve_dataset(&dataset_dir, None)?;
    println!("Storage layout: {}", dataset.layout.as_str());
    println!("Index path: {}", dataset.index_path.display());

    // Open and build index if needed
    let mut index = CityIndex::open(dataset.storage_layout(), &dataset.index_path)?;

    println!("\nReindexing...");
    let reindex_start = Instant::now();
    index.reindex()?;
    let reindex_duration = reindex_start.elapsed();
    println!("Reindex took: {:.2}s", reindex_duration.as_secs_f64());

    // Build test workload
    println!("\nBuilding test workload...");

    let workload = match realistic_workload::build_realistic_workload(dataset.dataset_root.as_path()) {
        Ok(wl) => {
            println!("Using realistic workload from feature-files layout");
            wl
        }
        Err(_) => {
            println!("Sampling IDs and bboxes from SQLite index...");
            sample_ids_and_bboxes(&dataset.index_path)?
        }
    };

    let test_ids = &workload.get_ids;
    let test_bboxes = &workload.query_bboxes;

    println!("Test workload has {} get IDs and {} bbox queries", test_ids.len(), test_bboxes.len());

    if test_ids.is_empty() || test_bboxes.is_empty() {
        eprintln!("No features found in dataset!");
        std::process::exit(1);
    }

    println!("\n--- GET Performance Test ---");
    println!("Testing {} get operations...", test_ids.len());

    let warmup_start = Instant::now();
    for id in test_ids[0..test_ids.len().min(10)].iter() {
        let _ = index.get(id)?;
    }
    let warmup_duration = warmup_start.elapsed();
    println!("Warmup (10 ops): {:.4}s ({:.4}ms/op)",
        warmup_duration.as_secs_f64(),
        warmup_duration.as_secs_f64() / 10.0 * 1000.0);

    let get_start = Instant::now();
    let mut get_count = 0;
    for id in test_ids.iter() {
        let _ = index.get(id)?;
        get_count += 1;
    }
    let get_duration = get_start.elapsed();

    println!("Measured ({} ops): {:.4}s", get_count, get_duration.as_secs_f64());
    println!("  - Total: {:.2}s", get_duration.as_secs_f64());
    println!("  - Per operation: {:.4}ms", get_duration.as_secs_f64() / get_count as f64 * 1000.0);
    println!("  - Throughput: {:.0} ops/sec", get_count as f64 / get_duration.as_secs_f64());

    println!("\n--- QUERY (BBox) Performance Test ---");
    println!("Testing {} query operations...", test_bboxes.len());

    let query_warmup_start = Instant::now();
    for bbox in test_bboxes[0..test_bboxes.len().min(3)].iter() {
        let results = index.query(bbox)?;
        let _ = results.len();
    }
    let query_warmup_duration = query_warmup_start.elapsed();
    println!("Warmup (3 ops): {:.4}s ({:.4}ms/op)",
        query_warmup_duration.as_secs_f64(),
        query_warmup_duration.as_secs_f64() / 3.0 * 1000.0);

    let query_start = Instant::now();
    let mut query_count = 0;
    let mut total_results = 0;
    for bbox in test_bboxes.iter() {
        let results = index.query(bbox)?;
        total_results += results.len();
        query_count += 1;
    }
    let query_duration = query_start.elapsed();

    println!("Measured ({} ops): {:.4}s", query_count, query_duration.as_secs_f64());
    println!("  - Total: {:.2}s", query_duration.as_secs_f64());
    println!("  - Per operation: {:.4}ms", query_duration.as_secs_f64() / query_count as f64 * 1000.0);
    println!("  - Throughput: {:.0} ops/sec", query_count as f64 / query_duration.as_secs_f64());
    println!("  - Total results returned: {}", total_results);
    println!("  - Avg results per query: {:.1}", total_results as f64 / query_count as f64);

    println!("\n--- Summary ---");
    println!("GET:   {:.4}ms/op ({:.0} ops/sec)",
        get_duration.as_secs_f64() / get_count as f64 * 1000.0,
        get_count as f64 / get_duration.as_secs_f64());
    println!("QUERY: {:.4}ms/op ({:.0} ops/sec)",
        query_duration.as_secs_f64() / query_count as f64 * 1000.0,
        query_count as f64 / query_duration.as_secs_f64());

    Ok(())
}

fn sample_ids_and_bboxes(index_path: &PathBuf) -> Result<realistic_workload::RealisticWorkload> {
    let conn = Connection::open(index_path)
        .map_err(|e| cjlib::Error::Import(format!("Failed to open index: {}", e)))?;

    // Sample up to 1000 feature IDs and up to 100 bboxes from the database
    let mut stmt = conn.prepare(
        "SELECT feature_id, min_x, max_x, min_y, max_y FROM feature_bbox JOIN bbox_map ON feature_rowid = feature_rowid LIMIT 1000"
    ).map_err(|e| cjlib::Error::Import(format!("Failed to prepare query: {}", e)))?;

    let mut ids = Vec::new();
    let mut bboxes = Vec::new();

    let rows = stmt.query_map([], |row| {
        let id: String = row.get(0)?;
        let min_x: f64 = row.get(1)?;
        let max_x: f64 = row.get(2)?;
        let min_y: f64 = row.get(3)?;
        let max_y: f64 = row.get(4)?;
        Ok((id, BBox { min_x, max_x, min_y, max_y }))
    }).map_err(|e| cjlib::Error::Import(format!("Failed to query rows: {}", e)))?;

    for row in rows {
        let (id, bbox) = row.map_err(|e| cjlib::Error::Import(format!("Failed to read row: {}", e)))?;
        ids.push(id);
        bboxes.push(bbox);
    }

    // If we don't have enough bboxes, expand them to cover different regions
    if bboxes.len() < 100 && !bboxes.is_empty() {
        let base_bbox = bboxes[0];
        let width = (base_bbox.max_x - base_bbox.min_x) / 2.0;
        let height = (base_bbox.max_y - base_bbox.min_y) / 2.0;

        for i in 0..100 {
            let offset_x = (i as f64 * 0.5) * width;
            let offset_y = ((i as f64 * 0.3) % 10.0) * height;
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
