#![allow(clippy::all, clippy::pedantic)]

use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::hint::black_box;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use cjindex::realistic_workload::{QUERY_BATCH_COUNT, build_realistic_workload};
use cjindex::{BBox, CityIndex, StorageLayout};
use cjlib::{Error, Result};
use rusqlite::{Connection, OptionalExtension, params};

#[allow(dead_code)]
#[path = "../../tests/common/data_prep.rs"]
mod data_prep;

const WARMUP_ROUNDS: usize = 2;
const MEASURE_ROUNDS: usize = 5;

fn main() -> Result<()> {
    let datasets = prepared_datasets()?;
    let workload = build_realistic_workload(&datasets.feature_files)?;

    println!("# Backend Read Performance Investigation");
    println!();
    println!("## Commands");
    println!();
    println!("- `cargo run --release --bin investigate-read-performance`");
    println!("- realistic workload source: `cjindex::realistic_workload`");
    println!();

    let feature_files = prepare_layout(
        LayoutKind::FeatureFiles,
        datasets.feature_files.clone(),
        &workload.get_ids,
        &workload.query_bboxes,
    )?;
    let cityjson = prepare_layout(
        LayoutKind::CityJson,
        datasets.cityjson.clone(),
        &workload.get_ids,
        &workload.query_bboxes,
    )?;
    let ndjson = prepare_layout(
        LayoutKind::Ndjson,
        datasets.ndjson.clone(),
        &workload.get_ids,
        &workload.query_bboxes,
    )?;

    let layouts = [&feature_files, &cityjson, &ndjson];
    print_corpus_summary(&layouts);
    print_workload_shape("Get Workload", |layout| &layout.get_shape, &layouts);
    print_workload_shape(
        "BBox Workload Sweep",
        |layout| &layout.query_sweep_shape,
        &layouts,
    );
    print_workload_shape(
        "BBox Benchmark Batch",
        |layout| &layout.query_batch_shape,
        &layouts,
    );

    let get_timings = layouts
        .iter()
        .map(|layout| Ok((*layout, measure_get(layout)?)))
        .collect::<Result<Vec<_>>>()?;
    print_stage_timings("Get Timings", &get_timings, |layout| &layout.get_shape);

    let query_timings = layouts
        .iter()
        .map(|layout| Ok((*layout, measure_query(layout)?)))
        .collect::<Result<Vec<_>>>()?;
    print_stage_timings("BBox Query Timings", &query_timings, |layout| {
        &layout.query_batch_shape
    });

    print_findings(
        &feature_files,
        &cityjson,
        &ndjson,
        &get_timings,
        &query_timings,
    );

    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LayoutKind {
    FeatureFiles,
    CityJson,
    Ndjson,
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

struct PreparedLayout {
    kind: LayoutKind,
    root: PathBuf,
    index_path: PathBuf,
    source_count: usize,
    feature_count: usize,
    get_ids: Vec<String>,
    query_bboxes: Vec<BBox>,
    get_locations: Vec<LocationSpec>,
    query_locations: Vec<Vec<LocationSpec>>,
    get_shape: WorkloadShape,
    query_sweep_shape: WorkloadShape,
    query_batch_shape: WorkloadShape,
}

#[derive(Clone)]
struct LocationSpec {
    source_id: i64,
    source_path: PathBuf,
    offset: u64,
    length: u64,
    vertices_offset: Option<u64>,
    vertices_length: Option<u64>,
    member_ranges: Option<Vec<MemberRange>>,
}

#[derive(Clone, serde::Deserialize)]
struct MemberRange {
    id: String,
    offset: u64,
    length: u64,
}

#[derive(Default)]
struct WorkloadShape {
    result_count: usize,
    unique_sources: usize,
    total_primary_bytes: u64,
    average_primary_bytes: f64,
    p50_primary_bytes: u64,
    p95_primary_bytes: u64,
    cache_hits: usize,
    cache_misses: usize,
    total_secondary_bytes_on_miss: u64,
    per_query_hits: Vec<usize>,
}

struct StageTimings {
    lookup_only: TimingSummary,
    read_only: TimingSummary,
    full: TimingSummary,
}

#[derive(Clone)]
struct TimingSummary {
    median: Duration,
    min: Duration,
    max: Duration,
}

fn prepared_datasets() -> Result<data_prep::PreparedDatasets> {
    let output_root = Path::new(data_prep::DEFAULT_OUTPUT_ROOT);
    let feature_files_root = output_root.join("feature-files");
    let cityjson_root = output_root.join("cityjson");
    let ndjson_root = output_root.join("ndjson");

    if feature_files_root.exists() && cityjson_root.exists() && ndjson_root.exists() {
        return Ok(data_prep::PreparedDatasets {
            feature_files: feature_files_root,
            cityjson: cityjson_root,
            ndjson: ndjson_root,
        });
    }

    data_prep::prepare_test_sets(Path::new(data_prep::DEFAULT_INPUT_ROOT), output_root)
}

fn prepare_layout(
    kind: LayoutKind,
    root: PathBuf,
    get_ids: &[String],
    query_bboxes: &[BBox],
) -> Result<PreparedLayout> {
    let index_path = unique_temp_file(&format!("cjindex-investigate-{}", kind.label()), "sqlite");
    let mut index = CityIndex::open(kind.storage_layout(&root), &index_path)?;
    index.reindex()?;
    drop(index);

    let conn = Connection::open(&index_path).map_err(sqlite_error)?;
    let source_count = count_table_rows(&conn, "sources")?;
    let feature_count = count_table_rows(&conn, "features")?;
    let get_locations = get_ids
        .iter()
        .map(|id| {
            lookup_location_by_id(&conn, id)?.ok_or_else(|| {
                Error::Import(format!(
                    "missing indexed location for {id} in {}",
                    kind.label()
                ))
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let query_locations = query_bboxes
        .iter()
        .map(|bbox| lookup_locations_by_bbox(&conn, bbox))
        .collect::<Result<Vec<_>>>()?;
    let get_shape = summarize_workload(&get_locations);
    let query_sweep_shape = summarize_workload(
        &query_locations
            .iter()
            .flatten()
            .cloned()
            .collect::<Vec<_>>(),
    )
    .with_query_hits(query_locations.iter().map(Vec::len).collect());
    let query_batch_shape = summarize_workload(
        &query_locations
            .iter()
            .take(QUERY_BATCH_COUNT.min(query_locations.len()))
            .flatten()
            .cloned()
            .collect::<Vec<_>>(),
    );

    Ok(PreparedLayout {
        kind,
        root,
        index_path,
        source_count,
        feature_count,
        get_ids: get_ids.to_vec(),
        query_bboxes: query_bboxes.to_vec(),
        get_locations,
        query_locations,
        get_shape,
        query_sweep_shape,
        query_batch_shape,
    })
}

impl WorkloadShape {
    fn with_query_hits(mut self, per_query_hits: Vec<usize>) -> Self {
        self.per_query_hits = per_query_hits;
        self
    }
}

fn summarize_workload(locations: &[LocationSpec]) -> WorkloadShape {
    let mut primary_lengths = locations
        .iter()
        .map(total_primary_bytes_for_location)
        .collect::<Vec<_>>();
    primary_lengths.sort_unstable();
    let total_primary_bytes = primary_lengths.iter().sum();
    let unique_sources = locations
        .iter()
        .map(|loc| loc.source_id)
        .collect::<BTreeSet<_>>()
        .len();

    let mut seen_sources = HashSet::new();
    let mut cache_hits = 0;
    let mut cache_misses = 0;
    let mut total_secondary_bytes_on_miss = 0;
    for loc in locations {
        if let Some(vertices_length) = loc.vertices_length {
            if seen_sources.insert(loc.source_id) {
                cache_misses += 1;
                total_secondary_bytes_on_miss += vertices_length;
            } else {
                cache_hits += 1;
            }
        }
    }

    WorkloadShape {
        result_count: locations.len(),
        unique_sources,
        total_primary_bytes,
        average_primary_bytes: total_primary_bytes as f64 / locations.len().max(1) as f64,
        p50_primary_bytes: percentile(&primary_lengths, 0.50),
        p95_primary_bytes: percentile(&primary_lengths, 0.95),
        cache_hits,
        cache_misses,
        total_secondary_bytes_on_miss,
        per_query_hits: Vec::new(),
    }
}

fn percentile(sorted: &[u64], percentile: f64) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let index = ((sorted.len() - 1) as f64 * percentile).round() as usize;
    sorted[index]
}

fn measure_get(layout: &PreparedLayout) -> Result<StageTimings> {
    let lookup_conn = Connection::open(&layout.index_path).map_err(sqlite_error)?;
    let mut lookup_stmt = lookup_conn
        .prepare(
            r#"
            SELECT
                s.id,
                f.path,
                f.offset,
                f.length,
                s.vertices_offset,
                s.vertices_length,
                f.member_ranges
            FROM features AS f
            JOIN sources AS s ON s.id = f.source_id
            WHERE f.feature_id = ?1
            "#,
        )
        .map_err(sqlite_error)?;
    let lookup_only =
        measure_rounds(|| measure_get_lookup_only(&mut lookup_stmt, &layout.get_ids))?;

    let mut cityjson_vertices_cache = HashSet::new();
    let read_only = measure_rounds(|| {
        read_locations_only(
            layout.kind,
            &layout.get_locations,
            &mut cityjson_vertices_cache,
        )
    })?;

    let full_index = CityIndex::open(layout.kind.storage_layout(&layout.root), &layout.index_path)?;
    let full = measure_rounds(|| measure_get_full(&full_index, &layout.get_ids))?;

    Ok(StageTimings {
        lookup_only,
        read_only,
        full,
    })
}

fn measure_query(layout: &PreparedLayout) -> Result<StageTimings> {
    let batch_len = QUERY_BATCH_COUNT.min(layout.query_bboxes.len());
    let query_bboxes = &layout.query_bboxes[..batch_len];
    let query_locations = &layout.query_locations[..batch_len];
    let lookup_conn = Connection::open(&layout.index_path).map_err(sqlite_error)?;
    let mut lookup_stmt = lookup_conn
        .prepare(
            r#"
            SELECT DISTINCT
                s.id,
                f.path,
                f.offset,
                f.length,
                s.vertices_offset,
                s.vertices_length,
                f.member_ranges
            FROM feature_bbox AS fb
            JOIN bbox_map AS bm ON bm.feature_rowid = fb.feature_rowid
            JOIN features AS f ON f.feature_id = bm.feature_id
            JOIN sources AS s ON s.id = f.source_id
            WHERE fb.min_x <= ?2
              AND fb.max_x >= ?1
              AND fb.min_y <= ?4
              AND fb.max_y >= ?3
            ORDER BY bm.feature_id
            "#,
        )
        .map_err(sqlite_error)?;
    let lookup_only = measure_rounds(|| measure_query_lookup_only(&mut lookup_stmt, query_bboxes))?;

    let mut cityjson_vertices_cache = HashSet::new();
    let read_only = measure_rounds(|| {
        measure_query_read_only(layout.kind, query_locations, &mut cityjson_vertices_cache)
    })?;

    let full_index = CityIndex::open(layout.kind.storage_layout(&layout.root), &layout.index_path)?;
    let full = measure_rounds(|| measure_query_full(&full_index, query_bboxes))?;

    Ok(StageTimings {
        lookup_only,
        read_only,
        full,
    })
}

fn measure_rounds<F>(mut batch: F) -> Result<TimingSummary>
where
    F: FnMut() -> Result<usize>,
{
    for _ in 0..WARMUP_ROUNDS {
        black_box(batch()?);
    }

    let mut samples = Vec::with_capacity(MEASURE_ROUNDS);
    for _ in 0..MEASURE_ROUNDS {
        let start = Instant::now();
        let checksum = batch()?;
        let elapsed = start.elapsed();
        black_box(checksum);
        samples.push(elapsed);
    }

    samples.sort_unstable();
    Ok(TimingSummary {
        median: samples[samples.len() / 2],
        min: *samples.first().expect("timing samples should exist"),
        max: *samples.last().expect("timing samples should exist"),
    })
}

fn measure_get_lookup_only(
    stmt: &mut rusqlite::Statement<'_>,
    get_ids: &[String],
) -> Result<usize> {
    let mut checksum = 0usize;
    for feature_id in get_ids {
        let resolved = stmt
            .query_row(params![feature_id], location_from_row)
            .optional()
            .map_err(sqlite_error)?
            .ok_or_else(|| Error::Import("lookup-only round could not resolve location".into()))?;
        checksum ^= checksum_location(&resolved);
    }
    Ok(checksum)
}

fn measure_query_lookup_only(
    stmt: &mut rusqlite::Statement<'_>,
    query_bboxes: &[BBox],
) -> Result<usize> {
    let mut checksum = 0usize;
    for bbox in query_bboxes {
        let rows = stmt
            .query_map(
                params![bbox.min_x, bbox.max_x, bbox.min_y, bbox.max_y],
                location_from_row,
            )
            .map_err(sqlite_error)?;
        let locations = rows
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(sqlite_error)?;
        for loc in locations {
            checksum ^= checksum_location(&loc);
        }
    }
    Ok(checksum)
}

fn measure_query_read_only(
    kind: LayoutKind,
    query_locations: &[Vec<LocationSpec>],
    cityjson_vertices_cache: &mut HashSet<i64>,
) -> Result<usize> {
    let mut checksum = 0usize;
    for locations in query_locations {
        checksum ^= read_locations_only(kind, locations, cityjson_vertices_cache)?;
    }
    Ok(checksum)
}

fn read_locations_only(
    kind: LayoutKind,
    locations: &[LocationSpec],
    cityjson_vertices_cache: &mut HashSet<i64>,
) -> Result<usize> {
    let mut checksum = 0usize;

    for loc in locations {
        match &loc.member_ranges {
            Some(member_ranges) => {
                for member_range in member_ranges {
                    let bytes = read_exact_range(
                        &loc.source_path,
                        member_range.offset,
                        member_range.length,
                    )?;
                    checksum ^= bytes.len();
                    checksum ^= member_range.id.len();
                }
            }
            None => {
                let bytes = read_exact_range(&loc.source_path, loc.offset, loc.length)?;
                checksum ^= bytes.len();
            }
        }

        if kind == LayoutKind::CityJson && cityjson_vertices_cache.insert(loc.source_id) {
            let vertices_offset = loc.vertices_offset.ok_or_else(|| {
                Error::Import("CityJSON read-only probe is missing vertices_offset".into())
            })?;
            let vertices_length = loc.vertices_length.ok_or_else(|| {
                Error::Import("CityJSON read-only probe is missing vertices_length".into())
            })?;
            let vertices = read_exact_range(&loc.source_path, vertices_offset, vertices_length)?;
            checksum ^= vertices.len();
        }
    }

    Ok(checksum)
}

fn measure_get_full(index: &CityIndex, get_ids: &[String]) -> Result<usize> {
    let mut checksum = 0usize;
    for feature_id in get_ids {
        let model = index
            .get(feature_id)?
            .ok_or_else(|| Error::Import(format!("full get is missing model {feature_id}")))?;
        black_box(model);
        checksum ^= 1;
    }
    Ok(checksum)
}

fn measure_query_full(index: &CityIndex, query_bboxes: &[BBox]) -> Result<usize> {
    let mut checksum = 0usize;
    for bbox in query_bboxes {
        let models = index.query(bbox)?;
        black_box(&models);
        checksum ^= models.len();
    }
    Ok(checksum)
}

fn lookup_location_by_id(conn: &Connection, id: &str) -> Result<Option<LocationSpec>> {
    conn.query_row(
        r#"
        SELECT
            s.id,
            f.path,
            f.offset,
            f.length,
            s.vertices_offset,
            s.vertices_length,
            f.member_ranges
        FROM features AS f
        JOIN sources AS s ON s.id = f.source_id
        WHERE f.feature_id = ?1
        "#,
        params![id],
        location_from_row,
    )
    .optional()
    .map_err(sqlite_error)
}

fn lookup_locations_by_bbox(conn: &Connection, bbox: &BBox) -> Result<Vec<LocationSpec>> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT DISTINCT
                s.id,
                f.path,
                f.offset,
                f.length,
                s.vertices_offset,
                s.vertices_length,
                f.member_ranges
            FROM feature_bbox AS fb
            JOIN bbox_map AS bm ON bm.feature_rowid = fb.feature_rowid
            JOIN features AS f ON f.feature_id = bm.feature_id
            JOIN sources AS s ON s.id = f.source_id
            WHERE fb.min_x <= ?2
              AND fb.max_x >= ?1
              AND fb.min_y <= ?4
              AND fb.max_y >= ?3
            ORDER BY bm.feature_id
            "#,
        )
        .map_err(sqlite_error)?;
    let rows = stmt
        .query_map(
            params![bbox.min_x, bbox.max_x, bbox.min_y, bbox.max_y],
            location_from_row,
        )
        .map_err(sqlite_error)?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(sqlite_error)
}

fn location_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<LocationSpec> {
    Ok(LocationSpec {
        source_id: row.get(0)?,
        source_path: PathBuf::from(row.get::<_, String>(1)?),
        offset: i64_to_u64(row.get::<_, i64>(2)?)?,
        length: i64_to_u64(row.get::<_, i64>(3)?)?,
        vertices_offset: row.get::<_, Option<i64>>(4)?.map(i64_to_u64).transpose()?,
        vertices_length: row.get::<_, Option<i64>>(5)?.map(i64_to_u64).transpose()?,
        member_ranges: row
            .get::<_, Option<String>>(6)?
            .map(|json| serde_json::from_str(&json))
            .transpose()
            .map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    6,
                    rusqlite::types::Type::Text,
                    Box::new(error),
                )
            })?,
    })
}

fn checksum_location(location: &LocationSpec) -> usize {
    let path_hash = location
        .source_path
        .to_string_lossy()
        .bytes()
        .fold(0usize, |acc, byte| {
            acc.wrapping_mul(131).wrapping_add(byte as usize)
        });
    path_hash ^ total_primary_bytes_for_location(location) as usize ^ location.offset as usize
}

fn total_primary_bytes_for_location(location: &LocationSpec) -> u64 {
    location
        .member_ranges
        .as_ref()
        .map(|member_ranges| {
            member_ranges
                .iter()
                .map(|member_range| member_range.length)
                .sum()
        })
        .unwrap_or(location.length)
}

fn read_exact_range(path: &Path, offset: u64, length: u64) -> Result<Vec<u8>> {
    let length = usize::try_from(length).map_err(|_| {
        Error::Import(format!(
            "range length does not fit usize for {}",
            path.display()
        ))
    })?;
    let mut file = fs::File::open(path)
        .map_err(|error| Error::Import(format!("failed to open {}: {error}", path.display())))?;
    file.seek(SeekFrom::Start(offset))
        .map_err(|error| Error::Import(format!("failed to seek {}: {error}", path.display())))?;
    let mut bytes = vec![0; length];
    file.read_exact(&mut bytes)
        .map_err(|error| Error::Import(format!("failed to read {}: {error}", path.display())))?;
    Ok(bytes)
}

fn print_corpus_summary(layouts: &[&PreparedLayout]) {
    println!("## Corpus Summary");
    println!();
    println!("| Backend | Indexed Sources | Indexed Features |");
    println!("| --- | ---: | ---: |");
    for layout in layouts {
        println!(
            "| `{}` | {} | {} |",
            layout.kind.label(),
            layout.source_count,
            layout.feature_count
        );
    }
    println!();
}

fn print_workload_shape(
    title: &str,
    shape: impl Fn(&PreparedLayout) -> &WorkloadShape,
    layouts: &[&PreparedLayout],
) {
    println!("## {title}");
    println!();
    println!(
        "| Backend | Reads | Unique Sources | Total Primary Bytes | Avg Primary Bytes | P50 Span | P95 Span | CityJSON Cache Misses | CityJSON Cache Hits | CityJSON Shared Vertices Bytes On First Touch |"
    );
    println!("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |");
    for layout in layouts {
        let shape = shape(layout);
        println!(
            "| `{}` | {} | {} | {} | {:.1} | {} | {} | {} | {} | {} |",
            layout.kind.label(),
            shape.result_count,
            shape.unique_sources,
            shape.total_primary_bytes,
            shape.average_primary_bytes,
            shape.p50_primary_bytes,
            shape.p95_primary_bytes,
            shape.cache_misses,
            shape.cache_hits,
            shape.total_secondary_bytes_on_miss
        );
    }
    println!();

    if let Some(feature_files_shape) = layouts.first().map(|layout| shape(layout)) {
        if !feature_files_shape.per_query_hits.is_empty() {
            let min_hits = feature_files_shape
                .per_query_hits
                .iter()
                .min()
                .copied()
                .unwrap_or(0);
            let max_hits = feature_files_shape
                .per_query_hits
                .iter()
                .max()
                .copied()
                .unwrap_or(0);
            let avg_hits = feature_files_shape.per_query_hits.iter().sum::<usize>() as f64
                / feature_files_shape.per_query_hits.len() as f64;
            println!(
                "BBox result counts from the canonical workload: min {}, avg {:.1}, max {}.",
                min_hits, avg_hits, max_hits
            );
            println!();
        }
    }
}

fn print_stage_timings(
    title: &str,
    timings: &[(&PreparedLayout, StageTimings)],
    shape: impl Fn(&PreparedLayout) -> &WorkloadShape,
) {
    println!("## {title}");
    println!();
    println!(
        "| Backend | Lookup Only | Read Only | Full | Estimated Remaining | Full Per Read | Full Sample Range |"
    );
    println!("| --- | ---: | ---: | ---: | ---: | ---: | ---: |");
    for (layout, timing) in timings {
        let reads = shape(layout).result_count.max(1);
        println!(
            "| `{}` | {} | {} | {} | {} | {} | {} to {} |",
            layout.kind.label(),
            format_duration(timing.lookup_only.median),
            format_duration(timing.read_only.median),
            format_duration(timing.full.median),
            format_duration(remaining_duration(
                timing.full.median,
                timing.lookup_only.median,
                timing.read_only.median
            )),
            format_duration(per_unit_duration(timing.full.median, reads)),
            format_duration(timing.full.min),
            format_duration(timing.full.max),
        );
    }
    println!();
}

fn print_findings(
    feature_files: &PreparedLayout,
    cityjson: &PreparedLayout,
    ndjson: &PreparedLayout,
    get_timings: &[(&PreparedLayout, StageTimings)],
    query_timings: &[(&PreparedLayout, StageTimings)],
) {
    let cityjson_get = timing_for(get_timings, LayoutKind::CityJson);
    let ndjson_get = timing_for(get_timings, LayoutKind::Ndjson);
    let cityjson_query = timing_for(query_timings, LayoutKind::CityJson);
    let ndjson_query = timing_for(query_timings, LayoutKind::Ndjson);

    println!("## Findings");
    println!();
    println!(
        "- `CityJSON get` reads much smaller primary spans than `NDJSON get`: {:.1} bytes on average versus {:.1} bytes.",
        cityjson.get_shape.average_primary_bytes, ndjson.get_shape.average_primary_bytes
    );
    println!(
        "- The `CityJSON get` workload touches {} source files across 1,000 reads, so the shared-vertices cache turns {} reads into hits after {} first touches.",
        cityjson.get_shape.unique_sources,
        cityjson.get_shape.cache_hits,
        cityjson.get_shape.cache_misses
    );
    println!(
        "- On the bbox workload, `CityJSON` still pays a heavier remaining end-to-end cost than `NDJSON`: {} estimated residual per 10-bbox batch versus {}.",
        format_duration(remaining_duration(
            cityjson_query.full.median,
            cityjson_query.lookup_only.median,
            cityjson_query.read_only.median
        )),
        format_duration(remaining_duration(
            ndjson_query.full.median,
            ndjson_query.lookup_only.median,
            ndjson_query.read_only.median
        ))
    );
    println!(
        "- After restoring full feature-package semantics, `CityJSON get` is slightly slower than `NDJSON get`: full batch {} versus `NDJSON` {}.",
        format_duration(cityjson_get.full.median),
        format_duration(ndjson_get.full.median)
    );
    println!(
        "- The realistic bbox workload returns {} features over 10 bboxes, so per-result query cost remains in the tens of microseconds even though per-bbox latency is in the hundred-millisecond range.",
        feature_files.query_batch_shape.result_count
    );
    println!(
        "- The full rotating bbox sweep covers {} tile-local windows and all {} `CityJSON` / `NDJSON` source files, while each measured Criterion batch still stays at 10 queries.",
        feature_files.query_bboxes.len(),
        cityjson.query_sweep_shape.unique_sources
    );
    println!();
}

fn timing_for<'a>(
    timings: &'a [(&PreparedLayout, StageTimings)],
    kind: LayoutKind,
) -> &'a StageTimings {
    timings
        .iter()
        .find_map(|(layout, timing)| (layout.kind == kind).then_some(timing))
        .expect("timing for layout should exist")
}

fn remaining_duration(full: Duration, lookup_only: Duration, read_only: Duration) -> Duration {
    full.saturating_sub(lookup_only).saturating_sub(read_only)
}

fn per_unit_duration(duration: Duration, units: usize) -> Duration {
    let units = units.max(1) as u128;
    Duration::from_nanos((duration.as_nanos() / units) as u64)
}

fn format_duration(duration: Duration) -> String {
    let nanos = duration.as_nanos();
    if nanos >= 1_000_000_000 {
        format!("{:.3} s", nanos as f64 / 1_000_000_000.0)
    } else if nanos >= 1_000_000 {
        format!("{:.3} ms", nanos as f64 / 1_000_000.0)
    } else if nanos >= 1_000 {
        format!("{:.3} us", nanos as f64 / 1_000.0)
    } else {
        format!("{nanos} ns")
    }
}

fn unique_temp_file(label: &str, suffix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time must be after the unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("cjindex-{label}-{unique}.{suffix}"));
    if path.exists() {
        fs::remove_file(&path).expect("temp file should be removable");
    }
    path
}

fn sqlite_error(error: rusqlite::Error) -> Error {
    Error::Import(error.to_string())
}

fn count_table_rows(conn: &Connection, table: &str) -> Result<usize> {
    let sql = format!("SELECT COUNT(*) FROM {table}");
    let count: i64 = conn
        .query_row(&sql, [], |row| row.get(0))
        .map_err(sqlite_error)?;
    usize::try_from(count)
        .map_err(|_| Error::Import(format!("table count for {table} does not fit in usize")))
}

fn i64_to_u64(value: i64) -> rusqlite::Result<u64> {
    u64::try_from(value).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(0, value))
}
