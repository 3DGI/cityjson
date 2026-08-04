//! CLI for constructing and measuring an ADR 012 vertex-store candidate.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex, mpsc};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use cityjson_index::profile::{MemorySnapshot, current_memory_snapshot};
use cityjson_index::vertex_store_bakeoff::{
    BakeoffProvenance, BakeoffResult, READ_BATCH_SIZE, SAMPLE_SIZE, VertexStore,
    VertexStoreStrategy, VertexStoreTelemetry, candidate, open_matching_read_sidecar, write_result,
};
use cityjson_index::{
    CityIndex, IndexedPackageRef, PackageType, PackageVertexCoverage, ResolvedDataset,
    StorageLayout, resolve_dataset,
};
use cityjson_lib::{Error, Result};
use clap::{Args, Parser, Subcommand};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const SAMPLE_SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Parser)]
#[command(
    name = "vertex-store-bakeoff",
    about = "Build and measure an ADR 012 candidate"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Build, validate, and atomically retain a candidate sidecar.
    Build(BuildArgs),
    /// Write the versioned package sample shared by every candidate.
    Sample(SampleArgs),
    /// Materialize the sample and record its digest and sidecar size.
    CorrectnessStorage(MeasuredArgs),
    /// Measure singleton and 2,048-package batch reads twice.
    ReadLatency(MeasuredArgs),
    /// Materialize every indexed package.
    TylerMaterialization(MeasuredArgs),
}

#[derive(Debug, Args)]
struct DatasetArgs {
    #[arg(long)]
    dataset_root: PathBuf,
    /// Explicit candidate sidecar; implicit/default paths are forbidden.
    #[arg(long)]
    sidecar: PathBuf,
}

#[derive(Debug, Args)]
struct BuildArgs {
    #[command(flatten)]
    dataset: DatasetArgs,
}

#[derive(Debug, Args)]
struct SampleArgs {
    #[command(flatten)]
    dataset: DatasetArgs,
    #[arg(long)]
    output: PathBuf,
    #[arg(long)]
    corpus_identity: String,
    #[arg(long, default_value_t = SAMPLE_SIZE)]
    limit: usize,
}

#[derive(Debug, Args)]
struct MeasuredArgs {
    #[command(flatten)]
    dataset: DatasetArgs,
    #[arg(long)]
    sample: Option<PathBuf>,
    #[arg(long)]
    result: PathBuf,
    /// Optional final VmRSS/VmHWM snapshot written atomically.
    #[arg(long)]
    profile_output: Option<PathBuf>,
    #[arg(long)]
    candidate_commit: String,
    #[arg(long)]
    harness_commit: String,
    #[arg(long)]
    corpus_identity: String,
    #[arg(long, default_value_t = 1)]
    workers: usize,
    #[arg(long, default_value_t = 1)]
    repetition: usize,
    #[arg(long = "runtime", value_parser = parse_key_value)]
    runtime_configuration: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
struct StablePackageIdentity {
    source_path: String,
    model_id: String,
    package_path: String,
    offset: Option<i64>,
    length: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SampleFile {
    schema_version: u32,
    corpus_identity: String,
    sample_identity: String,
    packages: Vec<StablePackageIdentity>,
}

#[derive(Debug, Serialize)]
struct MaterializationResult {
    package_count: usize,
    model_digest: String,
    elapsed_ns: u128,
}

#[derive(Debug, Serialize)]
struct CorrectnessStorageResult {
    sample_identity: String,
    materialization: MaterializationResult,
    sidecar_bytes: u64,
    sqlite_page_count: i64,
    sqlite_page_size: i64,
}

#[derive(Debug, Serialize)]
struct ReadLatencyResult {
    sample_identity: String,
    singleton_first: MaterializationResult,
    singleton_repeat: MaterializationResult,
    batch_first: MaterializationResult,
    batch_repeat: MaterializationResult,
}

#[derive(Debug, Serialize)]
struct TylerResult {
    package_count: usize,
    configured_workers: usize,
    model_digest: String,
    elapsed_ns: u128,
    total_pipeline_elapsed_ns: u128,
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Build(args) => build(&args),
        Command::Sample(args) => sample(&args),
        Command::CorrectnessStorage(args) => correctness_storage(&args),
        Command::ReadLatency(args) => read_latency(&args),
        Command::TylerMaterialization(args) => tyler_materialization(&args),
    }
}

fn active_strategy() -> Result<VertexStoreStrategy> {
    candidate::ACTIVE_STRATEGY.ok_or_else(|| {
        import_error("no candidate is active on this branch; select a candidate branch")
    })
}

fn resolved(args: &DatasetArgs) -> Result<ResolvedDataset> {
    if !args.sidecar.is_absolute() {
        return Err(import_error("--sidecar must be an absolute path"));
    }
    resolve_dataset(&args.dataset_root, Some(args.sidecar.clone()))
}

fn build(args: &BuildArgs) -> Result<()> {
    let strategy = active_strategy()?;
    let final_path = &args.dataset.sidecar;
    let resolved = resolved(&args.dataset)?;
    let temporary_path = temporary_sibling(final_path, "build")?;
    let result: Result<u64> = (|| {
        let mut store = candidate::create(temporary_path.clone())?;
        ensure_strategy(store.as_ref(), strategy)?;
        let mut index = CityIndex::open(resolved.storage_layout(), &temporary_path)?;
        index.reindex_with_vertex_store(store.as_mut())?;
        drop(index);

        let read_store = candidate::create(temporary_path.clone())?;
        let checked = CityIndex::open_vertex_store_read_only(
            resolved.storage_layout(),
            &temporary_path,
            read_store.as_ref(),
        )?;
        drop(checked);
        replace_sidecar(&temporary_path, final_path)?;
        Ok(fs::metadata(final_path)?.len())
    })();
    if result.is_err() && temporary_path.is_file() {
        fs::remove_file(&temporary_path)?;
    }
    let bytes = result?;
    println!(
        "built {} sidecar {} ({bytes} bytes)",
        strategy.identifier(),
        final_path.display()
    );
    Ok(())
}

fn sample(args: &SampleArgs) -> Result<()> {
    reject_unknown("corpus identity", &args.corpus_identity)?;
    if args.limit == 0 {
        return Err(import_error("--limit must be greater than zero"));
    }
    let strategy = active_strategy()?;
    let resolved = resolved(&args.dataset)?;
    let store = candidate::create(args.dataset.sidecar.clone())?;
    ensure_strategy(store.as_ref(), strategy)?;
    let index = CityIndex::open_vertex_store_read_only(
        resolved.storage_layout(),
        &args.dataset.sidecar,
        store.as_ref(),
    )?;
    let connection = open_matching_read_sidecar(&args.dataset.sidecar, strategy)?;
    let packages = stable_prioritized_sample(&index, &connection, args.limit)?;
    if packages.is_empty() {
        return Err(import_error(
            "the sidecar contains no regular CityJSON packages",
        ));
    }
    let sample = SampleFile {
        schema_version: SAMPLE_SCHEMA_VERSION,
        corpus_identity: args.corpus_identity.clone(),
        sample_identity: sample_identity(&args.corpus_identity, &packages)?,
        packages,
    };
    write_json_atomically(&args.output, &sample)
}

fn correctness_storage(args: &MeasuredArgs) -> Result<()> {
    let context = MeasurementContext::open(args, true)?;
    let sample = context.sample.as_ref().expect("sample was required");
    let connection = open_matching_read_sidecar(&args.dataset.sidecar, context.strategy)?;
    let refs = refs_for_sample(&context.index, &connection, &sample.packages)?;
    let started = Instant::now();
    let (packages, telemetry) = context
        .index
        .read_packages_with_vertex_store(&refs, context.store.as_ref())?;
    let materialization = summarize_packages(&packages, started.elapsed().as_nanos())?;
    let payload = CorrectnessStorageResult {
        sample_identity: sample.sample_identity.clone(),
        materialization,
        sidecar_bytes: fs::metadata(&args.dataset.sidecar)?.len(),
        sqlite_page_count: pragma_i64(&connection, "page_count")?,
        sqlite_page_size: pragma_i64(&connection, "page_size")?,
    };
    context.write("correctness-storage", telemetry, payload)
}

fn read_latency(args: &MeasuredArgs) -> Result<()> {
    if args.workers != 1 {
        return Err(import_error("read-latency requires --workers 1"));
    }
    let context = MeasurementContext::open(args, true)?;
    let sample = context.sample.as_ref().expect("sample was required");
    let connection = open_matching_read_sidecar(&args.dataset.sidecar, context.strategy)?;
    let refs = refs_for_sample(&context.index, &connection, &sample.packages)?;
    let (singleton_first, first_single_telemetry) =
        materialize_singletons(&context.index, context.store.as_ref(), &refs)?;
    let (singleton_repeat, repeat_single_telemetry) =
        materialize_singletons(&context.index, context.store.as_ref(), &refs)?;
    let (batch_first, first_batch_telemetry) =
        materialize_batches(&context.index, context.store.as_ref(), &refs)?;
    let (batch_repeat, repeat_batch_telemetry) =
        materialize_batches(&context.index, context.store.as_ref(), &refs)?;
    let telemetry = sum_telemetry([
        first_single_telemetry,
        repeat_single_telemetry,
        first_batch_telemetry,
        repeat_batch_telemetry,
    ]);
    let payload = ReadLatencyResult {
        sample_identity: sample.sample_identity.clone(),
        singleton_first,
        singleton_repeat,
        batch_first,
        batch_repeat,
    };
    context.write("read-latency", telemetry, payload)
}

fn tyler_materialization(args: &MeasuredArgs) -> Result<()> {
    if args.sample.is_some() {
        return Err(import_error(
            "tyler-materialization reads the complete index and does not accept --sample",
        ));
    }
    validate_tyler_workers(args.workers)?;
    let pipeline_started = Instant::now();
    let context = MeasurementContext::open(args, false)?;
    let connection = open_matching_read_sidecar(&args.dataset.sidecar, context.strategy)?;
    let refs = all_stable_package_refs(&context.index, &connection)?;
    let resolved = resolved(&args.dataset)?;
    let (summary, telemetry) = materialize_batches_parallel(
        &resolved,
        &args.dataset.sidecar,
        context.strategy,
        &refs,
        args.workers,
    )?;
    if let Some(path) = &args.profile_output {
        let snapshot = current_memory_snapshot()?;
        write_memory_snapshot(path, &snapshot)?;
    }
    let payload = TylerResult {
        package_count: summary.package_count,
        configured_workers: args.workers,
        model_digest: summary.model_digest,
        elapsed_ns: summary.elapsed_ns,
        total_pipeline_elapsed_ns: pipeline_started.elapsed().as_nanos(),
    };
    context.write("tyler-materialization", telemetry, payload)
}

struct MeasurementContext<'a> {
    args: &'a MeasuredArgs,
    strategy: VertexStoreStrategy,
    index: CityIndex,
    store: Box<dyn VertexStore>,
    sample: Option<SampleFile>,
}

impl<'a> MeasurementContext<'a> {
    fn open(args: &'a MeasuredArgs, sample_required: bool) -> Result<Self> {
        validate_provenance(args)?;
        let strategy = active_strategy()?;
        let resolved = resolved(&args.dataset)?;
        let store = candidate::create(args.dataset.sidecar.clone())?;
        ensure_strategy(store.as_ref(), strategy)?;
        let index = CityIndex::open_vertex_store_read_only(
            resolved.storage_layout(),
            &args.dataset.sidecar,
            store.as_ref(),
        )?;
        let sample = match &args.sample {
            Some(path) => Some(read_sample(path, &args.corpus_identity)?),
            None if sample_required => return Err(import_error("--sample is required")),
            None => None,
        };
        Ok(Self {
            args,
            strategy,
            index,
            store,
            sample,
        })
    }

    fn write<T: Serialize>(
        &self,
        experiment: &str,
        telemetry: VertexStoreTelemetry,
        payload: T,
    ) -> Result<()> {
        let runtime_configuration = self
            .args
            .runtime_configuration
            .iter()
            .cloned()
            .collect::<BTreeMap<_, _>>();
        if runtime_configuration.len() != self.args.runtime_configuration.len() {
            return Err(import_error("duplicate --runtime keys are not allowed"));
        }
        let provenance = BakeoffProvenance {
            strategy: self.strategy,
            candidate_commit: self.args.candidate_commit.clone(),
            harness_commit: self.args.harness_commit.clone(),
            corpus_identity: self.args.corpus_identity.clone(),
            sidecar_path: self.args.dataset.sidecar.clone(),
            worker_count: self.args.workers,
            repetition: self.args.repetition,
            runtime_configuration,
        };
        write_result(
            &self.args.result,
            &BakeoffResult::new(experiment, provenance, telemetry, payload),
        )
    }
}

fn ensure_strategy(store: &dyn VertexStore, expected: VertexStoreStrategy) -> Result<()> {
    if store.strategy() != expected {
        return Err(import_error(
            "candidate factory returned the wrong strategy",
        ));
    }
    Ok(())
}

fn validate_tyler_workers(workers: usize) -> Result<()> {
    if !matches!(workers, 1 | 4 | 24) {
        return Err(import_error(
            "tyler-materialization requires --workers 1, 4, or 24",
        ));
    }
    Ok(())
}

fn validate_provenance(args: &MeasuredArgs) -> Result<()> {
    reject_unknown("candidate commit", &args.candidate_commit)?;
    reject_unknown("harness commit", &args.harness_commit)?;
    reject_unknown("corpus identity", &args.corpus_identity)?;
    if args.workers == 0 || args.repetition == 0 {
        return Err(import_error(
            "--workers and --repetition must be greater than zero",
        ));
    }
    Ok(())
}

fn reject_unknown(label: &str, value: &str) -> Result<()> {
    let value = value.trim();
    if value.is_empty() || value.eq_ignore_ascii_case("unknown") {
        return Err(import_error(format!(
            "{label} must be explicit and non-unknown"
        )));
    }
    Ok(())
}

fn stable_package_rows(connection: &Connection) -> Result<Vec<(StablePackageIdentity, i64)>> {
    let mut statement = connection
        .prepare(
            "SELECT s.path, p.model_id, p.path, p.offset, p.length, p.id \
             FROM packages AS p JOIN sources AS s ON s.id = p.source_id \
             WHERE p.package_type = 'cityjson' \
             ORDER BY s.path, p.model_id, p.path, p.offset, p.length, p.id",
        )
        .map_err(|error| sql_error(&error))?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                StablePackageIdentity {
                    source_path: row.get(0)?,
                    model_id: row.get(1)?,
                    package_path: row.get(2)?,
                    offset: row.get(3)?,
                    length: row.get(4)?,
                },
                row.get(5)?,
            ))
        })
        .map_err(|error| sql_error(&error))?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|error| sql_error(&error))
}

fn all_stable_package_identities(connection: &Connection) -> Result<Vec<StablePackageIdentity>> {
    let mut identities = stable_package_rows(connection)?
        .into_iter()
        .map(|(identity, _)| identity)
        .collect::<Vec<_>>();
    identities.sort();
    if let Some(pair) = identities.windows(2).find(|pair| pair[0] == pair[1]) {
        return Err(import_error(format!(
            "stable package identity is ambiguous: {} / {}",
            pair[0].source_path, pair[0].model_id
        )));
    }
    Ok(identities)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SamplingCandidate {
    identity: StablePackageIdentity,
    source_size: u64,
    source_vertex_count: u64,
    coverage: PackageVertexCoverage,
}

fn source_sampling_state(connection: &Connection) -> Result<BTreeMap<i64, (String, u64, u64)>> {
    let mut statement = connection
        .prepare(
            "SELECT s.id, s.path, s.source_size, v.vertex_count \
             FROM sources AS s JOIN vertex_store_source_state AS v ON v.source_id = s.id \
             ORDER BY s.path",
        )
        .map_err(|error| sql_error(&error))?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })
        .map_err(|error| sql_error(&error))?;
    let mut states = BTreeMap::new();
    for row in rows {
        let (source_id, path, source_size, vertex_count) =
            row.map_err(|error| sql_error(&error))?;
        let source_size = u64::try_from(source_size)
            .map_err(|_| import_error(format!("source {source_id} has a negative size")))?;
        let vertex_count = u64::try_from(vertex_count)
            .map_err(|_| import_error(format!("source {source_id} has a negative vertex count")))?;
        if states
            .insert(source_id, (path, source_size, vertex_count))
            .is_some()
        {
            return Err(import_error(format!(
                "source {source_id} has duplicate sampling state"
            )));
        }
    }
    Ok(states)
}

fn stable_prioritized_sample(
    index: &CityIndex,
    connection: &Connection,
    limit: usize,
) -> Result<Vec<StablePackageIdentity>> {
    let rows = stable_package_rows(connection)?;
    if let Some(pair) = rows.windows(2).find(|pair| pair[0].0 == pair[1].0) {
        return Err(import_error(format!(
            "stable package identity is ambiguous: {} / {}",
            pair[0].0.source_path, pair[0].0.model_id
        )));
    }
    let references = rows
        .iter()
        .map(|(identity, record_id)| IndexedPackageRef {
            record_id: *record_id,
            model_id: identity.model_id.clone(),
            package_type: PackageType::CityJson,
            bounds: None,
        })
        .collect::<Vec<_>>();
    let coverage = index.package_vertex_coverage(&references)?;
    let source_state = source_sampling_state(connection)?;
    let mut candidates = Vec::with_capacity(rows.len());
    for ((identity, record_id), coverage) in rows.into_iter().zip(coverage) {
        if coverage.record_id != record_id {
            return Err(import_error(format!(
                "coverage record {} does not match package {record_id}",
                coverage.record_id
            )));
        }
        let (source_path, source_size, source_vertex_count) =
            source_state.get(&coverage.source_id).ok_or_else(|| {
                import_error(format!(
                    "source {} is missing sampling state",
                    coverage.source_id
                ))
            })?;
        if source_path != &identity.source_path {
            return Err(import_error(format!(
                "package {record_id} source path does not match its coverage source"
            )));
        }
        candidates.push(SamplingCandidate {
            identity,
            source_size: *source_size,
            source_vertex_count: *source_vertex_count,
            coverage,
        });
    }
    Ok(prioritized_sample(&candidates, limit))
}

fn prioritized_sample(
    candidates: &[SamplingCandidate],
    limit: usize,
) -> Vec<StablePackageIdentity> {
    let mut by_source = BTreeMap::<String, Vec<&SamplingCandidate>>::new();
    for candidate in candidates {
        by_source
            .entry(candidate.identity.source_path.clone())
            .or_default()
            .push(candidate);
    }
    for source_candidates in by_source.values_mut() {
        source_candidates.sort_by_key(|candidate| &candidate.identity);
    }

    let capacity = limit.min(candidates.len());
    let mut sample = Vec::with_capacity(capacity);
    let mut selected = BTreeSet::new();

    // Baseline source coverage, using stable source-path and identity order.
    for source_candidates in by_source.values() {
        if let Some(candidate) = source_candidates.first() {
            push_candidate(candidate, capacity, &mut selected, &mut sample);
        }
    }

    // Prefer one new package per source for each source-relative or codec edge.
    for predicate in [
        touches_first_vertex as fn(&SamplingCandidate) -> bool,
        touches_last_vertex,
        touches_subblock_boundary,
        touches_chunk_boundary,
    ] {
        for source_candidates in by_source.values() {
            if let Some(candidate) = source_candidates
                .iter()
                .copied()
                .find(|candidate| predicate(candidate) && !selected.contains(&candidate.identity))
                .or_else(|| {
                    source_candidates
                        .iter()
                        .copied()
                        .find(|candidate| predicate(candidate))
                })
            {
                push_candidate(candidate, capacity, &mut selected, &mut sample);
            }
        }
    }

    // Give the largest source files the earliest remaining representatives.
    let mut largest_first = by_source.values().collect::<Vec<_>>();
    largest_first.sort_by(|left, right| {
        let left_size = left.first().map_or(0, |candidate| candidate.source_size);
        let right_size = right.first().map_or(0, |candidate| candidate.source_size);
        right_size.cmp(&left_size).then_with(|| {
            left[0]
                .identity
                .source_path
                .cmp(&right[0].identity.source_path)
        })
    });
    for source_candidates in largest_first {
        if let Some(candidate) = source_candidates
            .iter()
            .copied()
            .find(|candidate| !selected.contains(&candidate.identity))
        {
            push_candidate(candidate, capacity, &mut selected, &mut sample);
        }
    }

    // Fill without allowing a high-package-count source to dominate the tail.
    let queues = by_source.into_values().collect::<Vec<_>>();
    let mut offsets = vec![0_usize; queues.len()];
    while sample.len() < capacity {
        let mut progressed = false;
        for (queue, offset) in queues.iter().zip(&mut offsets) {
            while let Some(candidate) = queue.get(*offset) {
                *offset += 1;
                if selected.insert(candidate.identity.clone()) {
                    sample.push(candidate.identity.clone());
                    progressed = true;
                    break;
                }
            }
            if sample.len() == capacity {
                break;
            }
        }
        if !progressed {
            break;
        }
    }
    sample
}

fn push_candidate(
    candidate: &SamplingCandidate,
    limit: usize,
    selected: &mut BTreeSet<StablePackageIdentity>,
    sample: &mut Vec<StablePackageIdentity>,
) {
    if sample.len() < limit && selected.insert(candidate.identity.clone()) {
        sample.push(candidate.identity.clone());
    }
}

fn touches_first_vertex(candidate: &SamplingCandidate) -> bool {
    candidate.coverage.min_vertex_index == Some(0)
}

fn touches_last_vertex(candidate: &SamplingCandidate) -> bool {
    candidate.source_vertex_count > 0
        && candidate.coverage.max_vertex_index == candidate.source_vertex_count.checked_sub(1)
}

fn touches_subblock_boundary(candidate: &SamplingCandidate) -> bool {
    candidate.coverage.touches_subblock_boundary
}

fn touches_chunk_boundary(candidate: &SamplingCandidate) -> bool {
    candidate.coverage.touches_chunk_boundary
}

fn read_sample(path: &Path, corpus_identity: &str) -> Result<SampleFile> {
    let sample: SampleFile = serde_json::from_slice(&fs::read(path)?)
        .map_err(|error| import_error(error.to_string()))?;
    validate_sample(sample, corpus_identity)
}

fn validate_sample(sample: SampleFile, corpus_identity: &str) -> Result<SampleFile> {
    if sample.schema_version != SAMPLE_SCHEMA_VERSION {
        return Err(import_error(format!(
            "sample schema {} is unsupported; expected {SAMPLE_SCHEMA_VERSION}",
            sample.schema_version
        )));
    }
    if sample.corpus_identity != corpus_identity {
        return Err(import_error(
            "sample corpus identity does not match provenance",
        ));
    }
    if sample.packages.is_empty() {
        return Err(import_error("sample contains no package identities"));
    }
    if sample.sample_identity != sample_identity(&sample.corpus_identity, &sample.packages)? {
        return Err(import_error("sample identity does not match its contents"));
    }
    Ok(sample)
}

fn sample_identity(corpus_identity: &str, packages: &[StablePackageIdentity]) -> Result<String> {
    let mut hasher = Sha256::new();
    hasher.update(SAMPLE_SCHEMA_VERSION.to_le_bytes());
    hasher.update((corpus_identity.len() as u64).to_le_bytes());
    hasher.update(corpus_identity.as_bytes());
    for package in packages {
        let bytes = serde_json::to_vec(package).map_err(|error| import_error(error.to_string()))?;
        hasher.update((bytes.len() as u64).to_le_bytes());
        hasher.update(bytes);
    }
    Ok(digest_string(hasher.finalize().as_slice()))
}

fn resolve_sample_record_ids(
    connection: &Connection,
    packages: &[StablePackageIdentity],
) -> Result<Vec<i64>> {
    let mut matches = BTreeMap::<StablePackageIdentity, Vec<i64>>::new();
    for (identity, record_id) in stable_package_rows(connection)? {
        matches.entry(identity).or_default().push(record_id);
    }
    packages
        .iter()
        .map(|identity| match matches.get(identity).map(Vec::as_slice) {
            Some([record_id]) => Ok(*record_id),
            None | Some([]) => Err(import_error(format!(
                "sample package is missing: {} / {}",
                identity.source_path, identity.model_id
            ))),
            Some(_) => Err(import_error(format!(
                "sample package is ambiguous: {} / {}",
                identity.source_path, identity.model_id
            ))),
        })
        .collect()
}

fn refs_for_sample(
    index: &CityIndex,
    connection: &Connection,
    packages: &[StablePackageIdentity],
) -> Result<Vec<IndexedPackageRef>> {
    resolve_sample_record_ids(connection, packages)?
        .into_iter()
        .map(|record_id| {
            let reference = index
                .lookup_package_ref_by_record_id(record_id)?
                .ok_or_else(|| import_error(format!("sample package {record_id} was not found")))?;
            if reference.package_type != PackageType::CityJson {
                return Err(import_error(format!(
                    "sample package {record_id} is not regular CityJSON"
                )));
            }
            Ok(reference)
        })
        .collect()
}

fn all_stable_package_refs(
    index: &CityIndex,
    connection: &Connection,
) -> Result<Vec<IndexedPackageRef>> {
    let identities = all_stable_package_identities(connection)?;
    refs_for_sample(index, connection, &identities)
}

fn materialize_singletons(
    index: &CityIndex,
    store: &dyn VertexStore,
    refs: &[IndexedPackageRef],
) -> Result<(MaterializationResult, VertexStoreTelemetry)> {
    let started = Instant::now();
    let mut hasher = Sha256::new();
    let mut telemetry = VertexStoreTelemetry::default();
    for reference in refs {
        let (model, operation) = index.read_package_with_vertex_store(reference, store)?;
        hash_model(&mut hasher, &model)?;
        telemetry = sum_telemetry([telemetry, operation]);
    }
    Ok((
        MaterializationResult {
            package_count: refs.len(),
            model_digest: digest_string(hasher.finalize().as_slice()),
            elapsed_ns: started.elapsed().as_nanos(),
        },
        telemetry,
    ))
}

fn materialize_batches(
    index: &CityIndex,
    store: &dyn VertexStore,
    refs: &[IndexedPackageRef],
) -> Result<(MaterializationResult, VertexStoreTelemetry)> {
    let started = Instant::now();
    let mut hasher = Sha256::new();
    let mut telemetry = VertexStoreTelemetry::default();
    let mut package_count = 0;
    for batch in refs.chunks(READ_BATCH_SIZE) {
        let (packages, operation) = index.read_packages_with_vertex_store(batch, store)?;
        for package in &packages {
            hash_model(&mut hasher, &package.model)?;
        }
        package_count += packages.len();
        telemetry = sum_telemetry([telemetry, operation]);
    }
    Ok((
        MaterializationResult {
            package_count,
            model_digest: digest_string(hasher.finalize().as_slice()),
            elapsed_ns: started.elapsed().as_nanos(),
        },
        telemetry,
    ))
}

#[derive(Debug)]
struct MaterializedBatch {
    batch_index: usize,
    model_digests: Vec<[u8; 32]>,
    telemetry: VertexStoreTelemetry,
}

type TylerStartGate = Arc<(Mutex<Option<bool>>, Condvar)>;
type TylerReadySender = mpsc::Sender<std::result::Result<(), String>>;

fn materialize_tyler_worker(
    storage_layout: StorageLayout,
    sidecar: &Path,
    strategy: VertexStoreStrategy,
    refs: &[IndexedPackageRef],
    worker_batches: Vec<usize>,
    gate: &TylerStartGate,
    ready: &TylerReadySender,
) -> Result<Vec<MaterializedBatch>> {
    let setup = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let store = candidate::create(sidecar.to_path_buf())?;
        ensure_strategy(store.as_ref(), strategy)?;
        let index =
            CityIndex::open_vertex_store_read_only(storage_layout, sidecar, store.as_ref())?;
        Ok::<_, Error>((store, index))
    }));
    let (store, index) = match setup {
        Ok(Ok(setup)) => {
            ready.send(Ok(())).map_err(|error| {
                import_error(format!("Tyler worker could not report readiness: {error}"))
            })?;
            setup
        }
        Ok(Err(error)) => {
            let _ = ready.send(Err(error.to_string()));
            return Err(error);
        }
        Err(panic) => {
            let message = format!(
                "Tyler worker panicked during setup: {}",
                panic_message(panic.as_ref())
            );
            let _ = ready.send(Err(message.clone()));
            return Err(import_error(message));
        }
    };

    let (lock, condition) = &**gate;
    let mut run = lock
        .lock()
        .map_err(|_| import_error("Tyler start gate was poisoned"))?;
    while run.is_none() {
        run = condition
            .wait(run)
            .map_err(|_| import_error("Tyler start gate was poisoned"))?;
    }
    if !run.expect("start gate was checked") {
        return Err(import_error(
            "Tyler materialization aborted after worker setup failure",
        ));
    }
    drop(run);

    let mut output = Vec::with_capacity(worker_batches.len());
    for batch_index in worker_batches {
        let start = batch_index
            .checked_mul(READ_BATCH_SIZE)
            .ok_or_else(|| import_error("Tyler batch offset overflowed"))?;
        let end = start.saturating_add(READ_BATCH_SIZE).min(refs.len());
        let batch = refs
            .get(start..end)
            .ok_or_else(|| import_error("Tyler batch assignment is out of bounds"))?;
        let (packages, telemetry) = index.read_packages_with_vertex_store(batch, store.as_ref())?;
        if packages.len() != batch.len() {
            return Err(import_error(format!(
                "Tyler batch {batch_index} returned {} packages for {} references",
                packages.len(),
                batch.len()
            )));
        }
        let model_digests = packages
            .iter()
            .map(|package| canonical_model_digest(&package.model))
            .collect::<Result<Vec<_>>>()?;
        output.push(MaterializedBatch {
            batch_index,
            model_digests,
            telemetry,
        });
    }
    Ok(output)
}

fn release_tyler_workers(gate: &TylerStartGate, should_run: bool) -> Result<()> {
    let (lock, condition) = &**gate;
    match lock.lock() {
        Ok(mut run) => {
            *run = Some(should_run);
            condition.notify_all();
            Ok(())
        }
        Err(poisoned) => {
            *poisoned.into_inner() = Some(false);
            condition.notify_all();
            Err(import_error("Tyler start gate was poisoned"))
        }
    }
}

fn materialize_batches_parallel(
    resolved: &ResolvedDataset,
    sidecar: &Path,
    strategy: VertexStoreStrategy,
    refs: &[IndexedPackageRef],
    workers: usize,
) -> Result<(MaterializationResult, VertexStoreTelemetry)> {
    let assignments = partition_batch_indices(refs.len(), READ_BATCH_SIZE, workers)?;
    let expected_batch_count = refs.len().div_ceil(READ_BATCH_SIZE);
    let gate = Arc::new((Mutex::new(None::<bool>), Condvar::new()));
    let (ready_sender, ready_receiver) = mpsc::channel::<std::result::Result<(), String>>();

    let (mut materialized, materialization_elapsed_ns) = std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(assignments.len());
        let mut failure = None;
        for (worker_index, worker_batches) in assignments.into_iter().enumerate() {
            let storage_layout = resolved.storage_layout();
            let sidecar = sidecar.to_path_buf();
            let worker_gate = std::sync::Arc::clone(&gate);
            let worker_ready = ready_sender.clone();
            let spawn = std::thread::Builder::new()
                .name(format!("vertex-store-tyler-{worker_index}"))
                .spawn_scoped(scope, move || {
                    materialize_tyler_worker(
                        storage_layout,
                        &sidecar,
                        strategy,
                        refs,
                        worker_batches,
                        &worker_gate,
                        &worker_ready,
                    )
                });
            match spawn {
                Ok(handle) => handles.push(handle),
                Err(error) => {
                    failure = Some(import_error(format!(
                        "failed to spawn Tyler worker {worker_index}: {error}"
                    )));
                    break;
                }
            }
        }
        drop(ready_sender);

        for _ in 0..handles.len() {
            match ready_receiver.recv() {
                Ok(Ok(())) => {}
                Ok(Err(message)) => {
                    if failure.is_none() {
                        failure = Some(import_error(format!(
                            "Tyler worker setup failed: {message}"
                        )));
                    }
                }
                Err(error) => {
                    if failure.is_none() {
                        failure = Some(import_error(format!(
                            "Tyler readiness channel closed unexpectedly: {error}"
                        )));
                    }
                    break;
                }
            }
        }

        let should_run = failure.is_none() && handles.len() == workers;
        let materialization_started = should_run.then(Instant::now);
        if let Err(error) = release_tyler_workers(&gate, should_run)
            && failure.is_none()
        {
            failure = Some(error);
        }

        let mut output = Vec::with_capacity(expected_batch_count);
        for handle in handles {
            match handle.join() {
                Ok(Ok(worker_output)) => output.extend(worker_output),
                Ok(Err(error)) => {
                    if failure.is_none() {
                        failure = Some(error);
                    }
                }
                Err(panic) => {
                    if failure.is_none() {
                        failure = Some(import_error(format!(
                            "Tyler worker panicked: {}",
                            panic_message(panic.as_ref())
                        )));
                    }
                }
            }
        }
        if let Some(error) = failure {
            return Err(error);
        }
        let elapsed_ns = materialization_started
            .expect("successful worker setup starts the timer")
            .elapsed()
            .as_nanos();
        Ok((output, elapsed_ns))
    })?;
    materialized.sort_by_key(|batch| batch.batch_index);
    summarize_materialized_batches(
        &materialized,
        refs.len(),
        expected_batch_count,
        materialization_elapsed_ns,
    )
}

fn partition_batch_indices(
    package_count: usize,
    batch_size: usize,
    workers: usize,
) -> Result<Vec<Vec<usize>>> {
    if workers == 0 {
        return Err(import_error("--workers must be greater than zero"));
    }
    if batch_size == 0 {
        return Err(import_error("batch size must be greater than zero"));
    }
    let batch_count = package_count.div_ceil(batch_size);
    let mut assignments = vec![Vec::new(); workers];
    for batch_index in 0..batch_count {
        assignments[batch_index % workers].push(batch_index);
    }
    Ok(assignments)
}

fn summarize_materialized_batches(
    batches: &[MaterializedBatch],
    expected_package_count: usize,
    expected_batch_count: usize,
    elapsed_ns: u128,
) -> Result<(MaterializationResult, VertexStoreTelemetry)> {
    if batches.len() != expected_batch_count {
        return Err(import_error(format!(
            "Tyler materialized {} batches; expected {expected_batch_count}",
            batches.len()
        )));
    }
    let mut hasher = Sha256::new();
    let mut telemetry = VertexStoreTelemetry::default();
    let mut package_count = 0_usize;
    for (expected_batch_index, batch) in batches.iter().enumerate() {
        if batch.batch_index != expected_batch_index {
            return Err(import_error(format!(
                "Tyler batch order is incomplete at {expected_batch_index}"
            )));
        }
        for digest in &batch.model_digests {
            hasher.update(digest);
        }
        package_count = package_count
            .checked_add(batch.model_digests.len())
            .ok_or_else(|| import_error("Tyler package count overflowed"))?;
        telemetry = sum_telemetry([telemetry, batch.telemetry]);
    }
    if package_count != expected_package_count {
        return Err(import_error(format!(
            "Tyler materialized {package_count} packages; expected {expected_package_count}"
        )));
    }
    Ok((
        MaterializationResult {
            package_count,
            model_digest: digest_string(hasher.finalize().as_slice()),
            elapsed_ns,
        },
        telemetry,
    ))
}

fn panic_message(panic: &(dyn std::any::Any + Send)) -> &str {
    panic
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| panic.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("non-string panic payload")
}

fn summarize_packages(
    packages: &[cityjson_index::IndexedPackage],
    elapsed_ns: u128,
) -> Result<MaterializationResult> {
    let mut hasher = Sha256::new();
    for package in packages {
        hash_model(&mut hasher, &package.model)?;
    }
    Ok(MaterializationResult {
        package_count: packages.len(),
        model_digest: digest_string(hasher.finalize().as_slice()),
        elapsed_ns,
    })
}

fn digest_string(bytes: &[u8]) -> String {
    let mut value = String::with_capacity(7 + bytes.len() * 2);
    value.push_str("sha256:");
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut value, "{byte:02x}").expect("writing to a string cannot fail");
    }
    value
}

/// Hashes one canonical model digest into the package-order digest stream.
///
/// The fixed-size intermediate digest preserves the canonical model equivalence
/// relation while allowing parallel workers to retain only 32 bytes per package.
fn hash_model(hasher: &mut Sha256, model: &cityjson_lib::CityModel) -> Result<()> {
    hasher.update(canonical_model_digest(model)?);
    Ok(())
}

fn canonical_model_digest(model: &cityjson_lib::CityModel) -> Result<[u8; 32]> {
    let serialized = cityjson_lib::json::to_vec(model)?;
    let value: serde_json::Value =
        serde_json::from_slice(&serialized).map_err(|error| import_error(error.to_string()))?;
    let mut canonical = Vec::with_capacity(serialized.len());
    write_canonical_value(&value, &mut canonical)?;
    let mut hasher = Sha256::new();
    write_hash_length(&mut hasher, canonical.len())?;
    hasher.update(canonical);
    Ok(hasher.finalize().into())
}

fn write_canonical_value(value: &serde_json::Value, output: &mut Vec<u8>) -> Result<()> {
    match value {
        serde_json::Value::Null => output.push(b'n'),
        serde_json::Value::Bool(value) => output.push(if *value { b't' } else { b'f' }),
        serde_json::Value::Number(value) => {
            output.push(b'd');
            write_canonical_bytes(value.to_string().as_bytes(), output)?;
        }
        serde_json::Value::String(value) => {
            output.push(b's');
            write_canonical_bytes(value.as_bytes(), output)?;
        }
        serde_json::Value::Array(values) => {
            output.push(b'[');
            write_canonical_length(values.len(), output)?;
            for value in values {
                write_canonical_value(value, output)?;
            }
        }
        serde_json::Value::Object(values) => {
            output.push(b'{');
            write_canonical_length(values.len(), output)?;
            let mut keys = values.keys().collect::<Vec<_>>();
            keys.sort_unstable();
            for key in keys {
                write_canonical_bytes(key.as_bytes(), output)?;
                write_canonical_value(&values[key], output)?;
            }
        }
    }
    Ok(())
}

fn write_canonical_bytes(bytes: &[u8], output: &mut Vec<u8>) -> Result<()> {
    write_canonical_length(bytes.len(), output)?;
    output.extend_from_slice(bytes);
    Ok(())
}

fn write_canonical_length(length: usize, output: &mut Vec<u8>) -> Result<()> {
    let length = u64::try_from(length)
        .map_err(|_| import_error("canonical value length does not fit in u64"))?;
    output.extend_from_slice(&length.to_le_bytes());
    Ok(())
}

fn write_hash_length(hasher: &mut Sha256, length: usize) -> Result<()> {
    let length = u64::try_from(length)
        .map_err(|_| import_error("canonical model length does not fit in u64"))?;
    hasher.update(length.to_le_bytes());
    Ok(())
}

fn sum_telemetry(
    telemetry: impl IntoIterator<Item = VertexStoreTelemetry>,
) -> VertexStoreTelemetry {
    telemetry
        .into_iter()
        .fold(VertexStoreTelemetry::default(), |mut total, item| {
            total.requested_vertex_count = total
                .requested_vertex_count
                .saturating_add(item.requested_vertex_count);
            total.unique_vertex_count = total
                .unique_vertex_count
                .saturating_add(item.unique_vertex_count);
            total.returned_vertex_count = total
                .returned_vertex_count
                .saturating_add(item.returned_vertex_count);
            total.persistent_bytes_read = total
                .persistent_bytes_read
                .saturating_add(item.persistent_bytes_read);
            total.source_json_bytes_read = total
                .source_json_bytes_read
                .saturating_add(item.source_json_bytes_read);
            total.touched_units = total.touched_units.saturating_add(item.touched_units);
            total.retained_decoded_bytes = total
                .retained_decoded_bytes
                .max(item.retained_decoded_bytes);
            total
        })
}

fn pragma_i64(connection: &Connection, pragma: &str) -> Result<i64> {
    connection
        .query_row(&format!("PRAGMA {pragma}"), [], |row| row.get(0))
        .map_err(|error| sql_error(&error))
}

fn replace_sidecar(temporary: &Path, final_path: &Path) -> Result<()> {
    fs::rename(temporary, final_path).map_err(|error| {
        import_error(format!(
            "failed to replace sidecar {} with validated build {}: {error}",
            final_path.display(),
            temporary.display()
        ))
    })
}

fn temporary_sibling(path: &Path, label: &str) -> Result<PathBuf> {
    let parent = path
        .parent()
        .ok_or_else(|| import_error("path has no parent directory"))?;
    fs::create_dir_all(parent)?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| import_error("path has no UTF-8 file name"))?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| import_error(error.to_string()))?
        .as_nanos();
    Ok(parent.join(format!(
        ".{name}.{label}.{}.{nonce}.tmp",
        std::process::id()
    )))
}

fn write_json_atomically<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let temporary = temporary_sibling(path, "result")?;
    let bytes =
        serde_json::to_vec_pretty(value).map_err(|error| import_error(error.to_string()))?;
    fs::write(&temporary, bytes)?;
    if let Err(error) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(import_error(format!(
            "failed to replace result {}: {error}",
            path.display()
        )));
    }
    Ok(())
}

fn write_memory_snapshot(path: &Path, snapshot: &MemorySnapshot) -> Result<()> {
    write_json_atomically(path, snapshot)
}

fn parse_key_value(value: &str) -> std::result::Result<(String, String), String> {
    let (key, value) = value
        .split_once('=')
        .ok_or_else(|| "runtime values must use KEY=VALUE".to_owned())?;
    if key.is_empty() || value.is_empty() {
        return Err("runtime keys and values must be non-empty".to_owned());
    }
    Ok((key.to_owned(), value.to_owned()))
}

fn sql_error(error: &rusqlite::Error) -> Error {
    import_error(error.to_string())
}

fn import_error(message: impl Into<String>) -> Error {
    Error::Import(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measured_command_requires_provenance() {
        let parsed = Cli::try_parse_from([
            "vertex-store-bakeoff",
            "read-latency",
            "--dataset-root",
            "/dataset",
            "--sidecar",
            "/dataset/index.sqlite",
            "--sample",
            "/tmp/sample.json",
            "--result",
            "/tmp/result.json",
            "--candidate-commit",
            "candidate",
            "--harness-commit",
            "harness",
            "--corpus-identity",
            "corpus",
        ]);
        assert!(parsed.is_ok());
        let missing = Cli::try_parse_from([
            "vertex-store-bakeoff",
            "read-latency",
            "--dataset-root",
            "/dataset",
            "--sidecar",
            "/dataset/index.sqlite",
        ]);
        assert!(missing.is_err());
    }

    #[test]
    fn unknown_provenance_is_rejected() {
        assert!(reject_unknown("commit", "unknown").is_err());
        assert!(reject_unknown("commit", " UNKNOWN ").is_err());
        assert!(reject_unknown("commit", "abc123").is_ok());
    }

    fn identity(source: &str, model: &str) -> StablePackageIdentity {
        StablePackageIdentity {
            source_path: source.to_owned(),
            model_id: model.to_owned(),
            package_path: source.to_owned(),
            offset: Some(10),
            length: Some(20),
        }
    }

    fn package_database(rows: &[(i64, i64, &str, &str)]) -> Connection {
        let connection = Connection::open_in_memory().expect("database opens");
        connection
            .execute_batch(
                "CREATE TABLE sources (id INTEGER PRIMARY KEY, path TEXT NOT NULL);                 CREATE TABLE packages (                     id INTEGER PRIMARY KEY, source_id INTEGER NOT NULL, package_type TEXT NOT NULL,                     model_id TEXT NOT NULL, path TEXT NOT NULL, offset INTEGER, length INTEGER                 );",
            )
            .expect("schema creates");
        for (source_id, package_id, source, model) in rows {
            connection
                .execute(
                    "INSERT OR IGNORE INTO sources (id, path) VALUES (?1, ?2)",
                    rusqlite::params![source_id, source],
                )
                .expect("source inserts");
            connection
                .execute(
                    "INSERT INTO packages                      (id, source_id, package_type, model_id, path, offset, length)                      VALUES (?1, ?2, 'cityjson', ?3, ?4, 10, 20)",
                    rusqlite::params![package_id, source_id, model, source],
                )
                .expect("package inserts");
        }
        connection
    }

    fn sampling_candidate(
        source: &str,
        model: &str,
        source_size: u64,
        source_vertex_count: u64,
        bounds: (Option<u64>, Option<u64>),
        boundaries: (bool, bool),
    ) -> SamplingCandidate {
        SamplingCandidate {
            identity: identity(source, model),
            source_size,
            source_vertex_count,
            coverage: PackageVertexCoverage {
                record_id: 1,
                source_id: 1,
                min_vertex_index: bounds.0,
                max_vertex_index: bounds.1,
                touches_subblock_boundary: boundaries.0,
                touches_chunk_boundary: boundaries.1,
            },
        }
    }

    #[test]
    fn sample_forces_sources_extrema_and_unit_boundaries_without_duplicates() {
        let mut candidates = Vec::new();
        for (source, source_size) in [("/a", 100), ("/b", 200)] {
            candidates.extend([
                sampling_candidate(
                    source,
                    "0-base",
                    source_size,
                    1_000,
                    (Some(10), Some(20)),
                    (false, false),
                ),
                sampling_candidate(
                    source,
                    "1-first",
                    source_size,
                    1_000,
                    (Some(0), Some(10)),
                    (true, true),
                ),
                sampling_candidate(
                    source,
                    "2-last",
                    source_size,
                    1_000,
                    (Some(900), Some(999)),
                    (false, false),
                ),
                sampling_candidate(
                    source,
                    "3-subblock",
                    source_size,
                    1_000,
                    (Some(127), Some(130)),
                    (true, false),
                ),
                sampling_candidate(
                    source,
                    "4-chunk",
                    source_size,
                    1_000,
                    (Some(16_383), Some(16_383)),
                    (true, true),
                ),
                sampling_candidate(
                    source,
                    "5-extra",
                    source_size,
                    1_000,
                    (None, None),
                    (false, false),
                ),
            ]);
        }

        let sample = prioritized_sample(&candidates, candidates.len());
        let actual = sample
            .iter()
            .map(|identity| (identity.source_path.as_str(), identity.model_id.as_str()))
            .collect::<Vec<_>>();
        assert_eq!(
            actual,
            vec![
                ("/a", "0-base"),
                ("/b", "0-base"),
                ("/a", "1-first"),
                ("/b", "1-first"),
                ("/a", "2-last"),
                ("/b", "2-last"),
                ("/a", "3-subblock"),
                ("/b", "3-subblock"),
                ("/a", "4-chunk"),
                ("/b", "4-chunk"),
                ("/b", "5-extra"),
                ("/a", "5-extra"),
            ]
        );
        assert_eq!(sample.iter().collect::<BTreeSet<_>>().len(), sample.len());
    }

    #[test]
    fn sample_prioritizes_largest_sources_then_fills_round_robin_deterministically() {
        let mut candidates = Vec::new();
        for (source, source_size) in [("/c", 200), ("/a", 100), ("/b", 300)] {
            for model in ["0", "1", "2", "3"] {
                candidates.push(sampling_candidate(
                    source,
                    model,
                    source_size,
                    10,
                    (None, None),
                    (false, false),
                ));
            }
        }
        let expected = prioritized_sample(&candidates, 10);
        candidates.reverse();
        assert_eq!(prioritized_sample(&candidates, 10), expected);
        assert_eq!(
            expected
                .iter()
                .map(|identity| (identity.source_path.as_str(), identity.model_id.as_str()))
                .collect::<Vec<_>>(),
            vec![
                ("/a", "0"),
                ("/b", "0"),
                ("/c", "0"),
                ("/b", "1"),
                ("/c", "1"),
                ("/a", "1"),
                ("/a", "2"),
                ("/b", "2"),
                ("/c", "2"),
                ("/a", "3"),
            ]
        );
    }

    #[test]
    fn stable_sample_survives_different_numeric_ids() {
        let first = package_database(&[(1, 10, "/a.city.json", "a"), (2, 20, "/b.city.json", "b")]);
        let second = package_database(&[
            (92, 700, "/b.city.json", "b"),
            (41, 900, "/a.city.json", "a"),
        ]);
        let sample = vec![identity("/a.city.json", "a"), identity("/b.city.json", "b")];
        assert_eq!(
            resolve_sample_record_ids(&first, &sample).expect("first resolves"),
            vec![10, 20]
        );
        assert_eq!(
            resolve_sample_record_ids(&second, &sample).expect("second resolves"),
            vec![900, 700]
        );
        assert_eq!(
            sample_identity("corpus", &sample).expect("identity hashes"),
            sample_identity("corpus", &sample).expect("identity rehashes")
        );
    }

    #[test]
    fn full_scan_order_and_batch_boundaries_ignore_numeric_ids() {
        let first = package_database(&[
            (1, 10, "/c.city.json", "c"),
            (2, 20, "/a.city.json", "a2"),
            (2, 21, "/a.city.json", "a1"),
            (3, 30, "/b.city.json", "b2"),
            (3, 31, "/b.city.json", "b1"),
        ]);
        let second = package_database(&[
            (93, 501, "/b.city.json", "b1"),
            (42, 801, "/a.city.json", "a1"),
            (71, 101, "/c.city.json", "c"),
            (93, 502, "/b.city.json", "b2"),
            (42, 802, "/a.city.json", "a2"),
        ]);
        let first_order = all_stable_package_identities(&first).expect("first order builds");
        let second_order = all_stable_package_identities(&second).expect("second order builds");
        assert_eq!(first_order, second_order);
        assert_eq!(
            first_order.chunks(2).map(<[_]>::to_vec).collect::<Vec<_>>(),
            second_order
                .chunks(2)
                .map(<[_]>::to_vec)
                .collect::<Vec<_>>()
        );
        assert_ne!(
            resolve_sample_record_ids(&first, &first_order).expect("first ids resolve"),
            resolve_sample_record_ids(&second, &second_order).expect("second ids resolve")
        );
    }

    #[test]
    fn sample_resolution_rejects_missing_and_ambiguous_identity() {
        let connection =
            package_database(&[(1, 10, "/a.city.json", "a"), (1, 11, "/a.city.json", "a")]);
        assert!(
            resolve_sample_record_ids(&connection, &[identity("/missing.city.json", "x")]).is_err()
        );
        assert!(resolve_sample_record_ids(&connection, &[identity("/a.city.json", "a")]).is_err());
    }

    #[test]
    fn old_numeric_id_sample_schema_is_rejected() {
        let old = serde_json::json!({
            "schema_version": 1,
            "corpus_identity": "corpus",
            "sample_identity": "old",
            "record_ids": [1, 2, 3],
            "packages": [identity("/a.city.json", "a")],
        });
        let sample: SampleFile = serde_json::from_value(old).expect("shape parses");
        assert!(validate_sample(sample, "corpus").is_err());
    }

    #[test]
    fn sample_identity_is_stable_and_order_sensitive() {
        let first = sample_identity("corpus", &[identity("/a", "a"), identity("/b", "b")])
            .expect("identity hashes");
        assert_eq!(
            first,
            sample_identity("corpus", &[identity("/a", "a"), identity("/b", "b")])
                .expect("identity rehashes")
        );
        assert_ne!(
            first,
            sample_identity("corpus", &[identity("/b", "b"), identity("/a", "a")])
                .expect("reordered identity hashes")
        );
    }

    fn feature_with_attributes(attributes: &str) -> cityjson_lib::CityModel {
        let feature = format!(
            r#"{{
                "type":"CityJSONFeature",
                "id":"building",
                "CityObjects":{{
                    "building":{{
                        "type":"Building",
                        "attributes":{attributes}
                    }}
                }},
                "vertices":[]
            }}"#
        );
        cityjson_lib::json::from_feature_slice(feature.as_bytes()).expect("feature fixture parses")
    }

    fn digest_models(models: &[&cityjson_lib::CityModel]) -> String {
        let mut hasher = Sha256::new();
        for model in models {
            hash_model(&mut hasher, model).expect("model hashes");
        }
        digest_string(hasher.finalize().as_slice())
    }

    #[test]
    fn model_digest_canonicalizes_map_order_and_detects_value_changes() {
        let alpha_first = feature_with_attributes(r#"{"alpha":1,"beta":"two"}"#);
        let beta_first = feature_with_attributes(r#"{"beta":"two","alpha":1}"#);
        let changed = feature_with_attributes(r#"{"alpha":1,"beta":"changed"}"#);

        assert_eq!(
            digest_models(&[&alpha_first]),
            digest_models(&[&beta_first])
        );
        assert_ne!(digest_models(&[&alpha_first]), digest_models(&[&changed]));
        assert_eq!(
            digest_models(&[&alpha_first, &changed]),
            digest_models(&[&beta_first, &changed])
        );
    }

    #[test]
    fn batch_partition_is_deterministic_and_keeps_complete_batches() {
        assert_eq!(
            partition_batch_indices(9, 2, 3).expect("partition builds"),
            vec![vec![0, 3], vec![1, 4], vec![2]]
        );
        assert_eq!(
            partition_batch_indices(3, 2, 4).expect("wide partition builds"),
            vec![vec![0], vec![1], vec![], vec![]]
        );
        assert!(partition_batch_indices(1, 2, 0).is_err());
        assert!(partition_batch_indices(1, 0, 1).is_err());
    }

    #[test]
    fn tyler_rejects_workers_outside_the_adr_campaign() {
        for workers in [1, 4, 24] {
            assert!(validate_tyler_workers(workers).is_ok());
        }
        for workers in [0, 2, 3, 23, 25] {
            assert!(validate_tyler_workers(workers).is_err());
        }
    }

    #[test]
    fn fixed_model_digests_preserve_equivalence_and_package_order() {
        let alpha_first = feature_with_attributes(r#"{"alpha":1,"beta":"two"}"#);
        let beta_first = feature_with_attributes(r#"{"beta":"two","alpha":1}"#);
        let changed = feature_with_attributes(r#"{"alpha":1,"beta":"changed"}"#);
        assert_eq!(
            canonical_model_digest(&alpha_first).expect("model hashes"),
            canonical_model_digest(&beta_first).expect("equivalent model hashes")
        );

        let first = digest_models(&[&alpha_first, &changed]);
        let reversed = digest_models(&[&changed, &alpha_first]);
        assert_ne!(first, reversed);

        let batches = vec![
            MaterializedBatch {
                batch_index: 0,
                model_digests: vec![
                    canonical_model_digest(&alpha_first).expect("first model hashes"),
                ],
                telemetry: VertexStoreTelemetry::default(),
            },
            MaterializedBatch {
                batch_index: 1,
                model_digests: vec![canonical_model_digest(&changed).expect("second model hashes")],
                telemetry: VertexStoreTelemetry::default(),
            },
        ];
        let (summary, _) =
            summarize_materialized_batches(&batches, 2, 2, 7).expect("batches summarize");
        assert_eq!(summary.model_digest, first);
        assert_eq!(summary.elapsed_ns, 7);

        let incomplete = vec![MaterializedBatch {
            batch_index: 1,
            model_digests: Vec::new(),
            telemetry: VertexStoreTelemetry::default(),
        }];
        assert!(summarize_materialized_batches(&incomplete, 0, 1, 0).is_err());
    }

    #[test]
    fn telemetry_sums_reads_and_keeps_peak_retained_memory() {
        let total = sum_telemetry([
            VertexStoreTelemetry {
                persistent_bytes_read: 10,
                retained_decoded_bytes: 20,
                ..VertexStoreTelemetry::default()
            },
            VertexStoreTelemetry {
                persistent_bytes_read: 5,
                retained_decoded_bytes: 8,
                ..VertexStoreTelemetry::default()
            },
        ]);
        assert_eq!(total.persistent_bytes_read, 15);
        assert_eq!(total.retained_decoded_bytes, 20);
    }

    #[test]
    fn optional_memory_snapshot_replaces_previous_output_atomically() {
        let path = std::env::temp_dir().join(format!(
            "cityjson-index-bakeoff-profile-{}.json",
            std::process::id()
        ));
        let first = MemorySnapshot {
            current_rss_bytes: 11,
            process_peak_rss_bytes: 22,
            peak_rss_bytes: 22,
        };
        let second = MemorySnapshot {
            current_rss_bytes: 33,
            process_peak_rss_bytes: 44,
            peak_rss_bytes: 44,
        };
        write_memory_snapshot(&path, &first).expect("first profile writes");
        write_memory_snapshot(&path, &second).expect("replacement profile writes");
        let value: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).expect("profile reads")).expect("json parses");
        assert_eq!(value["current_rss_bytes"], 33);
        assert_eq!(value["process_peak_rss_bytes"], 44);
        assert_eq!(value["peak_rss_bytes"], 44);
        let temporary_prefix = format!(
            ".{}.result.",
            path.file_name()
                .expect("profile file name")
                .to_string_lossy()
        );
        let temporary_exists = fs::read_dir(path.parent().expect("profile parent"))
            .expect("profile parent reads")
            .any(|entry| {
                entry
                    .expect("profile directory entry")
                    .file_name()
                    .to_string_lossy()
                    .starts_with(&temporary_prefix)
            });
        assert!(!temporary_exists);
        fs::remove_file(path).expect("profile removes");
    }
}
