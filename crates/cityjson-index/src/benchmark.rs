use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use cityjson_lib::{Error, Result};
use clap::{Parser, ValueEnum};
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::profile;
use crate::{BBox, CityIndex, resolve_dataset};

const DEFAULT_CORPUS_ROOT: &str = "/home/balazs/Development/cityjson-corpus";
const DEFAULT_BASISVOORZIENING_ARTIFACT: &str =
    "artifacts/acquired/basisvoorziening-3d/2022/3d_volledig_84000_450000.city.json";
const DEFAULT_WORK_ROOT: &str = "target/benchmarks/basisvoorziening-3d";
const DEFAULT_SUBSET_SIZES: &[usize] = &[1_000, 5_000, 10_000, 25_000];

#[derive(Debug, Clone, Parser)]
#[command(
    name = "bench-index",
    about = "Run JSON-emitting CityJSON indexing benchmarks",
    long_about = r#"Run JSON-emitting CityJSON indexing benchmarks.

The benchmark runner prepares Basisvoorziening 3D inputs from the pinned corpus artifact, reuses the prepared datasets across the requested worker counts, and records one JSON object per measured operation.
"#
)]
pub struct BenchmarkCli {
    /// Emit machine-readable JSON output.
    #[arg(long)]
    pub json: bool,

    /// Root of the cityjson-corpus checkout.
    #[arg(long, default_value = DEFAULT_CORPUS_ROOT)]
    pub corpus_root: PathBuf,

    /// Benchmark work directory for prepared datasets.
    #[arg(long, default_value = DEFAULT_WORK_ROOT)]
    pub work_root: PathBuf,

    /// Override the pinned Basisvoorziening artifact path.
    #[arg(long)]
    pub artifact: Option<PathBuf>,

    /// Include a benchmark case.
    #[arg(long, value_enum)]
    pub case: Vec<BenchmarkCaseKind>,

    /// Worker counts to record for each dataset.
    #[arg(long, value_name = "WORKERS")]
    pub workers: Vec<usize>,

    /// Optional root directory containing additional Basisvoorziening tiles.
    #[arg(long)]
    pub multi_tile_root: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum BenchmarkCaseKind {
    SingleTileFull,
    SingleTileSubsets,
    MultiTile,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkReport {
    pub runs: Vec<BenchmarkOperationRecord>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkOperationRecord {
    pub dataset_label: String,
    pub source_artifact: PathBuf,
    pub prepared_dataset: PathBuf,
    pub subset_size: Option<usize>,
    pub byte_size: u64,
    pub worker_count: usize,
    pub operation: String,
    pub variant: Option<String>,
    pub elapsed_ns: u64,
    pub current_rss_bytes: u64,
    pub peak_rss_bytes: u64,
    pub feature_count: usize,
    pub source_count: usize,
    pub cityobject_count: usize,
    pub query_hit_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BenchmarkManifest {
    dataset_label: String,
    source_artifact: PathBuf,
    prepared_dataset: PathBuf,
    subset_size: Option<usize>,
    byte_size: u64,
    feature_count: usize,
    source_count: usize,
    cityobject_count: usize,
    dataset_bbox: BBox,
    representative_feature_ids: Vec<String>,
    query_windows: Vec<QueryWindow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QueryWindow {
    label: String,
    bbox: BBox,
}

#[derive(Debug, Clone)]
struct PreparedDataset {
    manifest: BenchmarkManifest,
}

pub fn run(cli: BenchmarkCli) -> Result<BenchmarkReport> {
    let artifact = cli
        .artifact
        .clone()
        .unwrap_or_else(|| cli.corpus_root.join(DEFAULT_BASISVOORZIENING_ARTIFACT));
    if !artifact.exists() {
        return Err(Error::Import(format!(
            "missing pinned Basisvoorziening 3D artifact {}; run `cd /home/balazs/Development/cityjson-corpus && just acquire-basisvoorziening-3d`",
            artifact.display()
        )));
    }

    let cases = if cli.case.is_empty() {
        vec![
            BenchmarkCaseKind::SingleTileFull,
            BenchmarkCaseKind::SingleTileSubsets,
        ]
    } else {
        cli.case.clone()
    };
    let worker_counts = worker_counts(cli.workers.clone());

    let mut runs = Vec::new();
    for case in cases {
        for dataset in prepare_case(&cli, case, &artifact)? {
            for worker_count in &worker_counts {
                runs.extend(run_dataset(&dataset, *worker_count)?);
            }
        }
    }

    Ok(BenchmarkReport { runs })
}

pub fn print_report(report: &BenchmarkReport, json: bool) -> Result<()> {
    if json {
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, report)
            .map_err(|error| Error::Import(error.to_string()))?;
        handle.write_all(b"\n")?;
        handle.flush()?;
        return Ok(());
    }

    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    for run in &report.runs {
        writeln!(
            handle,
            "{} worker={} op={} variant={} elapsed_ns={} rss_bytes={}/{} hits={}",
            run.dataset_label,
            run.worker_count,
            run.operation,
            run.variant.as_deref().unwrap_or("-"),
            run.elapsed_ns,
            run.current_rss_bytes,
            run.peak_rss_bytes,
            run.query_hit_count
                .map(|count| count.to_string())
                .unwrap_or_else(|| "-".to_owned())
        )?;
    }
    handle.flush()?;
    Ok(())
}

fn prepare_case(
    cli: &BenchmarkCli,
    case: BenchmarkCaseKind,
    artifact: &Path,
) -> Result<Vec<PreparedDataset>> {
    match case {
        BenchmarkCaseKind::SingleTileFull => Ok(vec![prepare_single_tile_dataset(
            cli,
            "single-tile-full",
            artifact,
            None,
        )?]),
        BenchmarkCaseKind::SingleTileSubsets => DEFAULT_SUBSET_SIZES
            .iter()
            .map(|subset_size| {
                prepare_single_tile_dataset(
                    cli,
                    &format!("single-tile-subset-{subset_size}"),
                    artifact,
                    Some(*subset_size),
                )
            })
            .collect(),
        BenchmarkCaseKind::MultiTile => prepare_multi_tile_dataset(cli),
    }
}

fn prepare_single_tile_dataset(
    cli: &BenchmarkCli,
    label: &str,
    artifact: &Path,
    subset_size: Option<usize>,
) -> Result<PreparedDataset> {
    let prepared_root = cli.work_root.join(label);
    reset_dir(&prepared_root)?;
    fs::create_dir_all(&prepared_root)?;

    let prepared_dataset = prepared_root.join("dataset.city.json");
    let (manifest, bytes) = match subset_size {
        None => {
            let bytes = fs::read(artifact)?;
            let manifest =
                manifest_for_cityjson_bytes(label, artifact, &prepared_root, None, &bytes)?;
            (manifest, bytes)
        }
        Some(limit) => {
            let bytes = fs::read(artifact)?;
            let mut document: Value =
                serde_json::from_slice(&bytes).map_err(|error| Error::Import(error.to_string()))?;
            let subset = subset_cityjson_document(&mut document, limit)?;
            let bytes = serde_json::to_vec_pretty(&subset)
                .map_err(|error| Error::Import(error.to_string()))?;
            let manifest = manifest_for_cityjson_bytes(
                label,
                artifact,
                &prepared_root,
                Some(extract_root_ids(&subset)?.len()),
                &bytes,
            )?;
            (manifest, bytes)
        }
    };

    fs::write(&prepared_dataset, &bytes)?;
    write_manifest(&prepared_root.join("benchmark-manifest.json"), &manifest)?;
    Ok(PreparedDataset { manifest })
}

fn prepare_multi_tile_dataset(cli: &BenchmarkCli) -> Result<Vec<PreparedDataset>> {
    let multi_root = cli.multi_tile_root.as_ref().ok_or_else(|| {
        Error::Import(
            "multi-tile benchmarking requires --multi-tile-root pointing at extra Basisvoorziening tiles"
                .to_owned(),
        )
    })?;
    if !multi_root.exists() {
        return Err(Error::Import(format!(
            "multi-tile root {} does not exist",
            multi_root.display()
        )));
    }

    let prepared_root = cli.work_root.join("multi-tile");
    reset_dir(&prepared_root)?;
    fs::create_dir_all(&prepared_root)?;

    let mut copied = Vec::new();
    for entry in WalkBuilder::new(multi_root)
        .hidden(false)
        .follow_links(true)
        .build()
        .filter_map(|entry| entry.ok())
    {
        if !entry.file_type().is_some_and(|ft| ft.is_file()) {
            continue;
        }
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(multi_root)
            .unwrap_or(entry.path());
        let dest = prepared_root.join(rel);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(entry.path(), &dest)?;
        copied.push(dest);
    }
    if copied.is_empty() {
        return Err(Error::Import(format!(
            "multi-tile root {} did not contain any CityJSON tiles",
            multi_root.display()
        )));
    }

    let manifest = multi_tile_manifest("multi-tile", &prepared_root, multi_root, &copied)?;
    write_manifest(&prepared_root.join("benchmark-manifest.json"), &manifest)?;
    Ok(vec![PreparedDataset { manifest }])
}

fn run_dataset(
    dataset: &PreparedDataset,
    worker_count: usize,
) -> Result<Vec<BenchmarkOperationRecord>> {
    let manifest = &dataset.manifest;
    let resolved = resolve_dataset(&manifest.prepared_dataset, None)?;

    let open_started = Instant::now();
    let index = CityIndex::open(resolved.storage_layout(), &resolved.index_path)?;
    let open_elapsed = u64::try_from(open_started.elapsed().as_nanos())
        .map_err(|_| Error::Import("benchmark elapsed time does not fit in u64".to_owned()))?;
    let open_ended = profile::current_memory_snapshot()?;

    let mut index = index;
    let index_started = Instant::now();
    index.reindex()?;
    let index_elapsed = u64::try_from(index_started.elapsed().as_nanos())
        .map_err(|_| Error::Import("benchmark elapsed time does not fit in u64".to_owned()))?;
    let index_ended = profile::current_memory_snapshot()?;

    let feature_count = index.feature_ref_count()?;
    let source_count = index.source_count()?;
    let cityobject_count = index.cityobject_count()?;

    let mut runs = vec![
        build_record(
            manifest,
            worker_count,
            "dataset_open",
            None,
            open_elapsed,
            open_ended,
            feature_count,
            source_count,
            cityobject_count,
            None,
        ),
        build_record(
            manifest,
            worker_count,
            "index_reindex",
            None,
            index_elapsed,
            index_ended,
            feature_count,
            source_count,
            cityobject_count,
            None,
        ),
    ];

    let all_refs = index.feature_ref_page(0, feature_count.min(256))?;
    let sampled_refs = all_refs.into_iter().take(256).collect::<Vec<_>>();

    runs.extend(run_full_scan(
        &index,
        manifest,
        worker_count,
        feature_count,
        source_count,
        cityobject_count,
    )?);
    runs.extend(run_gets(
        &index,
        manifest,
        worker_count,
        feature_count,
        source_count,
        cityobject_count,
    )?);
    runs.extend(run_queries(
        &index,
        manifest,
        worker_count,
        feature_count,
        source_count,
        cityobject_count,
    )?);
    runs.push(run_read_feature(
        &index,
        manifest,
        worker_count,
        feature_count,
        source_count,
        cityobject_count,
        &sampled_refs,
    )?);

    Ok(runs)
}

fn run_full_scan(
    index: &CityIndex,
    manifest: &BenchmarkManifest,
    worker_count: usize,
    feature_count: usize,
    source_count: usize,
    cityobject_count: usize,
) -> Result<Vec<BenchmarkOperationRecord>> {
    let started = Instant::now();
    let mut count = 0usize;
    for page in index.iter_all_feature_ref_pages(512)? {
        count += page?.len();
    }
    let elapsed_ns = u64::try_from(started.elapsed().as_nanos())
        .map_err(|_| Error::Import("benchmark elapsed time does not fit in u64".to_owned()))?;
    let memory = profile::current_memory_snapshot()?;
    Ok(vec![build_record(
        manifest,
        worker_count,
        "full_scan_reference_iteration",
        None,
        elapsed_ns,
        memory,
        feature_count,
        source_count,
        cityobject_count,
        Some(count),
    )])
}

fn run_gets(
    index: &CityIndex,
    manifest: &BenchmarkManifest,
    worker_count: usize,
    feature_count: usize,
    source_count: usize,
    cityobject_count: usize,
) -> Result<Vec<BenchmarkOperationRecord>> {
    let mut runs = Vec::new();
    for feature_id in representative_ids(manifest, feature_count) {
        let started = Instant::now();
        let hit = index.get(&feature_id)?;
        let elapsed_ns = u64::try_from(started.elapsed().as_nanos())
            .map_err(|_| Error::Import("benchmark elapsed time does not fit in u64".to_owned()))?;
        let memory = profile::current_memory_snapshot()?;
        runs.push(build_record(
            manifest,
            worker_count,
            "get",
            Some(feature_id),
            elapsed_ns,
            memory,
            feature_count,
            source_count,
            cityobject_count,
            Some(usize::from(hit.is_some())),
        ));
    }
    Ok(runs)
}

fn run_queries(
    index: &CityIndex,
    manifest: &BenchmarkManifest,
    worker_count: usize,
    feature_count: usize,
    source_count: usize,
    cityobject_count: usize,
) -> Result<Vec<BenchmarkOperationRecord>> {
    let mut runs = Vec::new();
    for window in &manifest.query_windows {
        let started = Instant::now();
        let hits = index.query(&window.bbox)?;
        let elapsed_ns = u64::try_from(started.elapsed().as_nanos())
            .map_err(|_| Error::Import("benchmark elapsed time does not fit in u64".to_owned()))?;
        let memory = profile::current_memory_snapshot()?;
        runs.push(build_record(
            manifest,
            worker_count,
            "bbox_query",
            Some(window.label.clone()),
            elapsed_ns,
            memory,
            feature_count,
            source_count,
            cityobject_count,
            Some(hits.len()),
        ));
    }
    Ok(runs)
}

fn run_read_feature(
    index: &CityIndex,
    manifest: &BenchmarkManifest,
    worker_count: usize,
    feature_count: usize,
    source_count: usize,
    cityobject_count: usize,
    refs: &[crate::IndexedFeatureRef],
) -> Result<BenchmarkOperationRecord> {
    let started = Instant::now();
    let mut reconstructed = 0usize;
    for feature in refs {
        let _model = index.read_feature(feature)?;
        reconstructed += 1;
    }
    let elapsed_ns = u64::try_from(started.elapsed().as_nanos())
        .map_err(|_| Error::Import("benchmark elapsed time does not fit in u64".to_owned()))?;
    let memory = profile::current_memory_snapshot()?;
    Ok(build_record(
        manifest,
        worker_count,
        "read_feature",
        Some(format!("sample-{}", refs.len())),
        elapsed_ns,
        memory,
        feature_count,
        source_count,
        cityobject_count,
        Some(reconstructed),
    ))
}

fn build_record(
    manifest: &BenchmarkManifest,
    worker_count: usize,
    operation: impl Into<String>,
    variant: Option<String>,
    elapsed_ns: u64,
    memory: profile::MemorySnapshot,
    feature_count: usize,
    source_count: usize,
    cityobject_count: usize,
    query_hit_count: Option<usize>,
) -> BenchmarkOperationRecord {
    BenchmarkOperationRecord {
        dataset_label: manifest.dataset_label.clone(),
        source_artifact: manifest.source_artifact.clone(),
        prepared_dataset: manifest.prepared_dataset.clone(),
        subset_size: manifest.subset_size,
        byte_size: manifest.byte_size,
        worker_count,
        operation: operation.into(),
        variant,
        elapsed_ns,
        current_rss_bytes: memory.current_rss_bytes,
        peak_rss_bytes: memory.peak_rss_bytes,
        feature_count,
        source_count,
        cityobject_count,
        query_hit_count,
    }
}

fn representative_ids(manifest: &BenchmarkManifest, feature_count: usize) -> Vec<String> {
    if manifest.representative_feature_ids.is_empty() {
        return Vec::new();
    }
    let mut ids = manifest.representative_feature_ids.clone();
    ids.truncate(ids.len().min(feature_count.max(1)));
    ids
}

fn worker_counts(mut requested: Vec<usize>) -> Vec<usize> {
    if requested.is_empty() {
        requested = vec![
            1,
            std::thread::available_parallelism()
                .map(|count| count.get())
                .unwrap_or(1),
            4,
        ];
    }
    requested.sort_unstable();
    requested.dedup();
    requested
}

fn reset_dir(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}

fn write_manifest(path: &Path, manifest: &BenchmarkManifest) -> Result<()> {
    let file = fs::File::create(path)?;
    serde_json::to_writer_pretty(file, manifest).map_err(|error| Error::Import(error.to_string()))
}

fn manifest_for_cityjson_bytes(
    label: &str,
    source_artifact: &Path,
    prepared_root: &Path,
    subset_size: Option<usize>,
    bytes: &[u8],
) -> Result<BenchmarkManifest> {
    let document: Value =
        serde_json::from_slice(bytes).map_err(|error| Error::Import(error.to_string()))?;
    let feature_ids = extract_root_ids(&document)?;
    let dataset_bbox = bbox_for_cityjson_document(&document)?;
    let query_windows = build_query_windows(dataset_bbox);

    Ok(BenchmarkManifest {
        dataset_label: label.to_owned(),
        source_artifact: source_artifact.to_path_buf(),
        prepared_dataset: prepared_root.to_path_buf(),
        subset_size,
        byte_size: u64::try_from(bytes.len())
            .map_err(|_| Error::Import("prepared dataset size does not fit in u64".to_owned()))?,
        feature_count: feature_ids.len(),
        source_count: 1,
        cityobject_count: count_cityobjects(&document)?,
        dataset_bbox,
        representative_feature_ids: representative_feature_ids(&feature_ids),
        query_windows,
    })
}

fn multi_tile_manifest(
    label: &str,
    prepared_root: &Path,
    source_root: &Path,
    copied: &[PathBuf],
) -> Result<BenchmarkManifest> {
    let mut feature_count = 0usize;
    let mut cityobject_count = 0usize;
    let mut all_ids = Vec::new();
    let mut bbox: Option<BBox> = None;

    for tile in copied {
        let bytes = fs::read(tile)?;
        let document: Value =
            serde_json::from_slice(&bytes).map_err(|error| Error::Import(error.to_string()))?;
        let ids = extract_root_ids(&document)?;
        feature_count += ids.len();
        cityobject_count += count_cityobjects(&document)?;
        all_ids.extend(ids);
        bbox = Some(match bbox {
            None => bbox_for_cityjson_document(&document)?,
            Some(existing) => existing.union(&bbox_for_cityjson_document(&document)?),
        });
    }

    let dataset_bbox = bbox.unwrap_or(BBox {
        min_x: 0.0,
        max_x: 0.0,
        min_y: 0.0,
        max_y: 0.0,
    });

    Ok(BenchmarkManifest {
        dataset_label: label.to_owned(),
        source_artifact: source_root.to_path_buf(),
        prepared_dataset: prepared_root.to_path_buf(),
        subset_size: None,
        byte_size: copied.iter().try_fold(0u64, |sum, path| {
            let len = fs::metadata(path)?.len();
            sum.checked_add(len)
                .ok_or_else(|| Error::Import("prepared dataset size overflowed u64".to_owned()))
        })?,
        feature_count,
        source_count: copied.len(),
        cityobject_count,
        dataset_bbox,
        representative_feature_ids: representative_feature_ids(&all_ids),
        query_windows: build_query_windows(dataset_bbox),
    })
}

fn build_query_windows(bbox: BBox) -> Vec<QueryWindow> {
    vec![
        QueryWindow {
            label: "small".to_owned(),
            bbox: shrink_bbox(bbox, 0.01),
        },
        QueryWindow {
            label: "medium".to_owned(),
            bbox: shrink_bbox(bbox, 0.10),
        },
        QueryWindow {
            label: "large".to_owned(),
            bbox: shrink_bbox(bbox, 0.50),
        },
        QueryWindow {
            label: "full".to_owned(),
            bbox,
        },
    ]
}

fn shrink_bbox(bbox: BBox, fraction: f64) -> BBox {
    let width = (bbox.max_x - bbox.min_x).abs();
    let height = (bbox.max_y - bbox.min_y).abs();
    if width == 0.0 || height == 0.0 {
        return bbox;
    }
    let x_pad = width * (1.0 - fraction) / 2.0;
    let y_pad = height * (1.0 - fraction) / 2.0;
    BBox {
        min_x: bbox.min_x + x_pad,
        max_x: bbox.max_x - x_pad,
        min_y: bbox.min_y + y_pad,
        max_y: bbox.max_y - y_pad,
    }
}

fn representative_feature_ids(feature_ids: &[String]) -> Vec<String> {
    if feature_ids.is_empty() {
        return Vec::new();
    }
    let mut selected = Vec::new();
    selected.push(feature_ids[0].clone());
    if feature_ids.len() > 2 {
        selected.push(feature_ids[feature_ids.len() / 2].clone());
    }
    if feature_ids.len() > 1 {
        selected.push(feature_ids[feature_ids.len() - 1].clone());
    }
    selected.sort();
    selected.dedup();
    selected
}

fn extract_root_ids(document: &Value) -> Result<Vec<String>> {
    let cityobjects = document
        .get("CityObjects")
        .and_then(Value::as_object)
        .ok_or_else(|| Error::Import("CityJSON document is missing CityObjects".to_owned()))?;

    let mut child_ids = BTreeSet::new();
    for object in cityobjects.values() {
        if let Some(children) = object.get("children").and_then(Value::as_array) {
            for child in children {
                if let Some(child_id) = child.as_str() {
                    child_ids.insert(child_id.to_owned());
                }
            }
        }
    }

    let mut ids = cityobjects
        .iter()
        .filter(|(id, object)| {
            object
                .get("parents")
                .and_then(Value::as_array)
                .is_none_or(Vec::is_empty)
                && !child_ids.contains(id.as_str())
        })
        .map(|(id, _)| id.clone())
        .collect::<Vec<_>>();
    ids.sort();
    Ok(ids)
}

fn count_cityobjects(document: &Value) -> Result<usize> {
    let cityobjects = document
        .get("CityObjects")
        .and_then(Value::as_object)
        .ok_or_else(|| Error::Import("CityJSON document is missing CityObjects".to_owned()))?;
    Ok(cityobjects.len())
}

fn bbox_for_cityjson_document(document: &Value) -> Result<BBox> {
    let vertices = document
        .get("vertices")
        .and_then(Value::as_array)
        .ok_or_else(|| Error::Import("CityJSON document is missing vertices".to_owned()))?;
    let transform = document
        .get("transform")
        .and_then(Value::as_object)
        .ok_or_else(|| Error::Import("CityJSON document is missing transform".to_owned()))?;
    let scale = parse_transform_component(transform, "scale")?;
    let translate = parse_transform_component(transform, "translate")?;

    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for vertex in vertices {
        let coords = vertex
            .as_array()
            .ok_or_else(|| Error::Import("vertex must be an array".to_owned()))?;
        if coords.len() != 3 {
            return Err(Error::Import(
                "vertex must have three coordinates".to_owned(),
            ));
        }
        let x = translate[0]
            + scale[0]
                * coords[0].as_f64().ok_or_else(|| {
                    Error::Import("vertex coordinates must be numeric".to_owned())
                })?;
        let y = translate[1]
            + scale[1]
                * coords[1].as_f64().ok_or_else(|| {
                    Error::Import("vertex coordinates must be numeric".to_owned())
                })?;
        min_x = min_x.min(x);
        max_x = max_x.max(x);
        min_y = min_y.min(y);
        max_y = max_y.max(y);
    }

    Ok(BBox {
        min_x,
        max_x,
        min_y,
        max_y,
    })
}

fn parse_transform_component(
    transform: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<[f64; 3]> {
    let values = transform
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| Error::Import(format!("transform is missing {key}")))?;
    if values.len() != 3 {
        return Err(Error::Import(format!(
            "transform {key} must contain three values"
        )));
    }
    Ok([
        values[0]
            .as_f64()
            .ok_or_else(|| Error::Import("transform values must be numeric".to_owned()))?,
        values[1]
            .as_f64()
            .ok_or_else(|| Error::Import("transform values must be numeric".to_owned()))?,
        values[2]
            .as_f64()
            .ok_or_else(|| Error::Import("transform values must be numeric".to_owned()))?,
    ])
}

fn subset_cityjson_document(document: &mut Value, limit: usize) -> Result<Value> {
    let cityobjects = document
        .get("CityObjects")
        .and_then(Value::as_object)
        .ok_or_else(|| Error::Import("CityJSON document is missing CityObjects".to_owned()))?
        .clone();
    let vertices = document
        .get("vertices")
        .and_then(Value::as_array)
        .ok_or_else(|| Error::Import("CityJSON document is missing vertices".to_owned()))?
        .clone();
    let root_ids = extract_root_ids(document)?;
    let selected_roots = root_ids.into_iter().take(limit).collect::<Vec<_>>();
    let mut selected_ids = BTreeSet::new();
    for root_id in &selected_roots {
        collect_cityobject_closure(root_id, &cityobjects, &mut selected_ids)?;
    }

    let mut selected_cityobjects = BTreeMap::new();
    for id in &selected_ids {
        let object = cityobjects
            .get(id)
            .ok_or_else(|| Error::Import(format!("CityObject {id} was not found")))?;
        let mut object = object.clone();
        filter_cityobject_relationships(&mut object, &selected_ids)?;
        selected_cityobjects.insert(id.clone(), object);
    }

    let mut referenced_vertices = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for id in &selected_roots {
        collect_object_vertex_indices(
            &selected_cityobjects,
            id,
            &mut referenced_vertices,
            &mut visited,
        )?;
    }

    let mut remap = HashMap::new();
    let mut local_vertices = Vec::with_capacity(referenced_vertices.len());
    for (new_index, old_index) in referenced_vertices.iter().enumerate() {
        remap.insert(*old_index, new_index);
        let vertex = vertices
            .get(*old_index)
            .ok_or_else(|| Error::Import(format!("vertex index {old_index} is out of bounds")))?;
        local_vertices.push(vertex.clone());
    }

    for object in selected_cityobjects.values_mut() {
        if let Some(geometries) = object
            .as_object_mut()
            .and_then(|object| object.get_mut("geometry"))
            .and_then(Value::as_array_mut)
        {
            for geometry in geometries {
                if let Some(boundaries) = geometry.get_mut("boundaries") {
                    remap_vertex_indices(boundaries, &remap)?;
                }
            }
        }
    }

    let mut root = document.clone();
    let root_object = root
        .as_object_mut()
        .ok_or_else(|| Error::Import("CityJSON document must be an object".to_owned()))?;
    root_object.insert(
        "CityObjects".to_owned(),
        Value::Object(selected_cityobjects.into_iter().collect()),
    );
    root_object.insert("vertices".to_owned(), Value::Array(local_vertices));
    Ok(root)
}

fn collect_cityobject_closure(
    object_id: &str,
    cityobjects: &serde_json::Map<String, Value>,
    selected_ids: &mut BTreeSet<String>,
) -> Result<()> {
    if !selected_ids.insert(object_id.to_owned()) {
        return Ok(());
    }
    let object = cityobjects
        .get(object_id)
        .ok_or_else(|| Error::Import(format!("CityObject {object_id} was not found")))?;
    if let Some(children) = object.get("children").and_then(Value::as_array) {
        for child in children {
            let child_id = child
                .as_str()
                .ok_or_else(|| Error::Import("CityObject children must be strings".to_owned()))?;
            if cityobjects.contains_key(child_id) {
                collect_cityobject_closure(child_id, cityobjects, selected_ids)?;
            }
        }
    }
    Ok(())
}

fn filter_cityobject_relationships(
    object: &mut Value,
    selected_ids: &BTreeSet<String>,
) -> Result<()> {
    let object = object
        .as_object_mut()
        .ok_or_else(|| Error::Import("CityObject must be an object".to_owned()))?;
    for key in ["children", "parents"] {
        let remove_key = match object.get_mut(key) {
            Some(value) => {
                let refs = value
                    .as_array_mut()
                    .ok_or_else(|| Error::Import(format!("{key} must be an array")))?;
                refs.retain(|entry| {
                    entry
                        .as_str()
                        .is_some_and(|object_id| selected_ids.contains(object_id))
                });
                refs.is_empty()
            }
            None => false,
        };
        if remove_key {
            object.remove(key);
        }
    }
    Ok(())
}

fn collect_object_vertex_indices(
    cityobjects: &BTreeMap<String, Value>,
    object_id: &str,
    indices: &mut BTreeSet<usize>,
    visited: &mut BTreeSet<String>,
) -> Result<()> {
    if !visited.insert(object_id.to_owned()) {
        return Ok(());
    }
    let object = cityobjects
        .get(object_id)
        .ok_or_else(|| Error::Import(format!("CityObject {object_id} was not found")))?;
    if let Some(geometries) = object.get("geometry").and_then(Value::as_array) {
        for geometry in geometries {
            if let Some(boundaries) = geometry.get("boundaries") {
                collect_vertex_indices_from_value(boundaries, indices)?;
            }
        }
    }
    if let Some(children) = object.get("children").and_then(Value::as_array) {
        for child in children {
            let child_id = child
                .as_str()
                .ok_or_else(|| Error::Import("CityObject children must be strings".to_owned()))?;
            if cityobjects.contains_key(child_id) {
                collect_object_vertex_indices(cityobjects, child_id, indices, visited)?;
            }
        }
    }
    Ok(())
}

fn collect_vertex_indices_from_value(value: &Value, indices: &mut BTreeSet<usize>) -> Result<()> {
    match value {
        Value::Array(items) => {
            for item in items {
                collect_vertex_indices_from_value(item, indices)?;
            }
            Ok(())
        }
        Value::Number(number) => {
            let index = number.as_u64().ok_or_else(|| {
                Error::Import("vertex indices must be non-negative integers".to_owned())
            })?;
            let index = usize::try_from(index)
                .map_err(|_| Error::Import("vertex index does not fit in usize".to_owned()))?;
            indices.insert(index);
            Ok(())
        }
        Value::Null => Ok(()),
        _ => Err(Error::Import(
            "geometry boundaries must be arrays or non-negative integers".to_owned(),
        )),
    }
}

fn remap_vertex_indices(value: &mut Value, remap: &HashMap<usize, usize>) -> Result<()> {
    match value {
        Value::Array(items) => {
            for item in items {
                remap_vertex_indices(item, remap)?;
            }
            Ok(())
        }
        Value::Number(number) => {
            let old_index = number.as_u64().ok_or_else(|| {
                Error::Import("vertex indices must be non-negative integers".to_owned())
            })?;
            let old_index = usize::try_from(old_index)
                .map_err(|_| Error::Import("vertex index does not fit in usize".to_owned()))?;
            let new_index = remap.get(&old_index).copied().ok_or_else(|| {
                Error::Import(format!("missing remap entry for vertex {old_index}"))
            })?;
            *value = Value::Number(serde_json::Number::from(
                u64::try_from(new_index)
                    .map_err(|_| Error::Import("vertex index does not fit in u64".to_owned()))?,
            ));
            Ok(())
        }
        Value::Null => Ok(()),
        _ => Err(Error::Import(
            "geometry boundaries must be arrays or non-negative integers".to_owned(),
        )),
    }
}

impl BBox {
    fn union(self, other: &BBox) -> BBox {
        BBox {
            min_x: self.min_x.min(other.min_x),
            max_x: self.max_x.max(other.max_x),
            min_y: self.min_y.min(other.min_y),
            max_y: self.max_y.max(other.max_y),
        }
    }
}
