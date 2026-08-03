//! Shared, coordinator-owned contract for the persistent vertex-store bake-off.
//!
//! Candidate branches may add implementations below `experiments/vertex_store`,
//! but must not change the identifiers, sidecar marker, result schema, or
//! batching contract in this module.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use cityjson_lib::{Error, Result};
use rusqlite::{Connection, OpenFlags, OptionalExtension, params};
use serde::{Deserialize, Serialize};

/// Schema marker written to every experiment sidecar.
pub const BAKEOFF_SCHEMA_VERSION: i64 = 3;
/// Version of the machine-readable measurement result format.
pub const RESULT_SCHEMA_VERSION: u32 = 1;
/// Number of package references in the shared batch experiment.
pub const READ_BATCH_SIZE: usize = 2_048;
/// Number of references in the deterministic Groningen read sample.
pub const SAMPLE_SIZE: usize = 10_000;

/// The fixed candidates described by ADR 012.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VertexStoreStrategy {
    PackedChunks,
    JsonOffsets,
    FrameOfReference,
}

impl VertexStoreStrategy {
    /// Stable strategy identifier used in sidecar and result provenance.
    #[must_use]
    pub const fn identifier(self) -> &'static str {
        match self {
            Self::PackedChunks => "packed-chunks",
            Self::JsonOffsets => "json-offsets",
            Self::FrameOfReference => "frame-of-reference",
        }
    }

    /// Returns the isolated sidecar name for this strategy.
    #[must_use]
    pub fn sidecar_path(self, dataset_root: &Path) -> PathBuf {
        dataset_root.join(format!(".cityjson-index-{}.sqlite", self.identifier()))
    }
}

/// One exact coordinate required for the current package batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct VertexRequirement {
    pub source_id: i64,
    pub vertex_index: u64,
}

/// Per-call I/O telemetry. Values are never cumulative process state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VertexStoreTelemetry {
    pub requested_vertex_count: u64,
    pub unique_vertex_count: u64,
    pub returned_vertex_count: u64,
    pub persistent_bytes_read: u64,
    pub source_json_bytes_read: u64,
    pub touched_units: u64,
}

/// Candidate-specific persistent storage boundary.
///
/// `build` is called only by an explicit reindex command. `validate_for_read`
/// and `load` must never create, migrate, delete, or rebuild persistent data.
pub trait VertexStore {
    /// Candidate identifier; it must match the sidecar marker.
    fn strategy(&self) -> VertexStoreStrategy;

    /// Builds persistent vertex state after the normalized index has been built.
    fn build(&mut self, connection: &mut Connection) -> Result<()>;

    /// Validates the candidate tables before serving a measured read process.
    fn validate_for_read(&self, connection: &Connection) -> Result<()>;

    /// Loads exactly the sorted, deduplicated requirements for one batch.
    fn load(
        &self,
        requirements: &[VertexRequirement],
    ) -> Result<(BTreeMap<VertexRequirement, [i64; 3]>, VertexStoreTelemetry)>;
}

/// Makes the shared requirement list deterministic and removes only duplicate
/// `(source_id, vertex_index)` pairs from the current call.
#[must_use]
pub fn deduplicate_requirements(
    requirements: impl IntoIterator<Item = VertexRequirement>,
) -> Vec<VertexRequirement> {
    requirements
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// Sidecar provenance written only by an explicit construction operation.
pub fn write_sidecar_marker(connection: &Connection, strategy: VertexStoreStrategy) -> Result<()> {
    sqlite(connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS vertex_store_bakeoff_state (\
             id INTEGER PRIMARY KEY CHECK (id = 1),\
             schema_version INTEGER NOT NULL,\
             strategy TEXT NOT NULL\
         );",
    ))?;
    sqlite(connection.execute(
        "INSERT INTO vertex_store_bakeoff_state (id, schema_version, strategy) VALUES (1, ?1, ?2) \
         ON CONFLICT(id) DO UPDATE SET schema_version = excluded.schema_version, strategy = excluded.strategy",
        params![BAKEOFF_SCHEMA_VERSION, strategy.identifier()],
    ))?;
    Ok(())
}

/// Opens and validates an already-built experiment sidecar without modifying it.
pub fn open_matching_read_sidecar(
    path: &Path,
    strategy: VertexStoreStrategy,
) -> Result<Connection> {
    if !path.is_file() {
        return Err(import_error(format!(
            "missing {} sidecar {}; run an explicit reindex first",
            strategy.identifier(),
            path.display()
        )));
    }
    let connection = sqlite(Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY,
    ))?;
    let marker = sqlite(
        connection
            .query_row(
                "SELECT schema_version, strategy FROM vertex_store_bakeoff_state WHERE id = 1",
                [],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
            )
            .optional(),
    )?
    .ok_or_else(|| import_error("sidecar is missing the vertex-store bakeoff marker"))?;
    if marker.0 != BAKEOFF_SCHEMA_VERSION {
        return Err(import_error(format!(
            "sidecar schema {} is stale; expected bakeoff schema {BAKEOFF_SCHEMA_VERSION}",
            marker.0
        )));
    }
    if marker.1 != strategy.identifier() {
        return Err(import_error(format!(
            "sidecar strategy {} does not match requested {}",
            marker.1,
            strategy.identifier()
        )));
    }
    Ok(connection)
}

/// Deterministically takes a source-stratified sample without randomness.
/// Every non-empty source contributes one reference before the remaining slots
/// are filled by a stable round-robin over source-local record-id order.
#[must_use]
pub fn deterministic_stratified_sample(
    by_source: &BTreeMap<i64, Vec<i64>>,
    limit: usize,
) -> Vec<i64> {
    let mut queues = by_source
        .values()
        .map(|ids| {
            let mut ids = ids.clone();
            ids.sort_unstable();
            ids
        })
        .filter(|ids| !ids.is_empty())
        .collect::<Vec<_>>();
    let mut sample = Vec::with_capacity(limit.min(queues.iter().map(Vec::len).sum()));
    let mut offsets = vec![0_usize; queues.len()];
    while sample.len() < limit {
        let mut progressed = false;
        for (queue, offset) in queues.iter_mut().zip(&mut offsets) {
            if let Some(record_id) = queue.get(*offset) {
                sample.push(*record_id);
                *offset += 1;
                progressed = true;
                if sample.len() == limit {
                    break;
                }
            }
        }
        if !progressed {
            break;
        }
    }
    sample
}

/// Provenance required in every result artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BakeoffProvenance {
    pub strategy: VertexStoreStrategy,
    pub candidate_commit: String,
    pub harness_commit: String,
    pub corpus_identity: String,
    pub sidecar_path: PathBuf,
    pub worker_count: usize,
    pub repetition: usize,
    pub runtime_configuration: BTreeMap<String, String>,
}

/// Versioned JSON envelope shared by all four ADR 012 experiments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BakeoffResult<T> {
    pub schema_version: u32,
    pub experiment: String,
    pub provenance: BakeoffProvenance,
    pub telemetry: VertexStoreTelemetry,
    pub result: T,
}

impl<T> BakeoffResult<T> {
    #[must_use]
    pub fn new(
        experiment: impl Into<String>,
        provenance: BakeoffProvenance,
        telemetry: VertexStoreTelemetry,
        result: T,
    ) -> Self {
        Self {
            schema_version: RESULT_SCHEMA_VERSION,
            experiment: experiment.into(),
            provenance,
            telemetry,
            result,
        }
    }
}

/// Writes one result artifact atomically enough for a single-process harness.
pub fn write_result<T: Serialize>(path: &Path, result: &BakeoffResult<T>) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| import_error("result path has no parent directory"))?;
    fs::create_dir_all(parent)?;
    let bytes =
        serde_json::to_vec_pretty(result).map_err(|error| import_error(error.to_string()))?;
    fs::write(path, bytes)?;
    Ok(())
}

fn sqlite<T>(value: rusqlite::Result<T>) -> Result<T> {
    value.map_err(|error| import_error(error.to_string()))
}

fn import_error(message: impl Into<String>) -> Error {
    Error::Import(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requirements_are_sorted_and_deduplicated() {
        let requirements = deduplicate_requirements([
            VertexRequirement {
                source_id: 2,
                vertex_index: 8,
            },
            VertexRequirement {
                source_id: 1,
                vertex_index: 3,
            },
            VertexRequirement {
                source_id: 2,
                vertex_index: 8,
            },
        ]);
        assert_eq!(requirements.len(), 2);
        assert_eq!(requirements[0].source_id, 1);
    }

    #[test]
    fn sample_is_source_stratified_and_stable() {
        let sources = BTreeMap::from([(1, vec![9, 3]), (2, vec![8, 4]), (3, vec![7])]);
        assert_eq!(
            deterministic_stratified_sample(&sources, 5),
            vec![3, 4, 7, 9, 8]
        );
    }

    fn temporary_sidecar(label: &str) -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock is after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "cityjson-index-bakeoff-{label}-{}-{nonce}.sqlite",
            std::process::id()
        ))
    }

    #[test]
    fn read_sidecar_rejects_stale_and_mismatched_markers() {
        let path = temporary_sidecar("marker");
        let connection = Connection::open(&path).expect("sidecar opens");
        write_sidecar_marker(&connection, VertexStoreStrategy::PackedChunks)
            .expect("marker writes");
        drop(connection);
        assert!(open_matching_read_sidecar(&path, VertexStoreStrategy::JsonOffsets).is_err());

        let connection = Connection::open(&path).expect("sidecar reopens");
        connection
            .execute(
                "UPDATE vertex_store_bakeoff_state SET schema_version = 2 WHERE id = 1",
                [],
            )
            .expect("schema marker updates");
        drop(connection);
        assert!(open_matching_read_sidecar(&path, VertexStoreStrategy::PackedChunks).is_err());
        fs::remove_file(path).expect("temporary sidecar removes");
    }

    #[test]
    fn strategy_paths_are_distinct() {
        let root = Path::new("dataset");
        assert_ne!(
            VertexStoreStrategy::PackedChunks.sidecar_path(root),
            VertexStoreStrategy::JsonOffsets.sidecar_path(root)
        );
    }
}
