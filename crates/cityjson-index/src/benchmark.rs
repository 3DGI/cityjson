use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use cityjson_lib::{Error, Result};
use clap::{Parser, ValueEnum};
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::profile;
use crate::{BBox, CityIndex, IndexedPackageRef, ResolvedDataset, resolve_dataset};

const DEFAULT_CORPUS_ROOT: &str = "/home/balazs/Development/cityjson-corpus";
const DEFAULT_BASISVOORZIENING_ARTIFACT: &str =
    "artifacts/acquired/basisvoorziening-3d/2022/3d_volledig_84000_450000.city.json";
const DEFAULT_WORK_ROOT: &str = "target/benchmarks/basisvoorziening-3d";
const DEFAULT_GRONINGEN_CORPUS_ROOT: &str = "target/benchmarks/groningen-182/cityjson";
const DEFAULT_SUBSET_SIZES: &[usize] = &[1_000, 5_000, 10_000, 25_000];
const DEFAULT_MULTI_SOURCE_SHARDS: usize = 4;
const DEFAULT_MULTI_SOURCE_FEATURES_PER_SHARD: usize = 1_000;
const DEFAULT_TYLER_TILE_COUNT: usize = 182;
const DEFAULT_BATCH_SIZES: &[usize] = &[1, 16, 256, 4096];
const DEFAULT_CONCURRENT_READERS: &[usize] = &[1, 4];
// Tyler's constants for matching its pipeline exactly
const BENCH_CJINDEX_PARALLEL_CHUNK_SIZE: usize = 2_048;
const BENCHMARK_SCHEMA_VERSION: u32 = 2;
const BENCHMARK_STAGE_EVENT_SCHEMA_VERSION: u32 = 3;
const PROFILE_CHECKPOINT_SETTLE_TIME: Duration = Duration::from_millis(300);

// Thread-local storage for CityIndex caching (matching Tyler's CJINDEX_THREAD_LOCAL pattern)
thread_local! {
    static BENCH_INDEX_THREAD_LOCAL: RefCell<Option<(PathBuf, CityIndex)>> =
        const { RefCell::new(None) };
}

#[derive(Debug, Clone, Parser)]
#[command(
    name = "bench-index",
    about = "Run JSON-emitting CityJSON indexing benchmarks",
    long_about = r#"Run JSON-emitting CityJSON indexing benchmarks.

The benchmark runner focuses on large multi-file corpus performance measurement. The primary benchmark uses the Groningen corpus (182 tiles) to simulate production workloads matching tyler's usage patterns. Each worker-count measurement uses a fresh SQLite index path. Prefer repeated benchmark invocations over a single pass when comparing timings. RSS fields report Linux /proc/self/status snapshots: current_rss_bytes is VmRSS, process_peak_rss_bytes is process-lifetime VmHWM, and peak_rss_bytes is a deprecated compatibility alias for that same process-lifetime peak.

Note: When warmth is specified without positions, all positions are tested on the SAME warmed index. This is intentional - it measures the effect of warmup across different source positions. In production (tyler), the application typically reuses the same index for multiple queries.
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

    /// Include a prepared storage layout. Defaults to all supported layouts.
    #[arg(long, value_enum)]
    pub layout: Vec<BenchmarkLayoutKind>,

    /// Worker counts to record for each dataset.
    #[arg(long, value_name = "WORKERS")]
    pub workers: Vec<usize>,

    /// Optional root directory containing additional Basisvoorziening tiles.
    #[arg(long)]
    pub multi_tile_root: Option<PathBuf>,

    /// Override Groningen corpus path for Tyler pipeline benchmark.
    #[arg(long)]
    pub groningen_corpus: Option<PathBuf>,

    /// Number of Groningen tiles to use for Tyler pipeline benchmark.
    #[arg(long, default_value_t = DEFAULT_TYLER_TILE_COUNT)]
    pub tyler_tile_count: usize,

    /// Benchmark warmth (cold = fresh index for each operation, warm = reuse existing index).
    #[arg(long, value_enum)]
    pub warmth: Vec<BenchmarkWarmth>,

    /// Source position for scalar reconstruction benchmarks.
    #[arg(long, value_enum)]
    pub source_position: Vec<SourcePosition>,

    /// Batch sizes for same-source batch reconstruction benchmarks.
    #[arg(long, value_name = "SIZES")]
    pub batch_size: Vec<usize>,

    /// Number of concurrent readers for concurrency benchmarks.
    #[arg(long, value_name = "COUNT")]
    pub concurrent_readers: Vec<usize>,

    /// Prepare datasets and sidecars without running measured operations.
    #[arg(long)]
    pub prepare_only: bool,

    /// Run one isolated Tyler profiling target.
    #[arg(long, value_enum)]
    pub profile_target: Option<BenchmarkProfileTarget>,

    /// Reuse the prepared Tyler manifest instead of recreating the dataset.
    #[arg(long, requires = "profile_target")]
    pub reuse_prepared: bool,

    /// Append incremental stage lifecycle events as JSON Lines.
    #[arg(long, requires = "profile_target")]
    pub profile_events: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum BenchmarkProfileTarget {
    TylerPipeline,
    TylerFeatureMaterialization,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum BenchmarkCaseKind {
    SingleTileFull,
    SingleTileSubsets,
    MultiSource,
    MultiTile,
    TylerPipeline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum BenchmarkLayoutKind {
    CityJson,
    CityJsonSeq,
    FeatureFiles,
}

impl BenchmarkLayoutKind {
    #[allow(dead_code)]
    const ALL: [Self; 3] = [Self::CityJson, Self::CityJsonSeq, Self::FeatureFiles];

    fn as_label(self) -> &'static str {
        match self {
            Self::CityJson => "cityjson",
            Self::CityJsonSeq => "cityjson-seq",
            Self::FeatureFiles => "feature-files",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum BenchmarkWarmth {
    Cold,
    Warm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum)]
pub enum SourcePosition {
    First,
    Middle,
    Last,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkReport {
    pub schema_version: u32,
    pub runs: Vec<BenchmarkOperationRecord>,
}

#[derive(Debug, Serialize)]
struct BenchmarkStageEvent<'a> {
    schema_version: u32,
    timestamp_ns: u64,
    event: &'a str,
    stage: &'a str,
    worker_count: usize,
    elapsed_ns: Option<u64>,
    observed_worker_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
struct WorkerVertexCacheStats {
    worker_index: usize,
    cached_source_count: usize,
    cached_vertex_count: usize,
    vertex_capacity_bytes: u64,
}

#[derive(Debug, Serialize)]
struct BenchmarkCacheCheckpointEvent<'a> {
    schema_version: u32,
    timestamp_ns: u64,
    event: &'a str,
    stage: &'a str,
    worker_count: usize,
    current_rss_bytes: u64,
    process_peak_rss_bytes: u64,
    cached_source_count: usize,
    cached_vertex_count: usize,
    vertex_capacity_bytes: u64,
    workers: &'a [WorkerVertexCacheStats],
}

#[derive(Debug, Clone)]
struct BenchmarkRecordInput {
    dataset_label: String,
    source_artifact: PathBuf,
    prepared_dataset: PathBuf,
    subset_size: Option<usize>,
    layout: BenchmarkLayoutKind,
    byte_size: u64,
    sidecar_byte_size: u64,
    worker_count: usize,
    operation: String,
    variant: Option<String>,
    elapsed_ns: u64,
    memory: profile::MemorySnapshot,
    feature_count: usize,
    package_count: usize,
    source_count: usize,
    cityobject_count: usize,
    cityobject_relationship_count: usize,
    multi_geometry_cityobject_count: usize,
    query_hit_count: Option<usize>,
    operation_local_peak_rss_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkOperationRecord {
    pub dataset_label: String,
    pub source_artifact: PathBuf,
    pub prepared_dataset: PathBuf,
    pub subset_size: Option<usize>,
    pub layout: BenchmarkLayoutKind,
    pub byte_size: u64,
    pub sidecar_byte_size: u64,
    pub worker_count: usize,
    pub operation: String,
    pub variant: Option<String>,
    pub elapsed_ns: u64,
    pub current_rss_bytes: u64,
    pub process_peak_rss_bytes: u64,
    /// Deprecated compatibility field. This is a process-lifetime peak RSS
    /// alias, not an operation-local peak.
    pub peak_rss_bytes: u64,
    pub feature_count: usize,
    pub package_count: usize,
    pub source_count: usize,
    pub cityobject_count: usize,
    pub cityobject_relationship_count: usize,
    pub multi_geometry_cityobject_count: usize,
    pub query_hit_count: Option<usize>,
    /// Operation-local peak RSS bytes, measured as the increment above pre-operation baseline.
    pub operation_local_peak_rss_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BenchmarkManifest {
    dataset_label: String,
    source_artifact: PathBuf,
    prepared_dataset: PathBuf,
    subset_size: Option<usize>,
    layout: BenchmarkLayoutKind,
    byte_size: u64,
    feature_count: usize,
    source_count: usize,
    cityobject_count: usize,
    cityobject_relationship_count: usize,
    multi_geometry_cityobject_count: usize,
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

/// Executes the benchmark suite and returns the collected records.
///
/// # Errors
///
/// Returns an error if the pinned artifact is missing, dataset preparation
/// fails, or any benchmarked operation fails.
pub fn run(cli: &BenchmarkCli) -> Result<BenchmarkReport> {
    let artifact = cli
        .artifact
        .clone()
        .unwrap_or_else(|| cli.corpus_root.join(DEFAULT_BASISVOORZIENING_ARTIFACT));

    let cases = if cli.case.is_empty() {
        vec![BenchmarkCaseKind::TylerPipeline]
    } else {
        cli.case.clone()
    };
    if cases
        .iter()
        .any(|case| !matches!(case, BenchmarkCaseKind::TylerPipeline))
        && !artifact.exists()
    {
        return Err(Error::Import(format!(
            "missing pinned Basisvoorziening 3D artifact {}; run `cd /home/balazs/Development/cityjson-corpus && just acquire-basisvoorziening-3d`",
            artifact.display()
        )));
    }

    if cli.profile_target.is_some() {
        return run_profile_target(cli);
    }

    let worker_counts = worker_counts(cli.workers.clone());
    let layouts = benchmark_layouts(&cli.layout);

    let mut runs = Vec::new();
    for case in cases {
        for layout in &layouts {
            for dataset in prepare_case(cli, case, *layout, &artifact)? {
                for worker_count in &worker_counts {
                    if cli.prepare_only {
                        prepare_benchmark_sidecar(&dataset, *worker_count)?;
                        continue;
                    }
                    runs.extend(with_worker_count_env(*worker_count, || {
                        run_dataset(
                            &dataset,
                            &cli.warmth,
                            &cli.source_position,
                            &cli.batch_size,
                            &cli.concurrent_readers,
                        )
                    })?);
                }
            }
        }
    }

    Ok(BenchmarkReport {
        schema_version: BENCHMARK_SCHEMA_VERSION,
        runs,
    })
}

fn run_profile_target(cli: &BenchmarkCli) -> Result<BenchmarkReport> {
    if cli.prepare_only {
        return Err(Error::Import(
            "--prepare-only and --profile-target are mutually exclusive".to_owned(),
        ));
    }
    if cli.workers.len() != 1 {
        return Err(Error::Import(
            "--profile-target requires exactly one --workers value".to_owned(),
        ));
    }
    if !cli.reuse_prepared {
        return Err(Error::Import(
            "--profile-target requires --reuse-prepared so dataset and sidecar preparation stay outside the profiled process"
                .to_owned(),
        ));
    }
    if !cli.case.is_empty()
        && !cli
            .case
            .iter()
            .all(|case| matches!(case, BenchmarkCaseKind::TylerPipeline))
    {
        return Err(Error::Import(
            "--profile-target only supports --case tyler-pipeline".to_owned(),
        ));
    }

    let layouts = if cli.layout.is_empty() {
        vec![BenchmarkLayoutKind::CityJson]
    } else {
        benchmark_layouts(&cli.layout)
    };
    if layouts.len() != 1 {
        return Err(Error::Import(
            "--profile-target requires exactly one --layout value".to_owned(),
        ));
    }
    let layout = layouts[0];
    let dataset = load_prepared_tyler_dataset(cli, layout)?;
    let worker_count = cli.workers[0];
    let index_path = benchmark_index_path(&dataset.manifest, worker_count);
    if !index_path.exists() {
        return Err(Error::Import(format!(
            "prepared Tyler sidecar {} does not exist; run --prepare-only for worker {worker_count} before profiling",
            index_path.display()
        )));
    }

    let runs = with_worker_count_env(worker_count, || {
        run_isolated_tyler_target(
            &dataset,
            cli.profile_target.expect("profile target was checked"),
            worker_count,
            cli.profile_events.as_deref(),
        )
    })?;
    Ok(BenchmarkReport {
        schema_version: BENCHMARK_SCHEMA_VERSION,
        runs,
    })
}

fn load_prepared_tyler_dataset(
    cli: &BenchmarkCli,
    layout: BenchmarkLayoutKind,
) -> Result<PreparedDataset> {
    let path = cli
        .work_root
        .join(layout.as_label())
        .join("tyler-pipeline")
        .join("benchmark-manifest.json");
    let bytes = fs::read(&path).map_err(|error| {
        Error::Import(format!(
            "failed to read prepared Tyler manifest {}: {error}",
            path.display()
        ))
    })?;
    let manifest: BenchmarkManifest = serde_json::from_slice(&bytes).map_err(|error| {
        Error::Import(format!(
            "failed to parse prepared Tyler manifest {}: {error}",
            path.display()
        ))
    })?;
    if manifest.layout != layout {
        return Err(Error::Import(format!(
            "prepared Tyler manifest {} records layout {}, expected {}",
            path.display(),
            manifest.layout.as_label(),
            layout.as_label()
        )));
    }
    if manifest.source_count != cli.tyler_tile_count {
        return Err(Error::Import(format!(
            "prepared Tyler manifest {} contains {} sources, expected {} tiles",
            path.display(),
            manifest.source_count,
            cli.tyler_tile_count
        )));
    }
    let requested_corpus = cli.groningen_corpus.clone().unwrap_or_else(|| {
        std::env::var("CITYJSON_GRONINGEN_CORPUS").map_or_else(
            |_| PathBuf::from(DEFAULT_GRONINGEN_CORPUS_ROOT),
            PathBuf::from,
        )
    });
    let prepared_corpus = fs::canonicalize(&manifest.source_artifact).map_err(|error| {
        Error::Import(format!(
            "failed to resolve prepared Groningen corpus {}: {error}",
            manifest.source_artifact.display()
        ))
    })?;
    let requested_corpus = fs::canonicalize(&requested_corpus).map_err(|error| {
        Error::Import(format!(
            "failed to resolve requested Groningen corpus {}: {error}",
            requested_corpus.display()
        ))
    })?;
    if prepared_corpus != requested_corpus {
        return Err(Error::Import(format!(
            "prepared Tyler manifest uses Groningen corpus {}, expected {}",
            prepared_corpus.display(),
            requested_corpus.display()
        )));
    }
    Ok(PreparedDataset { manifest })
}

fn prepare_benchmark_sidecar(dataset: &PreparedDataset, worker_count: usize) -> Result<()> {
    with_worker_count_env(worker_count, || {
        let index_path = fresh_benchmark_index_path(&dataset.manifest, worker_count)?;
        let resolved = resolve_dataset(&dataset.manifest.prepared_dataset, Some(index_path))?;
        let mut index = CityIndex::open(resolved.storage_layout(), &resolved.index_path)?;
        index.reindex()
    })
}

/// Writes the benchmark report to stdout in either JSON or compact text form.
///
/// # Errors
///
/// Returns an error if writing to stdout fails or JSON serialization fails.
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
            "{} worker={} op={} variant={} elapsed_ns={} current_rss_bytes={} process_peak_rss_bytes={} hits={}",
            run.dataset_label,
            run.worker_count,
            run.operation,
            run.variant.as_deref().unwrap_or("-"),
            run.elapsed_ns,
            run.current_rss_bytes,
            run.process_peak_rss_bytes,
            run.query_hit_count
                .map_or_else(|| "-".to_owned(), |count| count.to_string())
        )?;
    }
    handle.flush()?;
    Ok(())
}

fn prepare_case(
    cli: &BenchmarkCli,
    case: BenchmarkCaseKind,
    layout: BenchmarkLayoutKind,
    artifact: &Path,
) -> Result<Vec<PreparedDataset>> {
    match case {
        BenchmarkCaseKind::SingleTileFull => Ok(vec![prepare_single_tile_dataset(
            cli,
            "single-tile-full",
            layout,
            artifact,
            None,
        )?]),
        BenchmarkCaseKind::SingleTileSubsets => DEFAULT_SUBSET_SIZES
            .iter()
            .map(|subset_size| {
                prepare_single_tile_dataset(
                    cli,
                    &format!("single-tile-subset-{subset_size}"),
                    layout,
                    artifact,
                    Some(*subset_size),
                )
            })
            .collect(),
        BenchmarkCaseKind::MultiSource => {
            Ok(vec![prepare_multi_source_dataset(cli, layout, artifact)?])
        }
        BenchmarkCaseKind::MultiTile => prepare_multi_tile_dataset(cli, layout),
        BenchmarkCaseKind::TylerPipeline => {
            let groningen_root = cli.groningen_corpus.clone().unwrap_or_else(|| {
                std::env::var("CITYJSON_GRONINGEN_CORPUS").map_or_else(
                    |_| PathBuf::from(DEFAULT_GRONINGEN_CORPUS_ROOT),
                    PathBuf::from,
                )
            });
            Ok(vec![prepare_tyler_dataset(
                cli,
                layout,
                &groningen_root,
                cli.tyler_tile_count,
            )?])
        }
    }
}

fn prepare_single_tile_dataset(
    cli: &BenchmarkCli,
    label: &str,
    layout: BenchmarkLayoutKind,
    artifact: &Path,
    subset_size: Option<usize>,
) -> Result<PreparedDataset> {
    let prepared_root = cli.work_root.join(layout.as_label()).join(label);
    reset_dir(&prepared_root)?;
    fs::create_dir_all(&prepared_root)?;

    let bytes = fs::read(artifact)?;
    let mut document: Value =
        serde_json::from_slice(&bytes).map_err(|error| Error::Import(error.to_string()))?;
    let original_bytes = if subset_size.is_none() {
        Some(bytes.clone())
    } else {
        None
    };
    if let Some(limit) = subset_size {
        document = subset_cityjson_document(&mut document, limit)?;
    }
    let feature_count = extract_root_ids(&document)?.len();
    let byte_size = u64::try_from(bytes.len())
        .map_err(|_| Error::Import("prepared dataset size does not fit in u64".to_owned()))?;
    let source = CityJsonSourceDocument {
        file_stem: "dataset".to_owned(),
        document,
        original_bytes,
    };
    let manifest = materialize_layout_dataset(
        label,
        layout,
        artifact,
        &prepared_root,
        subset_size.map(|_| feature_count),
        &[source],
        byte_size,
    )?;
    write_manifest(&prepared_root.join("benchmark-manifest.json"), &manifest)?;
    Ok(PreparedDataset { manifest })
}

fn prepare_multi_source_dataset(
    cli: &BenchmarkCli,
    layout: BenchmarkLayoutKind,
    artifact: &Path,
) -> Result<PreparedDataset> {
    let prepared_root = cli.work_root.join(layout.as_label()).join("multi-source");
    reset_dir(&prepared_root)?;
    fs::create_dir_all(&prepared_root)?;

    let bytes = fs::read(artifact)?;
    let document: Value =
        serde_json::from_slice(&bytes).map_err(|error| Error::Import(error.to_string()))?;
    let root_ids = extract_root_ids(&document)?;
    if root_ids.len() < 2 {
        return Err(Error::Import(
            "multi-source benchmark preparation requires at least two root CityObjects".to_owned(),
        ));
    }

    let shard_count = DEFAULT_MULTI_SOURCE_SHARDS.min(root_ids.len());
    let total_feature_count =
        (DEFAULT_MULTI_SOURCE_FEATURES_PER_SHARD * shard_count).min(root_ids.len());
    let selected_root_ids = root_ids
        .into_iter()
        .take(total_feature_count)
        .collect::<Vec<_>>();
    let mut sources = Vec::with_capacity(shard_count);
    for shard_index in 0..shard_count {
        let start = shard_index * total_feature_count / shard_count;
        let end = (shard_index + 1) * total_feature_count / shard_count;
        let subset = subset_cityjson_document_by_roots(&document, &selected_root_ids[start..end])?;
        sources.push(CityJsonSourceDocument {
            file_stem: format!("source-{shard_index:02}"),
            document: subset,
            original_bytes: None,
        });
    }

    let byte_size = u64::try_from(bytes.len())
        .map_err(|_| Error::Import("prepared dataset size does not fit in u64".to_owned()))?;
    let manifest = materialize_layout_dataset(
        "multi-source",
        layout,
        artifact,
        &prepared_root,
        None,
        &sources,
        byte_size,
    )?;
    write_manifest(&prepared_root.join("benchmark-manifest.json"), &manifest)?;
    Ok(PreparedDataset { manifest })
}

fn prepare_multi_tile_dataset(
    cli: &BenchmarkCli,
    layout: BenchmarkLayoutKind,
) -> Result<Vec<PreparedDataset>> {
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

    let prepared_root = cli.work_root.join(layout.as_label()).join("multi-tile");
    reset_dir(&prepared_root)?;
    fs::create_dir_all(&prepared_root)?;

    let mut sources = Vec::new();
    let mut byte_size = 0u64;
    for entry in WalkBuilder::new(multi_root)
        .hidden(false)
        .follow_links(true)
        .build()
        .filter_map(std::result::Result::ok)
    {
        if !entry.file_type().is_some_and(|ft| ft.is_file()) {
            continue;
        }
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let bytes = fs::read(entry.path())?;
        let document: Value =
            serde_json::from_slice(&bytes).map_err(|error| Error::Import(error.to_string()))?;
        byte_size = byte_size
            .checked_add(u64::try_from(bytes.len()).map_err(|_| {
                Error::Import("prepared dataset size does not fit in u64".to_owned())
            })?)
            .ok_or_else(|| Error::Import("prepared dataset size overflowed u64".to_owned()))?;
        let stem = entry
            .path()
            .strip_prefix(multi_root)
            .unwrap_or(entry.path())
            .with_extension("")
            .to_string_lossy()
            .replace(std::path::MAIN_SEPARATOR, "-");
        sources.push(CityJsonSourceDocument {
            file_stem: stem,
            document,
            original_bytes: Some(bytes),
        });
    }
    if sources.is_empty() {
        return Err(Error::Import(format!(
            "multi-tile root {} did not contain any CityJSON tiles",
            multi_root.display()
        )));
    }

    let manifest = materialize_layout_dataset(
        "multi-tile",
        layout,
        multi_root,
        &prepared_root,
        None,
        &sources,
        byte_size,
    )?;
    write_manifest(&prepared_root.join("benchmark-manifest.json"), &manifest)?;
    Ok(vec![PreparedDataset { manifest }])
}

#[allow(clippy::too_many_lines)]
#[allow(clippy::implicit_clone)]
fn prepare_tyler_dataset(
    cli: &BenchmarkCli,
    layout: BenchmarkLayoutKind,
    groningen_root: &Path,
    tile_count: usize,
) -> Result<PreparedDataset> {
    let prepared_root = cli.work_root.join(layout.as_label()).join("tyler-pipeline");
    if prepared_root.join("benchmark-manifest.json").exists()
        && let Ok(dataset) = load_prepared_tyler_dataset(cli, layout)
    {
        return Ok(dataset);
    }
    reset_dir(&prepared_root)?;
    fs::create_dir_all(&prepared_root)?;

    // Validate that Groningen corpus exists
    if !groningen_root.exists() {
        return Err(Error::Import(format!(
            "Groningen corpus root {} does not exist. Run tools/download-groningen-corpus.sh first or set CITYJSON_GRONINGEN_CORPUS",
            groningen_root.display()
        )));
    }

    // Collect all CityJSON files from the Groningen corpus
    let mut cityjson_files: Vec<PathBuf> = Vec::new();
    for entry in WalkBuilder::new(groningen_root)
        .hidden(false)
        .follow_links(true)
        .build()
        .filter_map(std::result::Result::ok)
    {
        if entry.file_type().is_some_and(|ft| ft.is_file())
            && entry.path().extension().is_some_and(|ext| ext == "json")
            && entry
                .path()
                .file_name()
                .is_some_and(|name| name.to_string_lossy().ends_with(".city.json"))
        {
            cityjson_files.push(entry.path().to_path_buf());
        }
    }

    if cityjson_files.is_empty() {
        return Err(Error::Import(format!(
            "no CityJSON files found in Groningen corpus at {}",
            groningen_root.display()
        )));
    }

    // Sort files for deterministic ordering
    cityjson_files.sort();

    // Limit to requested tile count
    let max_files = tile_count.min(cityjson_files.len());
    let selected_files = cityjson_files
        .into_iter()
        .take(max_files)
        .collect::<Vec<_>>();

    // For Tyler pipeline, copy files to prepared directory and extract statistics
    // For CityJson layout, copy files directly without transformation
    // For other layouts, transform and write to prepared directory
    let mut feature_count = 0usize;
    let mut cityobject_count = 0usize;
    let mut relationship_count = 0usize;
    let mut multi_geometry_count = 0usize;
    let mut all_ids = Vec::new();
    let mut bbox: Option<BBox> = None;
    let mut byte_size = 0u64;

    if layout == BenchmarkLayoutKind::CityJson {
        // For CityJson layout: copy files directly and extract stats
        for file_path in &selected_files {
            let bytes = fs::read(file_path)?;
            let file_size = u64::try_from(bytes.len())
                .map_err(|_| Error::Import("file size does not fit in u64".to_owned()))?;
            byte_size = byte_size
                .checked_add(file_size)
                .ok_or_else(|| Error::Import("total byte size overflowed u64".to_owned()))?;

            // Copy file to prepared directory
            let dest_path = prepared_root.join(file_path.file_name().unwrap());
            fs::write(&dest_path, &bytes)?;

            // Parse and extract statistics
            let document: Value =
                serde_json::from_slice(&bytes).map_err(|error| Error::Import(error.to_string()))?;

            let ids = extract_root_ids(&document)?;
            feature_count += ids.len();
            cityobject_count += count_cityobjects(&document)?;
            relationship_count += count_cityobject_relationships(&document)?;
            multi_geometry_count += count_multi_geometry_cityobjects(&document)?;
            all_ids.extend(ids);
            bbox = Some(match bbox {
                None => bbox_for_cityjson_document(&document)?,
                Some(existing) => existing.union(&bbox_for_cityjson_document(&document)?),
            });
        }
    } else {
        // For CityJsonSeq and FeatureFiles: process files one at a time to avoid OOM
        for (index, file_path) in selected_files.iter().enumerate() {
            let bytes = fs::read(file_path)?;
            let file_size = u64::try_from(bytes.len())
                .map_err(|_| Error::Import("file size does not fit in u64".to_owned()))?;
            byte_size = byte_size
                .checked_add(file_size)
                .ok_or_else(|| Error::Import("total byte size overflowed u64".to_owned()))?;

            let document: Value =
                serde_json::from_slice(&bytes).map_err(|error| Error::Import(error.to_string()))?;

            let stem = file_path
                .file_stem()
                .unwrap_or_else(|| file_path.as_os_str());
            let stem_str = stem.to_string_lossy();
            let file_stem = format!("tile-{index:03}-{stem_str}");

            // Write transformed output immediately to avoid keeping all in memory
            match layout {
                BenchmarkLayoutKind::CityJsonSeq => {
                    let path = prepared_root.join(format!("{file_stem}.city.jsonl"));
                    if let Some(parent) = path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    let mut file = fs::File::create(&path)?;
                    write_json_line(&mut file, &cityjson_base_document(&document)?)?;
                    for feature in feature_documents_for_roots(&document)? {
                        write_json_line(&mut file, &feature)?;
                    }
                }
                BenchmarkLayoutKind::FeatureFiles => {
                    let source_root = if selected_files.len() == 1 {
                        prepared_root.to_path_buf()
                    } else {
                        prepared_root.join(&file_stem)
                    };
                    fs::create_dir_all(source_root.join("features"))?;
                    let metadata_path = source_root.join("metadata.json");
                    let metadata = cityjson_base_document(&document)?;
                    fs::write(
                        metadata_path,
                        serde_json::to_vec(&metadata)
                            .map_err(|error| Error::Import(error.to_string()))?,
                    )?;
                    for feature in feature_documents_for_roots(&document)? {
                        let feature_id =
                            feature.get("id").and_then(Value::as_str).ok_or_else(|| {
                                Error::Import("CityJSONFeature is missing id".to_owned())
                            })?;
                        let path = source_root
                            .join("features")
                            .join(format!("{}.city.jsonl", safe_file_stem(feature_id)));
                        write_json_line(&mut fs::File::create(path)?, &feature)?;
                    }
                }
                BenchmarkLayoutKind::CityJson => unreachable!(),
            }

            // Extract statistics while document is still in memory
            let ids = extract_root_ids(&document)?;
            feature_count += ids.len();
            cityobject_count += count_cityobjects(&document)?;
            relationship_count += count_cityobject_relationships(&document)?;
            multi_geometry_count += count_multi_geometry_cityobjects(&document)?;
            all_ids.extend(ids);
            bbox = Some(match bbox {
                None => bbox_for_cityjson_document(&document)?,
                Some(existing) => existing.union(&bbox_for_cityjson_document(&document)?),
            });

            // document is dropped here, freeing memory before next iteration
        }
    }

    let dataset_bbox = bbox.unwrap_or(BBox {
        min_x: 0.0,
        max_x: 0.0,
        min_y: 0.0,
        max_y: 0.0,
    });

    let manifest = BenchmarkManifest {
        dataset_label: format!("tyler-pipeline-{}", layout.as_label()),
        source_artifact: groningen_root.to_path_buf(),
        prepared_dataset: prepared_root.to_path_buf(),
        subset_size: None,
        layout,
        byte_size,
        feature_count,
        source_count: selected_files.len(),
        cityobject_count,
        cityobject_relationship_count: relationship_count,
        multi_geometry_cityobject_count: multi_geometry_count,
        dataset_bbox,
        representative_feature_ids: representative_feature_ids(&all_ids),
        query_windows: build_query_windows(dataset_bbox),
    };
    write_manifest(&prepared_root.join("benchmark-manifest.json"), &manifest)?;
    Ok(PreparedDataset { manifest })
}

#[derive(Debug, Clone)]
struct CityJsonSourceDocument {
    file_stem: String,
    document: Value,
    original_bytes: Option<Vec<u8>>,
}

fn benchmark_layouts(requested: &[BenchmarkLayoutKind]) -> Vec<BenchmarkLayoutKind> {
    if requested.is_empty() {
        // Exclude FeatureFiles from defaults to avoid file descriptor exhaustion
        // with large datasets (e.g., Basisvoorziening with 49k+ features).
        // Users can explicitly request FeatureFiles with --layout feature-files.
        return vec![
            BenchmarkLayoutKind::CityJson,
            BenchmarkLayoutKind::CityJsonSeq,
        ];
    }
    let mut layouts = requested.to_vec();
    layouts.sort_by_key(|layout| match layout {
        BenchmarkLayoutKind::CityJson => 0,
        BenchmarkLayoutKind::CityJsonSeq => 1,
        BenchmarkLayoutKind::FeatureFiles => 2,
    });
    layouts.dedup();
    layouts
}

fn materialize_layout_dataset(
    label: &str,
    layout: BenchmarkLayoutKind,
    source_artifact: &Path,
    prepared_root: &Path,
    subset_size: Option<usize>,
    sources: &[CityJsonSourceDocument],
    _source_byte_size: u64,
) -> Result<BenchmarkManifest> {
    match layout {
        BenchmarkLayoutKind::CityJson => materialize_cityjson_dataset(sources, prepared_root)?,
        BenchmarkLayoutKind::CityJsonSeq => {
            materialize_cityjson_seq_dataset(sources, prepared_root)?;
        }
        BenchmarkLayoutKind::FeatureFiles => {
            materialize_feature_files_dataset(sources, prepared_root)?;
        }
    }

    let mut feature_count = 0usize;
    let mut cityobject_count = 0usize;
    let mut relationship_count = 0usize;
    let mut multi_geometry_count = 0usize;
    let mut all_ids = Vec::new();
    let mut bbox: Option<BBox> = None;

    for source in sources {
        let ids = extract_root_ids(&source.document)?;
        feature_count += ids.len();
        cityobject_count += count_cityobjects(&source.document)?;
        relationship_count += count_cityobject_relationships(&source.document)?;
        multi_geometry_count += count_multi_geometry_cityobjects(&source.document)?;
        all_ids.extend(ids);
        bbox = Some(match bbox {
            None => bbox_for_cityjson_document(&source.document)?,
            Some(existing) => existing.union(&bbox_for_cityjson_document(&source.document)?),
        });
    }

    let dataset_bbox = bbox.unwrap_or(BBox {
        min_x: 0.0,
        max_x: 0.0,
        min_y: 0.0,
        max_y: 0.0,
    });

    Ok(BenchmarkManifest {
        dataset_label: format!("{label}-{}", layout.as_label()),
        source_artifact: source_artifact.to_path_buf(),
        prepared_dataset: prepared_root.to_path_buf(),
        subset_size,
        layout,
        byte_size: total_file_size(prepared_root)?,
        feature_count,
        source_count: sources.len(),
        cityobject_count,
        cityobject_relationship_count: relationship_count,
        multi_geometry_cityobject_count: multi_geometry_count,
        dataset_bbox,
        representative_feature_ids: representative_feature_ids(&all_ids),
        query_windows: build_query_windows(dataset_bbox),
    })
}

fn materialize_cityjson_dataset(
    sources: &[CityJsonSourceDocument],
    prepared_root: &Path,
) -> Result<()> {
    for source in sources {
        let path = prepared_root.join(format!("{}.city.json", source.file_stem));
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        if let Some(bytes) = &source.original_bytes {
            fs::write(path, bytes)?;
        } else {
            let bytes = serde_json::to_vec(&source.document)
                .map_err(|error| Error::Import(error.to_string()))?;
            fs::write(path, bytes)?;
        }
    }
    Ok(())
}

fn materialize_cityjson_seq_dataset(
    sources: &[CityJsonSourceDocument],
    prepared_root: &Path,
) -> Result<()> {
    for source in sources {
        let path = prepared_root.join(format!("{}.city.jsonl", source.file_stem));
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = fs::File::create(path)?;
        write_json_line(&mut file, &cityjson_base_document(&source.document)?)?;
        for feature in feature_documents_for_roots(&source.document)? {
            write_json_line(&mut file, &feature)?;
        }
    }
    Ok(())
}

fn materialize_feature_files_dataset(
    sources: &[CityJsonSourceDocument],
    prepared_root: &Path,
) -> Result<()> {
    for source in sources {
        let source_root = if sources.len() == 1 {
            prepared_root.to_path_buf()
        } else {
            prepared_root.join(&source.file_stem)
        };
        fs::create_dir_all(source_root.join("features"))?;
        let metadata_path = source_root.join("metadata.json");
        let metadata = cityjson_base_document(&source.document)?;
        fs::write(
            metadata_path,
            serde_json::to_vec(&metadata).map_err(|error| Error::Import(error.to_string()))?,
        )?;
        for feature in feature_documents_for_roots(&source.document)? {
            let feature_id = feature
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| Error::Import("CityJSONFeature is missing id".to_owned()))?;
            let path = source_root
                .join("features")
                .join(format!("{}.city.jsonl", safe_file_stem(feature_id)));
            write_json_line(&mut fs::File::create(path)?, &feature)?;
        }
    }
    Ok(())
}

fn write_json_line(file: &mut fs::File, value: &Value) -> Result<()> {
    serde_json::to_writer(&mut *file, value).map_err(|error| Error::Import(error.to_string()))?;
    file.write_all(b"\n")?;
    Ok(())
}

fn cityjson_base_document(document: &Value) -> Result<Value> {
    let mut metadata = document.clone();
    let root = metadata
        .as_object_mut()
        .ok_or_else(|| Error::Import("CityJSON document must be an object".to_owned()))?;
    root.insert(
        "CityObjects".to_owned(),
        Value::Object(serde_json::Map::new()),
    );
    root.insert("vertices".to_owned(), Value::Array(Vec::new()));
    Ok(metadata)
}

fn feature_documents_for_roots(document: &Value) -> Result<Vec<Value>> {
    extract_root_ids(document)?
        .into_iter()
        .map(|root_id| cityjson_feature_for_root(document, &root_id))
        .collect()
}

fn cityjson_feature_for_root(document: &Value, root_id: &str) -> Result<Value> {
    let cityobjects = document
        .get("CityObjects")
        .and_then(Value::as_object)
        .ok_or_else(|| Error::Import("CityJSON document is missing CityObjects".to_owned()))?;
    let vertices = document
        .get("vertices")
        .and_then(Value::as_array)
        .ok_or_else(|| Error::Import("CityJSON document is missing vertices".to_owned()))?;

    let mut selected_ids = BTreeSet::new();
    collect_cityobject_closure(root_id, cityobjects, &mut selected_ids)?;

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
    collect_object_vertex_indices(
        &selected_cityobjects,
        root_id,
        &mut referenced_vertices,
        &mut visited,
    )?;

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

    let mut feature = serde_json::Map::new();
    feature.insert(
        "type".to_owned(),
        Value::String("CityJSONFeature".to_owned()),
    );
    feature.insert("id".to_owned(), Value::String(root_id.to_owned()));
    feature.insert(
        "CityObjects".to_owned(),
        Value::Object(selected_cityobjects.into_iter().collect()),
    );
    feature.insert("vertices".to_owned(), Value::Array(local_vertices));
    Ok(Value::Object(feature))
}

fn safe_file_stem(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' => ch,
            _ => '_',
        })
        .collect()
}

fn total_file_size(root: &Path) -> Result<u64> {
    let mut total = 0u64;
    for entry in WalkBuilder::new(root)
        .hidden(false)
        .follow_links(true)
        .build()
    {
        let entry = entry.map_err(|error| Error::Import(error.to_string()))?;
        if !entry
            .file_type()
            .is_some_and(|file_type| file_type.is_file())
        {
            continue;
        }
        total = total
            .checked_add(
                entry
                    .metadata()
                    .map_err(|error| Error::Import(error.to_string()))?
                    .len(),
            )
            .ok_or_else(|| Error::Import("prepared dataset size overflowed u64".to_owned()))?;
    }
    Ok(total)
}

#[allow(
    clippy::too_many_lines,
    reason = "benchmark orchestration keeps measured run order explicit"
)]
fn run_dataset(
    dataset: &PreparedDataset,
    warmth_options: &[BenchmarkWarmth],
    source_positions: &[SourcePosition],
    batch_sizes: &[usize],
    concurrent_reader_counts: &[usize],
) -> Result<Vec<BenchmarkOperationRecord>> {
    let manifest = &dataset.manifest;

    // Check if this is a Tyler pipeline dataset
    if manifest.dataset_label.contains("tyler-pipeline") {
        return run_tyler_dataset(
            dataset,
            warmth_options,
            source_positions,
            batch_sizes,
            concurrent_reader_counts,
        );
    }

    let worker_count = crate::configured_worker_count()?;
    let index_path = fresh_benchmark_index_path(manifest, worker_count)?;
    let resolved = resolve_dataset(&manifest.prepared_dataset, Some(index_path))?;

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

    let feature_count = index.package_count()?;
    let source_count = index.source_count()?;
    let cityobject_count = index.cityobject_count()?;
    let sidecar_byte_size = fs::metadata(&resolved.index_path).map_or(0, |metadata| metadata.len());

    let mut runs = vec![
        build_record(BenchmarkRecordInput {
            dataset_label: manifest.dataset_label.clone(),
            source_artifact: manifest.source_artifact.clone(),
            prepared_dataset: manifest.prepared_dataset.clone(),
            subset_size: manifest.subset_size,
            layout: manifest.layout,
            byte_size: manifest.byte_size,
            sidecar_byte_size,
            worker_count,
            operation: "dataset_open".to_owned(),
            variant: None,
            elapsed_ns: open_elapsed,
            memory: open_ended,
            feature_count,
            package_count: feature_count,
            source_count,
            cityobject_count,
            cityobject_relationship_count: manifest.cityobject_relationship_count,
            multi_geometry_cityobject_count: manifest.multi_geometry_cityobject_count,
            query_hit_count: None,
            operation_local_peak_rss_bytes: None,
        }),
        build_record(BenchmarkRecordInput {
            dataset_label: manifest.dataset_label.clone(),
            source_artifact: manifest.source_artifact.clone(),
            prepared_dataset: manifest.prepared_dataset.clone(),
            subset_size: manifest.subset_size,
            layout: manifest.layout,
            byte_size: manifest.byte_size,
            sidecar_byte_size,
            worker_count,
            operation: "index_reindex".to_owned(),
            variant: None,
            elapsed_ns: index_elapsed,
            memory: index_ended,
            feature_count,
            package_count: feature_count,
            source_count,
            cityobject_count,
            cityobject_relationship_count: manifest.cityobject_relationship_count,
            multi_geometry_cityobject_count: manifest.multi_geometry_cityobject_count,
            query_hit_count: None,
            operation_local_peak_rss_bytes: None,
        }),
    ];

    let all_refs = index.package_ref_page_after_record_id(None, feature_count.min(256))?;
    let sampled_refs = all_refs.into_iter().take(256).collect::<Vec<_>>();
    let sampled_cityobjects =
        index.cityobject_ref_page_after_record_id(None, cityobject_count.min(256))?;

    runs.extend(run_full_scan(
        &index,
        manifest,
        worker_count,
        feature_count,
        source_count,
        cityobject_count,
    )?);
    runs.extend(run_cityobject_full_scan(
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
    runs.push(run_cityobject_id_lookup(
        &index,
        manifest,
        worker_count,
        feature_count,
        source_count,
        cityobject_count,
        &sampled_cityobjects,
    )?);
    runs.extend(run_package_bbox_lookup_only(
        &index,
        manifest,
        worker_count,
        feature_count,
        source_count,
        cityobject_count,
    )?);
    runs.extend(run_cityobject_queries(
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
    runs.push(run_read_package(
        &index,
        manifest,
        worker_count,
        feature_count,
        source_count,
        cityobject_count,
        &sampled_refs,
    )?);
    runs.push(run_read_packages(
        &index,
        manifest,
        worker_count,
        feature_count,
        source_count,
        cityobject_count,
        &sampled_refs,
    )?);

    // Add reconstruction benchmarks only when CLI options are explicitly provided
    // This prevents the default benchmarks from running the expensive reconstruction suite
    let run_reconstruction_benchmarks = !warmth_options.is_empty()
        || !source_positions.is_empty()
        || !batch_sizes.is_empty()
        || !concurrent_reader_counts.is_empty();

    if run_reconstruction_benchmarks {
        let effective_warmth = if warmth_options.is_empty() {
            &[BenchmarkWarmth::Cold, BenchmarkWarmth::Warm]
        } else {
            warmth_options
        };

        let effective_positions = if source_positions.is_empty() {
            &[
                SourcePosition::First,
                SourcePosition::Middle,
                SourcePosition::Last,
            ]
        } else {
            source_positions
        };

        let effective_batch_sizes = if batch_sizes.is_empty() {
            DEFAULT_BATCH_SIZES
        } else {
            batch_sizes
        };

        let effective_concurrent_readers = if concurrent_reader_counts.is_empty() {
            DEFAULT_CONCURRENT_READERS
        } else {
            concurrent_reader_counts
        };

        // Cold scalar reconstruction benchmarks
        if effective_warmth.contains(&BenchmarkWarmth::Cold) && !effective_positions.is_empty() {
            runs.extend(run_cold_scalar_reconstruction(
                manifest,
                worker_count,
                feature_count,
                source_count,
                cityobject_count,
                effective_positions,
            )?);
        }

        // Warm scalar reconstruction benchmarks
        if effective_warmth.contains(&BenchmarkWarmth::Warm) && !effective_positions.is_empty() {
            runs.extend(run_warm_scalar_reconstruction(
                manifest,
                worker_count,
                feature_count,
                source_count,
                cityobject_count,
                effective_positions,
            )?);
        }

        // Same-source batch reconstruction benchmarks
        if !effective_batch_sizes.is_empty() {
            runs.extend(run_same_source_batches(
                &index,
                manifest,
                worker_count,
                feature_count,
                source_count,
                cityobject_count,
                effective_batch_sizes,
            )?);
        }

        // Duplicate batch reconstruction benchmark
        if effective_batch_sizes.contains(&256) {
            runs.push(run_duplicate_batch_reconstruction(
                &index,
                manifest,
                worker_count,
                feature_count,
                source_count,
                cityobject_count,
            )?);
        }

        // Bbox queries with materialization
        runs.extend(run_bbox_queries_with_materialization(
            &index,
            manifest,
            worker_count,
            feature_count,
            source_count,
            cityobject_count,
        )?);

        // Concurrent readers benchmarks
        if !effective_concurrent_readers.is_empty() {
            runs.extend(run_concurrent_readers(
                &index,
                manifest,
                worker_count,
                feature_count,
                source_count,
                cityobject_count,
                effective_concurrent_readers,
            )?);
        }
    }

    Ok(runs)
}

/// Run Tyler pipeline dataset with Tyler-specific benchmarks
#[allow(
    clippy::too_many_lines,
    reason = "Tyler pipeline dataset benchmark includes comprehensive reconstruction benchmarks"
)]
fn run_tyler_dataset(
    dataset: &PreparedDataset,
    warmth_options: &[BenchmarkWarmth],
    source_positions: &[SourcePosition],
    batch_sizes: &[usize],
    concurrent_reader_counts: &[usize],
) -> Result<Vec<BenchmarkOperationRecord>> {
    let manifest = &dataset.manifest;
    let worker_count = crate::configured_worker_count()?;
    let index_path = fresh_benchmark_index_path(manifest, worker_count)?;
    let resolved = resolve_dataset(&manifest.prepared_dataset, Some(index_path))?;

    let open_started = Instant::now();
    let mut index = CityIndex::open(resolved.storage_layout(), &resolved.index_path)?;
    let open_elapsed = u64::try_from(open_started.elapsed().as_nanos())
        .map_err(|_| Error::Import("benchmark elapsed time does not fit in u64".to_owned()))?;
    let open_ended = profile::current_memory_snapshot()?;

    let index_started = Instant::now();
    index.reindex()?;
    let index_elapsed = u64::try_from(index_started.elapsed().as_nanos())
        .map_err(|_| Error::Import("benchmark elapsed time does not fit in u64".to_owned()))?;
    let index_ended = profile::current_memory_snapshot()?;

    let feature_count = index.package_count()?;
    let source_count = index.source_count()?;
    let cityobject_count = index.cityobject_count()?;
    let sidecar_byte_size = fs::metadata(&resolved.index_path).map_or(0, |metadata| metadata.len());

    let mut runs = vec![
        build_record(BenchmarkRecordInput {
            dataset_label: manifest.dataset_label.clone(),
            source_artifact: manifest.source_artifact.clone(),
            prepared_dataset: manifest.prepared_dataset.clone(),
            subset_size: manifest.subset_size,
            layout: manifest.layout,
            byte_size: manifest.byte_size,
            sidecar_byte_size,
            worker_count,
            operation: "dataset_open".to_owned(),
            variant: None,
            elapsed_ns: open_elapsed,
            memory: open_ended,
            feature_count,
            package_count: feature_count,
            source_count,
            cityobject_count,
            cityobject_relationship_count: manifest.cityobject_relationship_count,
            multi_geometry_cityobject_count: manifest.multi_geometry_cityobject_count,
            query_hit_count: None,
            operation_local_peak_rss_bytes: None,
        }),
        build_record(BenchmarkRecordInput {
            dataset_label: manifest.dataset_label.clone(),
            source_artifact: manifest.source_artifact.clone(),
            prepared_dataset: manifest.prepared_dataset.clone(),
            subset_size: manifest.subset_size,
            layout: manifest.layout,
            byte_size: manifest.byte_size,
            sidecar_byte_size,
            worker_count,
            operation: "index_reindex".to_owned(),
            variant: None,
            elapsed_ns: index_elapsed,
            memory: index_ended,
            feature_count,
            package_count: feature_count,
            source_count,
            cityobject_count,
            cityobject_relationship_count: manifest.cityobject_relationship_count,
            multi_geometry_cityobject_count: manifest.multi_geometry_cityobject_count,
            query_hit_count: None,
            operation_local_peak_rss_bytes: None,
        }),
    ];

    // Add Tyler pipeline specific benchmarks
    runs.extend(run_tyler_pipeline(
        &index,
        manifest,
        worker_count,
        feature_count,
        source_count,
        cityobject_count,
        None,
    )?);

    // Add reconstruction benchmarks for Tyler pipeline - always included for multi-file corpora
    // This ensures reconstruction benchmarks work with multi-file corpora
    let run_reconstruction_benchmarks = true;

    if run_reconstruction_benchmarks {
        let effective_warmth = if warmth_options.is_empty() {
            &[BenchmarkWarmth::Cold, BenchmarkWarmth::Warm]
        } else {
            warmth_options
        };

        let effective_positions = if source_positions.is_empty() {
            &[
                SourcePosition::First,
                SourcePosition::Middle,
                SourcePosition::Last,
            ]
        } else {
            source_positions
        };

        let effective_batch_sizes = if batch_sizes.is_empty() {
            DEFAULT_BATCH_SIZES
        } else {
            batch_sizes
        };

        let effective_concurrent_readers = if concurrent_reader_counts.is_empty() {
            DEFAULT_CONCURRENT_READERS
        } else {
            concurrent_reader_counts
        };

        // Cold scalar reconstruction benchmarks
        if effective_warmth.contains(&BenchmarkWarmth::Cold) && !effective_positions.is_empty() {
            runs.extend(run_cold_scalar_reconstruction(
                manifest,
                worker_count,
                feature_count,
                source_count,
                cityobject_count,
                effective_positions,
            )?);
        }

        // Warm scalar reconstruction benchmarks
        if effective_warmth.contains(&BenchmarkWarmth::Warm) && !effective_positions.is_empty() {
            runs.extend(run_warm_scalar_reconstruction(
                manifest,
                worker_count,
                feature_count,
                source_count,
                cityobject_count,
                effective_positions,
            )?);
        }

        // Same-source batch reconstruction benchmarks
        if !effective_batch_sizes.is_empty() {
            runs.extend(run_same_source_batches(
                &index,
                manifest,
                worker_count,
                feature_count,
                source_count,
                cityobject_count,
                effective_batch_sizes,
            )?);
        }

        // Concurrent readers benchmarks
        if !effective_concurrent_readers.is_empty() {
            runs.extend(run_concurrent_readers(
                &index,
                manifest,
                worker_count,
                feature_count,
                source_count,
                cityobject_count,
                effective_concurrent_readers,
            )?);
        }
    }

    Ok(runs)
}

/// Simulates the Tyler 3-stage pipeline: extent construction, grid indexing, and feature processing
#[allow(
    clippy::too_many_lines,
    reason = "Tyler pipeline simulation requires explicit step sequencing for accurate benchmarking"
)]
fn run_tyler_pipeline(
    index: &CityIndex,
    manifest: &BenchmarkManifest,
    worker_count: usize,
    feature_count: usize,
    source_count: usize,
    cityobject_count: usize,
    event_path: Option<&Path>,
) -> Result<Vec<BenchmarkOperationRecord>> {
    use rayon::prelude::*;

    let mut runs = Vec::new();
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(worker_count)
        .build()
        .map_err(|error| {
            Error::Import(format!("failed to build benchmark worker pool: {error}"))
        })?;

    // Stage 1: Extent construction - full scan to compute bbox
    // This simulates Tyler's first pass to compute extent
    append_stage_event(
        event_path,
        "stage_start",
        "tyler_extent_construction",
        worker_count,
        None,
        None,
    )?;
    let extent_started = Instant::now();
    let mut extent_bbox: Option<BBox> = None;
    let mut extent_feature_count = 0usize;

    let mut after_record_id = None;
    loop {
        let page = index.package_ref_page_after_record_id(after_record_id, 512)?;
        if page.is_empty() {
            break;
        }

        for package_ref in &page {
            if let Some(bounds) = package_ref.bounds {
                let package_bbox = BBox {
                    min_x: bounds.min_x,
                    max_x: bounds.max_x,
                    min_y: bounds.min_y,
                    max_y: bounds.max_y,
                };
                extent_bbox = Some(match extent_bbox {
                    None => package_bbox,
                    Some(existing) => BBox {
                        min_x: existing.min_x.min(package_bbox.min_x),
                        max_x: existing.max_x.max(package_bbox.max_x),
                        min_y: existing.min_y.min(package_bbox.min_y),
                        max_y: existing.max_y.max(package_bbox.max_y),
                    },
                });
            }
            extent_feature_count += 1;
        }

        after_record_id = page.last().map(|package| package.record_id);
    }

    let extent_elapsed = u64::try_from(extent_started.elapsed().as_nanos())
        .map_err(|_| Error::Import("benchmark elapsed time does not fit in u64".to_owned()))?;
    let extent_memory = profile::current_memory_snapshot()?;
    append_stage_event(
        event_path,
        "stage_end",
        "tyler_extent_construction",
        worker_count,
        Some(extent_elapsed),
        Some(1),
    )?;

    runs.push(build_record(BenchmarkRecordInput {
        dataset_label: manifest.dataset_label.clone(),
        source_artifact: manifest.source_artifact.clone(),
        prepared_dataset: manifest.prepared_dataset.clone(),
        subset_size: manifest.subset_size,
        layout: manifest.layout,
        byte_size: manifest.byte_size,
        sidecar_byte_size: fs::metadata(
            manifest
                .prepared_dataset
                .join(format!(".cityjson-index.worker-{worker_count}.sqlite")),
        )
        .map_or(0, |metadata| metadata.len()),
        worker_count,
        operation: "tyler_extent_construction".to_owned(),
        variant: None,
        elapsed_ns: extent_elapsed,
        memory: extent_memory,
        feature_count,
        package_count: feature_count,
        source_count,
        cityobject_count,
        cityobject_relationship_count: manifest.cityobject_relationship_count,
        multi_geometry_cityobject_count: manifest.multi_geometry_cityobject_count,
        query_hit_count: Some(extent_feature_count),
        operation_local_peak_rss_bytes: None,
    }));

    // Stage 2: Grid indexing - parallel processing of features for grid assignment
    // This simulates Tyler's second pass where features are assigned to grid cells
    append_stage_event(
        event_path,
        "stage_start",
        "tyler_grid_indexing",
        worker_count,
        None,
        None,
    )?;
    let grid_started = Instant::now();
    let all_refs = index.package_ref_page_after_record_id(None, feature_count)?;

    // Process in parallel chunks using rayon (simulating Tyler's parallelism)
    let chunk_size = 256;
    let grid_workers = Mutex::new(BTreeSet::new());
    let grid_feature_count: usize = pool.install(|| {
        all_refs
            .chunks(chunk_size)
            .par_bridge()
            .map(|chunk| {
                if let Some(index) = rayon::current_thread_index() {
                    grid_workers
                        .lock()
                        .expect("grid worker observation mutex was poisoned")
                        .insert(index);
                }
                let mut chunk_count = 0usize;
                for package_ref in chunk {
                    if let Some(bounds) = package_ref.bounds {
                        let _ = BBox {
                            min_x: bounds.min_x,
                            max_x: bounds.max_x,
                            min_y: bounds.min_y,
                            max_y: bounds.max_y,
                        };
                    }
                    chunk_count += 1;
                }
                chunk_count
            })
            .sum()
    });

    let grid_elapsed = u64::try_from(grid_started.elapsed().as_nanos())
        .map_err(|_| Error::Import("benchmark elapsed time does not fit in u64".to_owned()))?;
    let grid_memory = profile::current_memory_snapshot()?;
    let observed_grid_workers = grid_workers
        .into_inner()
        .map_err(|_| Error::Import("grid worker observation mutex was poisoned".to_owned()))?
        .len();
    append_stage_event(
        event_path,
        "stage_end",
        "tyler_grid_indexing",
        worker_count,
        Some(grid_elapsed),
        Some(observed_grid_workers),
    )?;

    runs.push(build_record(BenchmarkRecordInput {
        dataset_label: manifest.dataset_label.clone(),
        source_artifact: manifest.source_artifact.clone(),
        prepared_dataset: manifest.prepared_dataset.clone(),
        subset_size: manifest.subset_size,
        layout: manifest.layout,
        byte_size: manifest.byte_size,
        sidecar_byte_size: fs::metadata(
            manifest
                .prepared_dataset
                .join(format!(".cityjson-index.worker-{worker_count}.sqlite")),
        )
        .map_or(0, |metadata| metadata.len()),
        worker_count,
        operation: "tyler_grid_indexing".to_owned(),
        variant: Some(format!("chunk_size-{chunk_size}")),
        elapsed_ns: grid_elapsed,
        memory: grid_memory,
        feature_count,
        package_count: feature_count,
        source_count,
        cityobject_count,
        cityobject_relationship_count: manifest.cityobject_relationship_count,
        multi_geometry_cityobject_count: manifest.multi_geometry_cityobject_count,
        query_hit_count: Some(grid_feature_count),
        operation_local_peak_rss_bytes: None,
    }));

    runs.push(run_tyler_feature_materialization(
        manifest,
        worker_count,
        feature_count,
        source_count,
        cityobject_count,
        &all_refs,
        &pool,
        event_path,
    )?);

    Ok(runs)
}

#[allow(
    clippy::too_many_arguments,
    reason = "the benchmark record needs the complete dataset context"
)]
fn run_tyler_feature_materialization(
    manifest: &BenchmarkManifest,
    worker_count: usize,
    feature_count: usize,
    source_count: usize,
    cityobject_count: usize,
    all_refs: &[IndexedPackageRef],
    pool: &rayon::ThreadPool,
    event_path: Option<&Path>,
) -> Result<BenchmarkOperationRecord> {
    use rayon::prelude::*;

    let index_path = manifest
        .prepared_dataset
        .join(format!(".cityjson-index.worker-{worker_count}.sqlite"));
    let resolved = resolve_dataset(&manifest.prepared_dataset, Some(index_path))?;
    let layout = resolved.storage_layout();
    let resolved_index_path = resolved.index_path.clone();
    let observed_workers = Mutex::new(BTreeSet::new());

    append_stage_event(
        event_path,
        "stage_start",
        "tyler_feature_materialization",
        worker_count,
        None,
        None,
    )?;
    let started = Instant::now();
    let read_count = pool.install(|| {
        all_refs
            .chunks(BENCH_CJINDEX_PARALLEL_CHUNK_SIZE)
            .par_bridge()
            .map(|chunk| {
                if let Some(index) = rayon::current_thread_index() {
                    observed_workers
                        .lock()
                        .expect("materialization worker observation mutex was poisoned")
                        .insert(index);
                }
                BENCH_INDEX_THREAD_LOCAL.with(|cell| {
                    if cell.borrow().is_none() {
                        let index = CityIndex::open(layout.clone(), &resolved_index_path)
                            .expect("benchmark worker should open its CityIndex");
                        *cell.borrow_mut() = Some((resolved_index_path.clone(), index));
                    }

                    let slot = cell.borrow();
                    let (_, thread_index) = slot
                        .as_ref()
                        .expect("benchmark worker CityIndex should be initialized");
                    for package_ref in chunk {
                        let _model = thread_index
                            .read_package(package_ref)
                            .expect("benchmark worker should reconstruct its package");
                    }
                    chunk.len()
                })
            })
            .sum::<usize>()
    });
    let elapsed_ns = u64::try_from(started.elapsed().as_nanos())
        .map_err(|_| Error::Import("benchmark elapsed time does not fit in u64".to_owned()))?;
    let memory = profile::current_memory_snapshot()?;
    let observed_worker_count = observed_workers
        .into_inner()
        .map_err(|_| {
            Error::Import("materialization worker observation mutex was poisoned".to_owned())
        })?
        .len();
    append_stage_event(
        event_path,
        "stage_end",
        "tyler_feature_materialization",
        worker_count,
        Some(elapsed_ns),
        Some(observed_worker_count),
    )?;

    if event_path.is_some() {
        record_and_clear_worker_vertex_caches(event_path, pool, worker_count)?;
    }

    Ok(build_record(BenchmarkRecordInput {
        dataset_label: manifest.dataset_label.clone(),
        source_artifact: manifest.source_artifact.clone(),
        prepared_dataset: manifest.prepared_dataset.clone(),
        subset_size: manifest.subset_size,
        layout: manifest.layout,
        byte_size: manifest.byte_size,
        sidecar_byte_size: fs::metadata(&resolved_index_path).map_or(0, |metadata| metadata.len()),
        worker_count,
        operation: "tyler_feature_materialization".to_owned(),
        variant: Some(format!("observed-workers-{observed_worker_count}")),
        elapsed_ns,
        memory,
        feature_count,
        package_count: feature_count,
        source_count,
        cityobject_count,
        cityobject_relationship_count: manifest.cityobject_relationship_count,
        multi_geometry_cityobject_count: manifest.multi_geometry_cityobject_count,
        query_hit_count: Some(read_count),
        operation_local_peak_rss_bytes: None,
    }))
}

fn run_isolated_tyler_target(
    dataset: &PreparedDataset,
    target: BenchmarkProfileTarget,
    worker_count: usize,
    event_path: Option<&Path>,
) -> Result<Vec<BenchmarkOperationRecord>> {
    let manifest = &dataset.manifest;
    let index_path = benchmark_index_path(manifest, worker_count);
    if !index_path.exists() {
        return Err(Error::Import(format!(
            "prepared Tyler sidecar {} disappeared before profiling",
            index_path.display()
        )));
    }
    let resolved = resolve_dataset(&manifest.prepared_dataset, Some(index_path))?;
    let index = CityIndex::open(resolved.storage_layout(), &resolved.index_path)?;
    let feature_count = index.package_count()?;
    let source_count = index.source_count()?;
    let cityobject_count = index.cityobject_count()?;
    if feature_count != manifest.feature_count
        || source_count != manifest.source_count
        || cityobject_count != manifest.cityobject_count
    {
        return Err(Error::Import(format!(
            "prepared Tyler sidecar does not match its manifest: packages {feature_count}/{}, sources {source_count}/{}, CityObjects {cityobject_count}/{}",
            manifest.feature_count, manifest.source_count, manifest.cityobject_count
        )));
    }

    match target {
        BenchmarkProfileTarget::TylerPipeline => run_tyler_pipeline(
            &index,
            manifest,
            worker_count,
            feature_count,
            source_count,
            cityobject_count,
            event_path,
        ),
        BenchmarkProfileTarget::TylerFeatureMaterialization => {
            let all_refs = index.package_ref_page_after_record_id(None, feature_count)?;
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(worker_count)
                .build()
                .map_err(|error| {
                    Error::Import(format!("failed to build benchmark worker pool: {error}"))
                })?;
            Ok(vec![run_tyler_feature_materialization(
                manifest,
                worker_count,
                feature_count,
                source_count,
                cityobject_count,
                &all_refs,
                &pool,
                event_path,
            )?])
        }
    }
}

fn append_stage_event(
    path: Option<&Path>,
    event: &str,
    stage: &str,
    worker_count: usize,
    elapsed_ns: Option<u64>,
    observed_worker_count: Option<usize>,
) -> Result<()> {
    let Some(path) = path else {
        return Ok(());
    };
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    serde_json::to_writer(
        &mut file,
        &BenchmarkStageEvent {
            schema_version: BENCHMARK_STAGE_EVENT_SCHEMA_VERSION,
            timestamp_ns: unix_time_ns()?,
            event,
            stage,
            worker_count,
            elapsed_ns,
            observed_worker_count,
        },
    )
    .map_err(|error| Error::Import(error.to_string()))?;
    file.write_all(b"\n")?;
    file.flush()?;
    Ok(())
}

fn collect_worker_vertex_cache_stats(pool: &rayon::ThreadPool) -> Vec<WorkerVertexCacheStats> {
    pool.broadcast(|context| {
        BENCH_INDEX_THREAD_LOCAL.with(|cell| {
            let stats = cell
                .borrow()
                .as_ref()
                .map_or_else(crate::VertexCacheStats::default, |(_, index)| {
                    index.vertex_cache_stats()
                });
            WorkerVertexCacheStats {
                worker_index: context.index(),
                cached_source_count: stats.cached_source_count,
                cached_vertex_count: stats.cached_vertex_count,
                vertex_capacity_bytes: stats.vertex_capacity_bytes,
            }
        })
    })
}

fn record_and_clear_worker_vertex_caches(
    event_path: Option<&Path>,
    pool: &rayon::ThreadPool,
    worker_count: usize,
) -> Result<()> {
    let worker_cache_stats = collect_worker_vertex_cache_stats(pool);
    append_cache_checkpoint_event(
        event_path,
        "cache_before_drop",
        worker_count,
        &worker_cache_stats,
    )?;
    std::thread::sleep(PROFILE_CHECKPOINT_SETTLE_TIME);
    clear_worker_vertex_caches(pool);
    std::thread::sleep(PROFILE_CHECKPOINT_SETTLE_TIME);
    let cleared_cache_stats = collect_worker_vertex_cache_stats(pool);
    append_cache_checkpoint_event(
        event_path,
        "cache_after_drop",
        worker_count,
        &cleared_cache_stats,
    )?;
    std::thread::sleep(PROFILE_CHECKPOINT_SETTLE_TIME);
    Ok(())
}

fn clear_worker_vertex_caches(pool: &rayon::ThreadPool) {
    pool.broadcast(|_| {
        BENCH_INDEX_THREAD_LOCAL.with(|cell| {
            cell.borrow_mut().take();
        });
    });
}

fn append_cache_checkpoint_event(
    path: Option<&Path>,
    event: &str,
    worker_count: usize,
    workers: &[WorkerVertexCacheStats],
) -> Result<()> {
    let Some(path) = path else {
        return Ok(());
    };
    let memory = profile::current_memory_snapshot()?;
    let cached_source_count = workers
        .iter()
        .map(|worker| worker.cached_source_count)
        .sum();
    let cached_vertex_count = workers
        .iter()
        .map(|worker| worker.cached_vertex_count)
        .sum();
    let vertex_capacity_bytes = workers.iter().try_fold(0_u64, |total, worker| {
        total
            .checked_add(worker.vertex_capacity_bytes)
            .ok_or_else(|| Error::Import("worker vertex cache capacity overflowed u64".to_owned()))
    })?;
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    serde_json::to_writer(
        &mut file,
        &BenchmarkCacheCheckpointEvent {
            schema_version: BENCHMARK_STAGE_EVENT_SCHEMA_VERSION,
            timestamp_ns: unix_time_ns()?,
            event,
            stage: "tyler_feature_materialization",
            worker_count,
            current_rss_bytes: memory.current_rss_bytes,
            process_peak_rss_bytes: memory.process_peak_rss_bytes,
            cached_source_count,
            cached_vertex_count,
            vertex_capacity_bytes,
            workers,
        },
    )
    .map_err(|error| Error::Import(error.to_string()))?;
    file.write_all(b"\n")?;
    file.flush()?;
    Ok(())
}

fn unix_time_ns() -> Result<u64> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| Error::Import(format!("system clock is before Unix epoch: {error}")))?;
    u64::try_from(elapsed.as_nanos())
        .map_err(|_| Error::Import("Unix timestamp does not fit in u64".to_owned()))
}

fn fresh_benchmark_index_path(
    manifest: &BenchmarkManifest,
    worker_count: usize,
) -> Result<PathBuf> {
    let index_path = benchmark_index_path(manifest, worker_count);
    remove_file_if_exists(&index_path)?;
    remove_file_if_exists(&index_path.with_extension("sqlite-wal"))?;
    remove_file_if_exists(&index_path.with_extension("sqlite-shm"))?;
    Ok(index_path)
}

fn benchmark_index_path(manifest: &BenchmarkManifest, worker_count: usize) -> PathBuf {
    manifest
        .prepared_dataset
        .join(format!(".cityjson-index.worker-{worker_count}.sqlite"))
}

fn remove_file_if_exists(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
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
    let mut after_record_id = None;
    loop {
        let page = index.package_ref_page_after_record_id(after_record_id, 512)?;
        if page.is_empty() {
            break;
        }
        after_record_id = page.last().map(|package| package.record_id);
        count += page.len();
    }
    let elapsed_ns = u64::try_from(started.elapsed().as_nanos())
        .map_err(|_| Error::Import("benchmark elapsed time does not fit in u64".to_owned()))?;
    let memory = profile::current_memory_snapshot()?;
    Ok(vec![build_record(BenchmarkRecordInput {
        dataset_label: manifest.dataset_label.clone(),
        source_artifact: manifest.source_artifact.clone(),
        prepared_dataset: manifest.prepared_dataset.clone(),
        subset_size: manifest.subset_size,
        byte_size: manifest.byte_size,
        layout: manifest.layout,
        sidecar_byte_size: fs::metadata(
            manifest
                .prepared_dataset
                .join(format!(".cityjson-index.worker-{worker_count}.sqlite")),
        )
        .map_or(0, |metadata| metadata.len()),
        worker_count,
        operation: "full_scan_reference_iteration".to_owned(),
        variant: None,
        elapsed_ns,
        memory,
        feature_count,
        package_count: feature_count,
        source_count,
        cityobject_count,
        cityobject_relationship_count: manifest.cityobject_relationship_count,
        multi_geometry_cityobject_count: manifest.multi_geometry_cityobject_count,
        query_hit_count: Some(count),
        operation_local_peak_rss_bytes: None,
    })])
}

fn run_cityobject_full_scan(
    index: &CityIndex,
    manifest: &BenchmarkManifest,
    worker_count: usize,
    feature_count: usize,
    source_count: usize,
    cityobject_count: usize,
) -> Result<Vec<BenchmarkOperationRecord>> {
    let started = Instant::now();
    let mut count = 0usize;
    let mut after_record_id = None;
    loop {
        let page = index.cityobject_ref_page_after_record_id(after_record_id, 512)?;
        if page.is_empty() {
            break;
        }
        after_record_id = page.last().map(|cityobject| cityobject.record_id);
        count += page.len();
    }
    let elapsed_ns = u64::try_from(started.elapsed().as_nanos())
        .map_err(|_| Error::Import("benchmark elapsed time does not fit in u64".to_owned()))?;
    let memory = profile::current_memory_snapshot()?;
    Ok(vec![build_record(BenchmarkRecordInput {
        dataset_label: manifest.dataset_label.clone(),
        source_artifact: manifest.source_artifact.clone(),
        prepared_dataset: manifest.prepared_dataset.clone(),
        subset_size: manifest.subset_size,
        byte_size: manifest.byte_size,
        layout: manifest.layout,
        sidecar_byte_size: fs::metadata(
            manifest
                .prepared_dataset
                .join(format!(".cityjson-index.worker-{worker_count}.sqlite")),
        )
        .map_or(0, |metadata| metadata.len()),
        worker_count,
        operation: "cityobject_full_scan_reference_iteration".to_owned(),
        variant: None,
        elapsed_ns,
        memory,
        feature_count,
        package_count: feature_count,
        source_count,
        cityobject_count,
        cityobject_relationship_count: manifest.cityobject_relationship_count,
        multi_geometry_cityobject_count: manifest.multi_geometry_cityobject_count,
        query_hit_count: Some(count),
        operation_local_peak_rss_bytes: None,
    })])
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
        let hit = index.get_packages(&feature_id)?;
        let elapsed_ns = u64::try_from(started.elapsed().as_nanos())
            .map_err(|_| Error::Import("benchmark elapsed time does not fit in u64".to_owned()))?;
        let memory = profile::current_memory_snapshot()?;
        runs.push(build_record(BenchmarkRecordInput {
            dataset_label: manifest.dataset_label.clone(),
            source_artifact: manifest.source_artifact.clone(),
            prepared_dataset: manifest.prepared_dataset.clone(),
            subset_size: manifest.subset_size,
            layout: manifest.layout,
            byte_size: manifest.byte_size,
            sidecar_byte_size: fs::metadata(
                manifest
                    .prepared_dataset
                    .join(format!(".cityjson-index.worker-{worker_count}.sqlite")),
            )
            .map_or(0, |metadata| metadata.len()),
            worker_count,
            operation: "get".to_owned(),
            variant: Some(feature_id),
            elapsed_ns,
            memory,
            feature_count,
            package_count: feature_count,
            source_count,
            cityobject_count,
            cityobject_relationship_count: manifest.cityobject_relationship_count,
            multi_geometry_cityobject_count: manifest.multi_geometry_cityobject_count,
            query_hit_count: Some(hit.len()),
            operation_local_peak_rss_bytes: None,
        }));
    }
    Ok(runs)
}

fn run_cityobject_id_lookup(
    index: &CityIndex,
    manifest: &BenchmarkManifest,
    worker_count: usize,
    feature_count: usize,
    source_count: usize,
    cityobject_count: usize,
    refs: &[crate::IndexedCityObjectRef],
) -> Result<BenchmarkOperationRecord> {
    let ids = refs
        .iter()
        .map(|cityobject| cityobject.external_id.as_str())
        .collect::<Vec<_>>();
    let started = Instant::now();
    let hits = index.lookup_cityobject_refs_for_ids(&ids)?;
    let elapsed_ns = u64::try_from(started.elapsed().as_nanos())
        .map_err(|_| Error::Import("benchmark elapsed time does not fit in u64".to_owned()))?;
    let memory = profile::current_memory_snapshot()?;
    Ok(build_record(BenchmarkRecordInput {
        dataset_label: manifest.dataset_label.clone(),
        source_artifact: manifest.source_artifact.clone(),
        prepared_dataset: manifest.prepared_dataset.clone(),
        subset_size: manifest.subset_size,
        layout: manifest.layout,
        byte_size: manifest.byte_size,
        sidecar_byte_size: fs::metadata(
            manifest
                .prepared_dataset
                .join(format!(".cityjson-index.worker-{worker_count}.sqlite")),
        )
        .map_or(0, |metadata| metadata.len()),
        worker_count,
        operation: "cityobject_id_lookup".to_owned(),
        variant: Some(format!("sample-{}", ids.len())),
        elapsed_ns,
        memory,
        feature_count,
        package_count: feature_count,
        source_count,
        cityobject_count,
        cityobject_relationship_count: manifest.cityobject_relationship_count,
        multi_geometry_cityobject_count: manifest.multi_geometry_cityobject_count,
        query_hit_count: Some(hits.len()),
        operation_local_peak_rss_bytes: None,
    }))
}

fn run_package_bbox_lookup_only(
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
        let hits = index.query_package_refs(&window.bbox)?;
        let elapsed_ns = u64::try_from(started.elapsed().as_nanos())
            .map_err(|_| Error::Import("benchmark elapsed time does not fit in u64".to_owned()))?;
        let memory = profile::current_memory_snapshot()?;
        runs.push(build_record(BenchmarkRecordInput {
            dataset_label: manifest.dataset_label.clone(),
            source_artifact: manifest.source_artifact.clone(),
            prepared_dataset: manifest.prepared_dataset.clone(),
            subset_size: manifest.subset_size,
            layout: manifest.layout,
            byte_size: manifest.byte_size,
            sidecar_byte_size: fs::metadata(
                manifest
                    .prepared_dataset
                    .join(format!(".cityjson-index.worker-{worker_count}.sqlite")),
            )
            .map_or(0, |metadata| metadata.len()),
            worker_count,
            operation: "package_bbox_lookup_only".to_owned(),
            variant: Some(window.label.clone()),
            elapsed_ns,
            memory,
            feature_count,
            package_count: feature_count,
            source_count,
            cityobject_count,
            cityobject_relationship_count: manifest.cityobject_relationship_count,
            multi_geometry_cityobject_count: manifest.multi_geometry_cityobject_count,
            query_hit_count: Some(hits.len()),
            operation_local_peak_rss_bytes: None,
        }));
    }
    Ok(runs)
}

fn run_cityobject_queries(
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
        let hits = index.query_cityobject_refs(&window.bbox)?;
        let elapsed_ns = u64::try_from(started.elapsed().as_nanos())
            .map_err(|_| Error::Import("benchmark elapsed time does not fit in u64".to_owned()))?;
        let memory = profile::current_memory_snapshot()?;
        runs.push(build_record(BenchmarkRecordInput {
            dataset_label: manifest.dataset_label.clone(),
            source_artifact: manifest.source_artifact.clone(),
            prepared_dataset: manifest.prepared_dataset.clone(),
            subset_size: manifest.subset_size,
            layout: manifest.layout,
            byte_size: manifest.byte_size,
            sidecar_byte_size: fs::metadata(
                manifest
                    .prepared_dataset
                    .join(format!(".cityjson-index.worker-{worker_count}.sqlite")),
            )
            .map_or(0, |metadata| metadata.len()),
            worker_count,
            operation: "cityobject_bbox_query".to_owned(),
            variant: Some(window.label.clone()),
            elapsed_ns,
            memory,
            feature_count,
            package_count: feature_count,
            source_count,
            cityobject_count,
            cityobject_relationship_count: manifest.cityobject_relationship_count,
            multi_geometry_cityobject_count: manifest.multi_geometry_cityobject_count,
            query_hit_count: Some(hits.len()),
            operation_local_peak_rss_bytes: None,
        }));
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
        let hits = index.query_package_refs(&window.bbox)?;
        let _packages = index.read_packages(&hits)?;
        let elapsed_ns = u64::try_from(started.elapsed().as_nanos())
            .map_err(|_| Error::Import("benchmark elapsed time does not fit in u64".to_owned()))?;
        let memory = profile::current_memory_snapshot()?;
        runs.push(build_record(BenchmarkRecordInput {
            dataset_label: manifest.dataset_label.clone(),
            source_artifact: manifest.source_artifact.clone(),
            prepared_dataset: manifest.prepared_dataset.clone(),
            subset_size: manifest.subset_size,
            layout: manifest.layout,
            byte_size: manifest.byte_size,
            sidecar_byte_size: fs::metadata(
                manifest
                    .prepared_dataset
                    .join(format!(".cityjson-index.worker-{worker_count}.sqlite")),
            )
            .map_or(0, |metadata| metadata.len()),
            worker_count,
            operation: "bbox_query".to_owned(),
            variant: Some(window.label.clone()),
            elapsed_ns,
            memory,
            feature_count,
            package_count: feature_count,
            source_count,
            cityobject_count,
            cityobject_relationship_count: manifest.cityobject_relationship_count,
            multi_geometry_cityobject_count: manifest.multi_geometry_cityobject_count,
            query_hit_count: Some(hits.len()),
            operation_local_peak_rss_bytes: None,
        }));
    }
    Ok(runs)
}

fn run_read_package(
    index: &CityIndex,
    manifest: &BenchmarkManifest,
    worker_count: usize,
    feature_count: usize,
    source_count: usize,
    cityobject_count: usize,
    refs: &[crate::IndexedPackageRef],
) -> Result<BenchmarkOperationRecord> {
    let started = Instant::now();
    let mut reconstructed = 0usize;
    for package in refs {
        let _model = index.read_package(package)?;
        reconstructed += 1;
    }
    let elapsed_ns = u64::try_from(started.elapsed().as_nanos())
        .map_err(|_| Error::Import("benchmark elapsed time does not fit in u64".to_owned()))?;
    let memory = profile::current_memory_snapshot()?;
    Ok(build_record(BenchmarkRecordInput {
        dataset_label: manifest.dataset_label.clone(),
        source_artifact: manifest.source_artifact.clone(),
        prepared_dataset: manifest.prepared_dataset.clone(),
        subset_size: manifest.subset_size,
        byte_size: manifest.byte_size,
        layout: manifest.layout,
        sidecar_byte_size: fs::metadata(
            manifest
                .prepared_dataset
                .join(format!(".cityjson-index.worker-{worker_count}.sqlite")),
        )
        .map_or(0, |metadata| metadata.len()),
        worker_count,
        operation: "read_package".to_owned(),
        variant: Some(format!("sample-{}", refs.len())),
        elapsed_ns,
        memory,
        feature_count,
        package_count: feature_count,
        source_count,
        cityobject_count,
        cityobject_relationship_count: manifest.cityobject_relationship_count,
        multi_geometry_cityobject_count: manifest.multi_geometry_cityobject_count,
        query_hit_count: Some(reconstructed),
        operation_local_peak_rss_bytes: None,
    }))
}

fn run_read_packages(
    index: &CityIndex,
    manifest: &BenchmarkManifest,
    worker_count: usize,
    feature_count: usize,
    source_count: usize,
    cityobject_count: usize,
    refs: &[crate::IndexedPackageRef],
) -> Result<BenchmarkOperationRecord> {
    let started = Instant::now();
    let packages = index.read_packages(refs)?;
    let elapsed_ns = u64::try_from(started.elapsed().as_nanos())
        .map_err(|_| Error::Import("benchmark elapsed time does not fit in u64".to_owned()))?;
    let memory = profile::current_memory_snapshot()?;
    Ok(build_record(BenchmarkRecordInput {
        dataset_label: manifest.dataset_label.clone(),
        source_artifact: manifest.source_artifact.clone(),
        prepared_dataset: manifest.prepared_dataset.clone(),
        subset_size: manifest.subset_size,
        byte_size: manifest.byte_size,
        layout: manifest.layout,
        sidecar_byte_size: fs::metadata(
            manifest
                .prepared_dataset
                .join(format!(".cityjson-index.worker-{worker_count}.sqlite")),
        )
        .map_or(0, |metadata| metadata.len()),
        worker_count,
        operation: "read_packages".to_owned(),
        variant: Some(format!("sample-{}", refs.len())),
        elapsed_ns,
        memory,
        feature_count,
        package_count: feature_count,
        source_count,
        cityobject_count,
        cityobject_relationship_count: manifest.cityobject_relationship_count,
        multi_geometry_cityobject_count: manifest.multi_geometry_cityobject_count,
        query_hit_count: Some(packages.len()),
        operation_local_peak_rss_bytes: None,
    }))
}

fn build_record(input: BenchmarkRecordInput) -> BenchmarkOperationRecord {
    BenchmarkOperationRecord {
        dataset_label: input.dataset_label,
        source_artifact: input.source_artifact,
        prepared_dataset: input.prepared_dataset,
        subset_size: input.subset_size,
        layout: input.layout,
        byte_size: input.byte_size,
        sidecar_byte_size: input.sidecar_byte_size,
        worker_count: input.worker_count,
        operation: input.operation,
        variant: input.variant,
        elapsed_ns: input.elapsed_ns,
        current_rss_bytes: input.memory.current_rss_bytes,
        process_peak_rss_bytes: input.memory.process_peak_rss_bytes,
        peak_rss_bytes: input.memory.peak_rss_bytes,
        feature_count: input.feature_count,
        package_count: input.package_count,
        source_count: input.source_count,
        cityobject_count: input.cityobject_count,
        cityobject_relationship_count: input.cityobject_relationship_count,
        multi_geometry_cityobject_count: input.multi_geometry_cityobject_count,
        query_hit_count: input.query_hit_count,
        operation_local_peak_rss_bytes: input.operation_local_peak_rss_bytes,
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
            std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get),
            4,
        ];
    }
    requested.sort_unstable();
    requested.dedup();
    requested
}

fn with_worker_count_env<T>(worker_count: usize, f: impl FnOnce() -> Result<T>) -> Result<T> {
    struct WorkerCountEnvGuard {
        previous: Option<std::ffi::OsString>,
    }

    impl Drop for WorkerCountEnvGuard {
        fn drop(&mut self) {
            // SAFETY: the benchmark runner sets and restores the variable on the
            // current thread immediately around a single indexing run.
            unsafe {
                match self.previous.take() {
                    Some(previous) => std::env::set_var(crate::WORKER_COUNT_ENV, previous),
                    None => std::env::remove_var(crate::WORKER_COUNT_ENV),
                }
            }
        }
    }

    let previous = std::env::var_os(crate::WORKER_COUNT_ENV);
    let _guard = WorkerCountEnvGuard { previous };
    // SAFETY: the benchmark process is single-threaded around environment
    // mutation for a given run, and the variable is restored by the guard.
    unsafe {
        std::env::set_var(crate::WORKER_COUNT_ENV, worker_count.to_string());
    }
    f()
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

fn count_cityobject_relationships(document: &Value) -> Result<usize> {
    let cityobjects = document
        .get("CityObjects")
        .and_then(Value::as_object)
        .ok_or_else(|| Error::Import("CityJSON document is missing CityObjects".to_owned()))?;
    let mut relationships = BTreeSet::new();
    for (object_id, object) in cityobjects {
        if let Some(children) = object.get("children").and_then(Value::as_array) {
            for child in children {
                if let Some(child_id) = child.as_str() {
                    relationships.insert((object_id.clone(), child_id.to_owned()));
                }
            }
        }
        if let Some(parents) = object.get("parents").and_then(Value::as_array) {
            for parent in parents {
                if let Some(parent_id) = parent.as_str() {
                    relationships.insert((parent_id.to_owned(), object_id.clone()));
                }
            }
        }
    }
    Ok(relationships.len())
}

fn count_multi_geometry_cityobjects(document: &Value) -> Result<usize> {
    let cityobjects = document
        .get("CityObjects")
        .and_then(Value::as_object)
        .ok_or_else(|| Error::Import("CityJSON document is missing CityObjects".to_owned()))?;
    Ok(cityobjects
        .values()
        .filter(|object| {
            object
                .get("geometry")
                .and_then(Value::as_array)
                .is_some_and(|geometries| geometries.len() > 1)
        })
        .count())
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
    let root_ids = extract_root_ids(document)?;
    let selected_roots = root_ids.into_iter().take(limit).collect::<Vec<_>>();
    subset_cityjson_document_by_roots(document, &selected_roots)
}

fn subset_cityjson_document_by_roots(document: &Value, selected_roots: &[String]) -> Result<Value> {
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
    let mut selected_ids = BTreeSet::new();
    for root_id in selected_roots {
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
    for id in selected_roots {
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

/// Measures operation-local peak RSS by comparing `VmHWM` before and after the operation.
/// Returns the difference (operation-local peak) if measurement is successful.
/// On non-Linux platforms or if measurement fails, returns None.
fn measure_operation_local_peak_rss<F, R>(f: F) -> Result<Option<u64>>
where
    F: FnOnce() -> Result<R>,
{
    #[cfg(target_os = "linux")]
    {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        fn read_vm_hwm_value() -> Result<u64> {
            let status = File::open("/proc/self/status")?;
            let reader = BufReader::new(status);

            for line in reader.lines() {
                let line = line?;
                if let Some(value) = line.strip_prefix("VmHWM:") {
                    let kib = value
                        .split_whitespace()
                        .find_map(|part| part.parse::<u64>().ok())
                        .ok_or_else(|| Error::Import("failed to parse VmHWM value".to_owned()))?;
                    return kib
                        .checked_mul(1024)
                        .ok_or_else(|| Error::Import("VmHWM value overflowed bytes".to_owned()));
                }
            }
            Err(Error::Import(
                "VmHWM was not present in /proc/self/status".to_owned(),
            ))
        }

        let before_hwm = read_vm_hwm_value()?;
        let _result = f()?;
        let after_hwm = read_vm_hwm_value()?;

        // Operation-local peak is the difference between after and before
        // If after >= before, the operation increased the peak by (after - before)
        // If after < before, the operation didn't increase the peak (or peak was from elsewhere)
        let operation_local_peak = if after_hwm >= before_hwm {
            Some(after_hwm - before_hwm)
        } else {
            Some(0) // No operation-local increase
        };

        Ok(operation_local_peak)
    }

    #[cfg(not(target_os = "linux"))]
    {
        // On non-Linux platforms, we can't measure operation-local peak RSS
        let _ = f();
        Ok(None)
    }
}

/// Helper function to get source position packages for benchmarking.
/// For a given index and source position, returns a vector of package references
/// that can be used for scalar reconstruction benchmarks.
fn get_source_position_packages(
    index: &CityIndex,
    position: SourcePosition,
    package_count: usize,
) -> Result<Vec<IndexedPackageRef>> {
    let total_packages = index.package_count()?;

    if total_packages == 0 {
        return Ok(Vec::new());
    }

    match position {
        SourcePosition::First => {
            // Get the first package
            let refs = index.package_ref_page_after_record_id(None, 1)?;
            Ok(refs.into_iter().take(package_count.min(1)).collect())
        }
        SourcePosition::Last => {
            // Get the last package
            if total_packages == 1 {
                let refs = index.package_ref_page_after_record_id(None, 1)?;
                Ok(refs)
            } else {
                // Try to get packages near the end
                let page_size = package_count.min(256);
                let start_offset = total_packages.saturating_sub(page_size);
                let refs =
                    index.package_ref_page_after_record_id(
                        Some(start_offset.try_into().map_err(|_| {
                            Error::Import("record ID conversion failed".to_owned())
                        })?),
                        page_size,
                    )?;
                Ok(refs.into_iter().take(package_count).collect())
            }
        }
        SourcePosition::Middle => {
            // Get a package from the middle
            let middle_offset = total_packages / 2;
            let limit = package_count.min(256);
            let refs = index.package_ref_page_after_record_id(
                Some(
                    middle_offset
                        .try_into()
                        .map_err(|_| Error::Import("record ID conversion failed".to_owned()))?,
                ),
                limit,
            )?;
            Ok(refs.into_iter().take(1).collect())
        }
    }
}

/// Helper function to create a warm index for benchmarking.
/// This creates an index and performs an initial reindex to warm it up.
fn create_warm_index(resolved: &ResolvedDataset, worker_count: usize) -> Result<CityIndex> {
    // Set worker count for this operation
    with_worker_count_env(worker_count, || {
        let mut index = CityIndex::open(resolved.storage_layout(), &resolved.index_path)?;
        index.reindex()?;
        Ok(index)
    })
}

/// Run cold scalar reconstruction benchmarks for different source positions.
/// Uses fresh `CityIndex` for each measurement to simulate cold reads.
fn run_cold_scalar_reconstruction(
    manifest: &BenchmarkManifest,
    worker_count: usize,
    feature_count: usize,
    source_count: usize,
    cityobject_count: usize,
    positions: &[SourcePosition],
) -> Result<Vec<BenchmarkOperationRecord>> {
    let mut runs = Vec::new();

    // Create fresh index for each position measurement
    for &position in positions {
        let index_path = fresh_benchmark_index_path(manifest, worker_count)?;
        let resolved = resolve_dataset(&manifest.prepared_dataset, Some(index_path))?;

        let mut index = CityIndex::open(resolved.storage_layout(), &resolved.index_path)?;
        index.reindex()?;

        // Get packages for this position
        let position_refs = get_source_position_packages(&index, position, 1)?;

        if position_refs.is_empty() {
            continue;
        }

        let sidecar_byte_size =
            fs::metadata(&resolved.index_path).map_or(0, |metadata| metadata.len());

        // Measure timing and memory
        let started = Instant::now();
        for package in &position_refs {
            let _model = index.read_package(package)?;
        }
        let elapsed_ns = u64::try_from(started.elapsed().as_nanos())
            .map_err(|_| Error::Import("benchmark elapsed time does not fit in u64".to_owned()))?;
        let memory = profile::current_memory_snapshot()?;

        // Measure operation-local peak RSS
        let operation_local_peak = measure_operation_local_peak_rss(|| {
            for package in &position_refs {
                let _model = index.read_package(package)?;
            }
            Ok(())
        })?;

        runs.push(build_record(BenchmarkRecordInput {
            dataset_label: manifest.dataset_label.clone(),
            source_artifact: manifest.source_artifact.clone(),
            prepared_dataset: manifest.prepared_dataset.clone(),
            subset_size: manifest.subset_size,
            layout: manifest.layout,
            byte_size: manifest.byte_size,
            sidecar_byte_size,
            worker_count,
            operation: "cold_scalar_reconstruction".to_owned(),
            variant: Some(format!("position-{position:?}")),
            elapsed_ns,
            memory,
            feature_count,
            package_count: feature_count,
            source_count,
            cityobject_count,
            cityobject_relationship_count: manifest.cityobject_relationship_count,
            multi_geometry_cityobject_count: manifest.multi_geometry_cityobject_count,
            query_hit_count: Some(position_refs.len()),
            operation_local_peak_rss_bytes: operation_local_peak,
        }));
    }

    Ok(runs)
}

/// Run warm scalar reconstruction benchmarks for different source positions.
/// Uses a single warmed-up `CityIndex` for all measurements.
fn run_warm_scalar_reconstruction(
    manifest: &BenchmarkManifest,
    worker_count: usize,
    feature_count: usize,
    source_count: usize,
    cityobject_count: usize,
    positions: &[SourcePosition],
) -> Result<Vec<BenchmarkOperationRecord>> {
    let mut runs = Vec::new();

    // Create and warm up the index once
    let index_path = fresh_benchmark_index_path(manifest, worker_count)?;
    let resolved = resolve_dataset(&manifest.prepared_dataset, Some(index_path.clone()))?;
    let index = create_warm_index(&resolved, worker_count)?;

    let sidecar_byte_size = fs::metadata(&index_path).map_or(0, |metadata| metadata.len());

    // Get packages for each position
    let mut position_refs_map = HashMap::new();
    for &position in positions {
        let refs = get_source_position_packages(&index, position, 1)?;
        position_refs_map.insert(position, refs);
    }

    // Measure warm scalar reconstruction for each position
    for &position in positions {
        let position_refs = position_refs_map
            .get(&position)
            .cloned()
            .unwrap_or_default();

        if position_refs.is_empty() {
            continue;
        }

        // Warm up by reading once
        for package in &position_refs {
            let _ = index.read_package(package)?;
        }

        // Measure timing and memory
        let started = Instant::now();
        for package in &position_refs {
            let _model = index.read_package(package)?;
        }
        let elapsed_ns = u64::try_from(started.elapsed().as_nanos())
            .map_err(|_| Error::Import("benchmark elapsed time does not fit in u64".to_owned()))?;
        let memory = profile::current_memory_snapshot()?;

        // Measure operation-local peak RSS
        let operation_local_peak = measure_operation_local_peak_rss(|| {
            for package in &position_refs {
                let _model = index.read_package(package)?;
            }
            Ok(())
        })?;

        runs.push(build_record(BenchmarkRecordInput {
            dataset_label: manifest.dataset_label.clone(),
            source_artifact: manifest.source_artifact.clone(),
            prepared_dataset: manifest.prepared_dataset.clone(),
            subset_size: manifest.subset_size,
            layout: manifest.layout,
            byte_size: manifest.byte_size,
            sidecar_byte_size,
            worker_count,
            operation: "warm_scalar_reconstruction".to_owned(),
            variant: Some(format!("position-{position:?}")),
            elapsed_ns,
            memory,
            feature_count,
            package_count: feature_count,
            source_count,
            cityobject_count,
            cityobject_relationship_count: manifest.cityobject_relationship_count,
            multi_geometry_cityobject_count: manifest.multi_geometry_cityobject_count,
            query_hit_count: Some(position_refs.len()),
            operation_local_peak_rss_bytes: operation_local_peak,
        }));
    }

    Ok(runs)
}

/// Run same-source batch reconstruction benchmarks with different batch sizes.
fn run_same_source_batches(
    index: &CityIndex,
    manifest: &BenchmarkManifest,
    worker_count: usize,
    feature_count: usize,
    source_count: usize,
    cityobject_count: usize,
    batch_sizes: &[usize],
) -> Result<Vec<BenchmarkOperationRecord>> {
    let mut runs = Vec::new();

    let sidecar_byte_size = fs::metadata(
        manifest
            .prepared_dataset
            .join(format!(".cityjson-index.worker-{worker_count}.sqlite")),
    )
    .map_or(0, |metadata| metadata.len());

    // Get package references for benchmarking across all sources
    // Use feature_count to get a representative sample across all sources
    let all_refs = index.package_ref_page_after_record_id(None, feature_count)?;

    for &batch_size in batch_sizes {
        let batch_limit = batch_size.min(all_refs.len());
        let batch_refs: Vec<_> = all_refs.iter().take(batch_limit).cloned().collect();

        if batch_refs.is_empty() {
            continue;
        }

        // Measure timing and memory
        let started = Instant::now();
        let _packages = index.read_packages(&batch_refs)?;
        let elapsed_ns = u64::try_from(started.elapsed().as_nanos())
            .map_err(|_| Error::Import("benchmark elapsed time does not fit in u64".to_owned()))?;
        let memory = profile::current_memory_snapshot()?;

        // Measure operation-local peak RSS
        let operation_local_peak = measure_operation_local_peak_rss(|| {
            let batch_refs_cloned = batch_refs.clone();
            let _packages = index.read_packages(&batch_refs_cloned)?;
            Ok(())
        })?;

        runs.push(build_record(BenchmarkRecordInput {
            dataset_label: manifest.dataset_label.clone(),
            source_artifact: manifest.source_artifact.clone(),
            prepared_dataset: manifest.prepared_dataset.clone(),
            subset_size: manifest.subset_size,
            layout: manifest.layout,
            byte_size: manifest.byte_size,
            sidecar_byte_size,
            worker_count,
            operation: "same_source_batch_reconstruction".to_owned(),
            variant: Some(format!("batch_size-{batch_size},multi_source")),
            elapsed_ns,
            memory,
            feature_count,
            package_count: feature_count,
            source_count,
            cityobject_count,
            cityobject_relationship_count: manifest.cityobject_relationship_count,
            multi_geometry_cityobject_count: manifest.multi_geometry_cityobject_count,
            query_hit_count: Some(batch_refs.len()),
            operation_local_peak_rss_bytes: operation_local_peak,
        }));
    }

    Ok(runs)
}

/// Run duplicate batch reconstruction benchmark with 50% duplicates.
fn run_duplicate_batch_reconstruction(
    index: &CityIndex,
    manifest: &BenchmarkManifest,
    worker_count: usize,
    feature_count: usize,
    source_count: usize,
    cityobject_count: usize,
) -> Result<BenchmarkOperationRecord> {
    let sidecar_byte_size = fs::metadata(
        manifest
            .prepared_dataset
            .join(format!(".cityjson-index.worker-{worker_count}.sqlite")),
    )
    .map_or(0, |metadata| metadata.len());

    // Get package references
    let all_refs = index.package_ref_page_after_record_id(None, feature_count)?;

    // Create a batch with 50% duplicates (256 requests with 128 unique + 128 duplicates)
    let unique_count = 128.min(all_refs.len());
    let unique_refs: Vec<_> = all_refs.iter().take(unique_count).cloned().collect();

    let mut batch_refs = Vec::with_capacity(256);
    // Add unique refs
    batch_refs.extend_from_slice(&unique_refs);
    // Add duplicates (repeat the same refs)
    batch_refs.extend_from_slice(&unique_refs);

    if batch_refs.is_empty() {
        // Fallback to smaller batch if we don't have enough packages
        let small_batch_size = unique_count.min(256);
        let small_refs: Vec<_> = all_refs.iter().take(small_batch_size).cloned().collect();
        batch_refs = small_refs;
    }

    // Measure timing and memory
    let started = Instant::now();
    let _packages = index.read_packages(&batch_refs)?;
    let elapsed_ns = u64::try_from(started.elapsed().as_nanos())
        .map_err(|_| Error::Import("benchmark elapsed time does not fit in u64".to_owned()))?;
    let memory = profile::current_memory_snapshot()?;

    // Measure operation-local peak RSS
    let operation_local_peak = measure_operation_local_peak_rss(|| {
        let batch_refs_cloned = batch_refs.clone();
        let _packages = index.read_packages(&batch_refs_cloned)?;
        Ok(())
    })?;

    Ok(build_record(BenchmarkRecordInput {
        dataset_label: manifest.dataset_label.clone(),
        source_artifact: manifest.source_artifact.clone(),
        prepared_dataset: manifest.prepared_dataset.clone(),
        subset_size: manifest.subset_size,
        layout: manifest.layout,
        byte_size: manifest.byte_size,
        sidecar_byte_size,
        worker_count,
        operation: "duplicate_batch_reconstruction".to_owned(),
        variant: Some("50_percent_duplicates".to_owned()),
        elapsed_ns,
        memory,
        feature_count,
        package_count: feature_count,
        source_count,
        cityobject_count,
        cityobject_relationship_count: manifest.cityobject_relationship_count,
        multi_geometry_cityobject_count: manifest.multi_geometry_cityobject_count,
        query_hit_count: Some(batch_refs.len()),
        operation_local_peak_rss_bytes: operation_local_peak,
    }))
}

/// Run bbox queries with materialization benchmarks for different window sizes.
fn run_bbox_queries_with_materialization(
    index: &CityIndex,
    manifest: &BenchmarkManifest,
    worker_count: usize,
    feature_count: usize,
    source_count: usize,
    cityobject_count: usize,
) -> Result<Vec<BenchmarkOperationRecord>> {
    let mut runs = Vec::new();

    let sidecar_byte_size = fs::metadata(
        manifest
            .prepared_dataset
            .join(format!(".cityjson-index.worker-{worker_count}.sqlite")),
    )
    .map_or(0, |metadata| metadata.len());

    // Define different bbox window sizes: small, medium, large, full
    let window_labels = ["small", "medium", "large", "full"];

    // Create fallback windows if needed
    let fallback_windows: Vec<QueryWindow> = window_labels
        .iter()
        .enumerate()
        .map(|(i, _)| QueryWindow {
            label: format!("window-{}", window_labels[i]),
            bbox: manifest.dataset_bbox,
        })
        .collect();

    for i in 0..window_labels.len() {
        let label = window_labels[i];
        let window = if i < manifest.query_windows.len() {
            &manifest.query_windows[i]
        } else {
            &fallback_windows[i]
        };

        // Query packages for this bbox
        let started = Instant::now();
        let package_refs = index.query_package_refs(&window.bbox)?;

        // Materialize the packages
        let packages = index.read_packages(&package_refs)?;
        let elapsed_ns = u64::try_from(started.elapsed().as_nanos())
            .map_err(|_| Error::Import("benchmark elapsed time does not fit in u64".to_owned()))?;
        let memory = profile::current_memory_snapshot()?;

        runs.push(build_record(BenchmarkRecordInput {
            dataset_label: manifest.dataset_label.clone(),
            source_artifact: manifest.source_artifact.clone(),
            prepared_dataset: manifest.prepared_dataset.clone(),
            subset_size: manifest.subset_size,
            layout: manifest.layout,
            byte_size: manifest.byte_size,
            sidecar_byte_size,
            worker_count,
            operation: "bbox_query_with_materialization".to_owned(),
            variant: Some(format!("window-{label}")),
            elapsed_ns,
            memory,
            feature_count,
            package_count: feature_count,
            source_count,
            cityobject_count,
            cityobject_relationship_count: manifest.cityobject_relationship_count,
            multi_geometry_cityobject_count: manifest.multi_geometry_cityobject_count,
            query_hit_count: Some(packages.len()),
            operation_local_peak_rss_bytes: None,
        }));
    }

    Ok(runs)
}

/// Run concurrent readers benchmark with independent `CityIndex` instances.
fn run_concurrent_readers(
    index: &CityIndex,
    manifest: &BenchmarkManifest,
    worker_count: usize,
    feature_count: usize,
    source_count: usize,
    cityobject_count: usize,
    reader_counts: &[usize],
) -> Result<Vec<BenchmarkOperationRecord>> {
    use rayon::prelude::*;

    let mut runs = Vec::new();

    for &reader_count in reader_counts {
        let sidecar_byte_size = fs::metadata(
            manifest
                .prepared_dataset
                .join(format!(".cityjson-index.worker-{worker_count}.sqlite")),
        )
        .map_or(0, |metadata| metadata.len());

        // Get some package references to read
        let all_refs = index.package_ref_page_after_record_id(None, feature_count.min(256))?;

        let refs: Vec<_> = all_refs
            .iter()
            .take(reader_count.min(all_refs.len()))
            .collect();

        if refs.is_empty() {
            continue;
        }

        // Get the index path - all threads will open the same database read-only
        let index_path = manifest
            .prepared_dataset
            .join(format!(".cityjson-index.worker-{worker_count}.sqlite"));

        // Measure timing and memory
        let started = Instant::now();

        // Shared index with thread-local cache simulation
        // This matches tyler's pattern where each thread has its own cached index
        let index_path_clone = index_path.clone();
        let chunk_size = refs.len() / reader_count.max(1);
        let _results: Vec<_> = refs
            .chunks(chunk_size.max(1))
            .par_bridge()
            .map(|chunk| {
                // Each thread opens the same index database (read-only, no reindex)
                // This simulates tyler's thread-local caching pattern
                let resolved =
                    resolve_dataset(&manifest.prepared_dataset, Some(index_path_clone.clone()))
                        .unwrap();
                let thread_index =
                    CityIndex::open(resolved.storage_layout(), &resolved.index_path).unwrap();

                for package in chunk {
                    let _ = thread_index.read_package(package).unwrap();
                }
                Ok::<(), Error>(())
            })
            .collect::<Result<Vec<_>>>()?;

        let elapsed_ns = u64::try_from(started.elapsed().as_nanos())
            .map_err(|_| Error::Import("benchmark elapsed time does not fit in u64".to_owned()))?;
        let memory = profile::current_memory_snapshot()?;

        // Measure operation-local peak RSS
        let operation_local_peak = measure_operation_local_peak_rss(|| {
            let index_path_clone = index_path.clone();
            let chunk_size = refs.len() / reader_count.max(1);
            let _results: Vec<_> = refs
                .chunks(chunk_size.max(1))
                .par_bridge()
                .map(|chunk| {
                    // Each thread opens the same index database (read-only, no reindex)
                    // This simulates tyler's thread-local caching pattern
                    let resolved =
                        resolve_dataset(&manifest.prepared_dataset, Some(index_path_clone.clone()))
                            .unwrap();
                    let thread_index =
                        CityIndex::open(resolved.storage_layout(), &resolved.index_path).unwrap();

                    for package in chunk {
                        let _ = thread_index.read_package(package).unwrap();
                    }
                    Ok::<(), Error>(())
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(())
        })?;

        runs.push(build_record(BenchmarkRecordInput {
            dataset_label: manifest.dataset_label.clone(),
            source_artifact: manifest.source_artifact.clone(),
            prepared_dataset: manifest.prepared_dataset.clone(),
            subset_size: manifest.subset_size,
            layout: manifest.layout,
            byte_size: manifest.byte_size,
            sidecar_byte_size,
            worker_count,
            operation: "concurrent_readers".to_owned(),
            variant: Some(format!("reader_count-{reader_count}")),
            elapsed_ns,
            memory,
            feature_count,
            package_count: feature_count,
            source_count,
            cityobject_count,
            cityobject_relationship_count: manifest.cityobject_relationship_count,
            multi_geometry_cityobject_count: manifest.multi_geometry_cityobject_count,
            query_hit_count: Some(refs.len() * reader_count),
            operation_local_peak_rss_bytes: operation_local_peak,
        }));
    }

    Ok(runs)
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use serde_json::json;

    use super::*;

    #[test]
    fn single_tile_preparation_materializes_every_benchmark_layout() -> Result<()> {
        let root = temp_dir("benchmark-layouts");
        let artifact = root.join("basisvoorziening.city.json");
        fs::write(
            &artifact,
            serde_json::to_vec_pretty(&synthetic_cityjson_document(3))
                .map_err(|error| Error::Import(error.to_string()))?,
        )?;
        let cli = BenchmarkCli {
            json: false,
            corpus_root: root.clone(),
            work_root: root.join("work"),
            artifact: Some(artifact.clone()),
            case: Vec::new(),
            layout: Vec::new(),
            workers: vec![1],
            multi_tile_root: None,
            groningen_corpus: None,
            tyler_tile_count: DEFAULT_TYLER_TILE_COUNT,
            warmth: Vec::new(),
            source_position: Vec::new(),
            batch_size: Vec::new(),
            concurrent_readers: Vec::new(),
            prepare_only: false,
            profile_target: None,
            reuse_prepared: false,
            profile_events: None,
        };

        for layout in BenchmarkLayoutKind::ALL {
            let prepared =
                prepare_case(&cli, BenchmarkCaseKind::SingleTileFull, layout, &artifact)?;
            assert_eq!(prepared.len(), 1);
            let manifest = &prepared[0].manifest;
            assert_eq!(manifest.layout, layout);
            assert_eq!(manifest.feature_count, 3);
            assert_eq!(manifest.source_count, 1);
            assert!(manifest.byte_size > 0);
            assert!(manifest.dataset_label.ends_with(layout.as_label()));

            let resolved = resolve_dataset(&manifest.prepared_dataset, None)?;
            assert_eq!(resolved.source_paths().len(), 1);

            let index_path = fresh_benchmark_index_path(manifest, 1)?;
            let resolved = resolve_dataset(&manifest.prepared_dataset, Some(index_path))?;
            let mut index = CityIndex::open(resolved.storage_layout(), &resolved.index_path)?;
            index.reindex()?;
            assert_eq!(index.package_count()?, manifest.feature_count);
            assert_eq!(index.cityobject_count()?, manifest.cityobject_count);
        }

        Ok(())
    }

    #[test]
    fn single_tile_cityjson_preparation_preserves_unmodified_artifact_bytes() -> Result<()> {
        // Input: a minified CityJSON artifact prepared as the full single-tile CityJSON layout.
        // Assertions: the prepared dataset file is byte-for-byte identical to the source artifact
        // and the manifest records that exact prepared size.
        let root = temp_dir("benchmark-cityjson-raw-bytes");
        let artifact = root.join("basisvoorziening.city.json");
        let artifact_bytes = serde_json::to_vec(&synthetic_cityjson_document(3))
            .map_err(|error| Error::Import(error.to_string()))?;
        fs::write(&artifact, &artifact_bytes)?;
        let cli = BenchmarkCli {
            json: false,
            corpus_root: root.clone(),
            work_root: root.join("work"),
            artifact: Some(artifact.clone()),
            case: Vec::new(),
            layout: vec![BenchmarkLayoutKind::CityJson],
            workers: vec![1],
            multi_tile_root: None,
            groningen_corpus: None,
            tyler_tile_count: DEFAULT_TYLER_TILE_COUNT,
            warmth: Vec::new(),
            source_position: Vec::new(),
            batch_size: Vec::new(),
            concurrent_readers: Vec::new(),
            prepare_only: false,
            profile_target: None,
            reuse_prepared: false,
            profile_events: None,
        };

        let prepared = prepare_case(
            &cli,
            BenchmarkCaseKind::SingleTileFull,
            BenchmarkLayoutKind::CityJson,
            &artifact,
        )?;
        let manifest = &prepared[0].manifest;
        let prepared_bytes = fs::read(manifest.prepared_dataset.join("dataset.city.json"))?;

        assert_eq!(prepared_bytes, artifact_bytes);
        assert_eq!(
            manifest.byte_size,
            u64::try_from(artifact_bytes.len())
                .map_err(|_| Error::Import("test artifact size overflowed u64".to_owned()))?
        );

        Ok(())
    }

    #[test]
    fn subset_cityjson_preparation_writes_compact_valid_json() -> Result<()> {
        // Input: a pretty-printed CityJSON artifact prepared as a two-package CityJSON subset.
        // Assertions: the transformed prepared file is valid CityJSON JSON, contains the requested
        // package count, and is serialized compactly without pretty-print newlines.
        let root = temp_dir("benchmark-cityjson-compact-subset");
        let artifact = root.join("basisvoorziening.city.json");
        fs::write(
            &artifact,
            serde_json::to_vec_pretty(&synthetic_cityjson_document(4))
                .map_err(|error| Error::Import(error.to_string()))?,
        )?;
        let cli = BenchmarkCli {
            json: false,
            corpus_root: root.clone(),
            work_root: root.join("work"),
            artifact: Some(artifact.clone()),
            case: Vec::new(),
            layout: vec![BenchmarkLayoutKind::CityJson],
            workers: vec![1],
            multi_tile_root: None,
            groningen_corpus: None,
            tyler_tile_count: DEFAULT_TYLER_TILE_COUNT,
            warmth: Vec::new(),
            source_position: Vec::new(),
            batch_size: Vec::new(),
            concurrent_readers: Vec::new(),
            prepare_only: false,
            profile_target: None,
            reuse_prepared: false,
            profile_events: None,
        };

        let prepared = prepare_single_tile_dataset(
            &cli,
            "single-tile-subset-2",
            BenchmarkLayoutKind::CityJson,
            &artifact,
            Some(2),
        )?;
        let prepared_bytes =
            fs::read(prepared.manifest.prepared_dataset.join("dataset.city.json"))?;
        let prepared_document: Value = serde_json::from_slice(&prepared_bytes)
            .map_err(|error| Error::Import(error.to_string()))?;

        assert_eq!(extract_root_ids(&prepared_document)?.len(), 2);
        assert!(
            !prepared_bytes.contains(&b'\n'),
            "transformed CityJSON benchmark fixtures should not be pretty-printed"
        );

        Ok(())
    }

    #[test]
    fn multi_source_preparation_creates_parallel_source_shards() -> Result<()> {
        let root = temp_dir("benchmark-multi-source");
        let artifact = root.join("basisvoorziening.city.json");
        fs::write(
            &artifact,
            serde_json::to_vec_pretty(&synthetic_cityjson_document(8))
                .map_err(|error| Error::Import(error.to_string()))?,
        )?;
        let cli = BenchmarkCli {
            json: false,
            corpus_root: root.clone(),
            work_root: root.join("work"),
            artifact: Some(artifact.clone()),
            case: Vec::new(),
            layout: vec![BenchmarkLayoutKind::CityJson],
            workers: vec![4],
            multi_tile_root: None,
            groningen_corpus: None,
            tyler_tile_count: DEFAULT_TYLER_TILE_COUNT,
            warmth: Vec::new(),
            source_position: Vec::new(),
            batch_size: Vec::new(),
            concurrent_readers: Vec::new(),
            prepare_only: false,
            profile_target: None,
            reuse_prepared: false,
            profile_events: None,
        };

        let prepared = prepare_case(
            &cli,
            BenchmarkCaseKind::MultiSource,
            BenchmarkLayoutKind::CityJson,
            &artifact,
        )?;
        assert_eq!(prepared.len(), 1);
        let manifest = &prepared[0].manifest;
        assert!(
            manifest.source_count > 1,
            "multi-source preparation should create more than one source file"
        );

        let resolved = resolve_dataset(&manifest.prepared_dataset, None)?;
        assert!(
            resolved.source_paths().len() > 1,
            "resolved prepared dataset should expose multiple source shards"
        );
        for source_path in resolved.source_paths() {
            let shard_bytes = fs::read(source_path)?;
            let shard_document: Value = serde_json::from_slice(&shard_bytes)
                .map_err(|error| Error::Import(error.to_string()))?;
            assert!(
                !shard_bytes.contains(&b'\n'),
                "derived multi-source CityJSON shards should be compact JSON"
            );
            assert!(
                !extract_root_ids(&shard_document)?.is_empty(),
                "each benchmark shard should contain at least one package"
            );
        }
        assert!(
            resolved.source_paths().len().min(4) > 1,
            "a worker count greater than one should be able to reach multiple shards"
        );

        with_worker_count_env(4, || {
            let index_path = fresh_benchmark_index_path(manifest, 4)?;
            let resolved = resolve_dataset(&manifest.prepared_dataset, Some(index_path))?;
            let mut index = CityIndex::open(resolved.storage_layout(), &resolved.index_path)?;
            index.reindex()?;
            assert_eq!(
                index.source_count()?,
                manifest.source_count,
                "indexed source count should match prepared shards"
            );
            Ok(())
        })?;

        Ok(())
    }

    #[test]
    #[allow(
        clippy::too_many_lines,
        reason = "the lifecycle regression verifies reuse, telemetry, mismatch, and missing-sidecar failures"
    )]
    fn isolated_profile_reuses_a_matching_prepared_sidecar() -> Result<()> {
        let root = temp_dir("benchmark-profile-reuse");
        let corpus = root.join("groningen");
        fs::create_dir_all(&corpus)?;
        fs::write(
            corpus.join("tile.city.json"),
            serde_json::to_vec(&synthetic_cityjson_document(3))
                .map_err(|error| Error::Import(error.to_string()))?,
        )?;
        let mut cli = BenchmarkCli {
            json: false,
            corpus_root: root.clone(),
            work_root: root.join("work"),
            artifact: None,
            case: vec![BenchmarkCaseKind::TylerPipeline],
            layout: vec![BenchmarkLayoutKind::CityJson],
            workers: vec![1],
            multi_tile_root: None,
            groningen_corpus: Some(corpus),
            tyler_tile_count: 1,
            warmth: Vec::new(),
            source_position: Vec::new(),
            batch_size: Vec::new(),
            concurrent_readers: Vec::new(),
            prepare_only: false,
            profile_target: Some(BenchmarkProfileTarget::TylerFeatureMaterialization),
            reuse_prepared: true,
            profile_events: Some(root.join("stage-events.jsonl")),
        };
        let prepared = prepare_tyler_dataset(
            &cli,
            BenchmarkLayoutKind::CityJson,
            cli.groningen_corpus
                .as_deref()
                .expect("test Groningen corpus should be configured"),
            1,
        )?;
        prepare_benchmark_sidecar(&prepared, 1)?;
        let sidecar = benchmark_index_path(&prepared.manifest, 1);

        let reused = prepare_tyler_dataset(
            &cli,
            BenchmarkLayoutKind::CityJson,
            cli.groningen_corpus
                .as_deref()
                .expect("test Groningen corpus should be configured"),
            1,
        )?;
        assert_eq!(reused.manifest.source_count, 1);
        assert!(
            sidecar.exists(),
            "reusing a matching prepared dataset must preserve worker sidecars"
        );

        let report = run(&cli)?;

        assert!(
            sidecar.exists(),
            "profile must preserve the prepared sidecar"
        );
        assert_eq!(report.runs.len(), 1);
        assert_eq!(report.runs[0].feature_count, 3);
        assert_eq!(report.runs[0].package_count, 3);
        assert_eq!(report.runs[0].source_count, 1);

        let events_path = cli
            .profile_events
            .as_deref()
            .expect("profile event path should be configured");
        let events = fs::read_to_string(events_path)?
            .lines()
            .map(|line| {
                serde_json::from_str::<Value>(line)
                    .map_err(|error| Error::Import(error.to_string()))
            })
            .collect::<Result<Vec<_>>>()?;
        let before_drop = events
            .iter()
            .find(|event| event["event"] == "cache_before_drop")
            .expect("profile should record the populated worker caches");
        assert_eq!(before_drop["schema_version"], 3);
        assert!(before_drop["timestamp_ns"].as_u64().unwrap_or(0) > 0);
        assert_eq!(before_drop["cached_source_count"], 1);
        assert!(before_drop["cached_vertex_count"].as_u64().unwrap_or(0) > 0);
        assert!(before_drop["vertex_capacity_bytes"].as_u64().unwrap_or(0) > 0);
        assert_eq!(before_drop["workers"].as_array().map(Vec::len), Some(1));
        let after_drop = events
            .iter()
            .find(|event| event["event"] == "cache_after_drop")
            .expect("profile should record cleared worker caches");
        assert_eq!(after_drop["cached_source_count"], 0);
        assert_eq!(after_drop["cached_vertex_count"], 0);
        assert_eq!(after_drop["vertex_capacity_bytes"], 0);

        cli.tyler_tile_count = 2;
        let mismatch = run(&cli).expect_err("mismatched preparation should fail");
        assert!(
            mismatch
                .to_string()
                .contains("contains 1 sources, expected 2 tiles"),
            "unexpected mismatch error: {mismatch}"
        );

        cli.tyler_tile_count = 1;
        remove_file_if_exists(&sidecar)?;
        let missing = run(&cli).expect_err("missing prepared sidecar should fail");
        assert!(
            missing.to_string().contains("does not exist"),
            "unexpected missing-sidecar error: {missing}"
        );

        Ok(())
    }

    fn temp_dir(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after the unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("cityjson-index-{label}-{unique}.dir"));
        fs::create_dir_all(&path).expect("temp benchmark directory should be creatable");
        path
    }

    fn synthetic_cityjson_document(feature_count: usize) -> Value {
        let mut cityobjects = serde_json::Map::new();
        let mut vertices = Vec::with_capacity(feature_count * 3);
        for index in 0..feature_count {
            let base = index * 3;
            cityobjects.insert(
                format!("feature-{index:02}"),
                json!({
                    "type": "Building",
                    "geometry": [{
                        "type": "MultiSurface",
                        "lod": "1.0",
                        "boundaries": [[[base, base + 1, base + 2]]]
                    }]
                }),
            );
            let x = i64::try_from(index).expect("feature index should fit in i64") * 100;
            vertices.push(json!([x, 0, 0]));
            vertices.push(json!([x + 10, 0, 0]));
            vertices.push(json!([x, 10, 0]));
        }

        json!({
            "type": "CityJSON",
            "version": "2.0",
            "transform": {
                "scale": [1.0, 1.0, 1.0],
                "translate": [0.0, 0.0, 0.0]
            },
            "metadata": {
                "referenceSystem": "https://www.opengis.net/def/crs/EPSG/0/4979",
                "title": "benchmark test fixture"
            },
            "CityObjects": cityobjects,
            "vertices": vertices
        })
    }
}
