//! Candidate-independent sidecar construction and read validation.

use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

use cityjson_lib::{Error, Result};
use rusqlite::{Connection, params};

use super::{SourceVertexState, VertexStoreStrategy};

/// Replaces the common per-source construction summaries.
///
/// # Errors
///
/// Returns an error for invalid summaries or `SQLite` failures.
pub fn write_source_vertex_states(
    connection: &Connection,
    states: &[SourceVertexState],
) -> Result<()> {
    sqlite(connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS vertex_store_source_state (\
             source_id INTEGER PRIMARY KEY REFERENCES sources(id) ON DELETE CASCADE,\
             vertex_count INTEGER NOT NULL CHECK (vertex_count >= 0),\
             unit_count INTEGER NOT NULL CHECK (unit_count >= 0),\
             payload_bytes INTEGER NOT NULL CHECK (payload_bytes >= 0)\
         );\
         DELETE FROM vertex_store_source_state;",
    ))?;

    let mut previous_source = None;
    for state in states {
        if state.source_id < 0 || previous_source.is_some_and(|source| source >= state.source_id) {
            return Err(import_error(
                "source vertex states must be sorted by unique non-negative source id",
            ));
        }
        if state.vertex_count == 0 && state.unit_count != 0 {
            return Err(import_error("empty vertex sources must not contain units"));
        }
        sqlite(connection.execute(
            "INSERT INTO vertex_store_source_state \
             (source_id, vertex_count, unit_count, payload_bytes) VALUES (?1, ?2, ?3, ?4)",
            params![
                state.source_id,
                u64_to_i64(state.vertex_count, "source vertex count")?,
                u64_to_i64(state.unit_count, "source unit count")?,
                u64_to_i64(state.payload_bytes, "source payload bytes")?,
            ],
        ))?;
        previous_source = Some(state.source_id);
    }
    Ok(())
}

/// Validates schema-v2 state, marker, complete source coverage, and freshness.
///
/// # Errors
///
/// Returns an error for incomplete, stale, mismatched, or malformed state.
#[allow(clippy::too_many_lines)] // Candidate-independent fail-closed validation.
pub fn validate_common_read_sidecar(
    connection: &Connection,
    strategy: VertexStoreStrategy,
) -> Result<()> {
    for table in [
        "schema_state",
        "sources",
        "packages",
        "cityobjects",
        "package_cityobjects",
        "cityobject_relationships",
        "vertex_store_bakeoff_state",
        "vertex_store_source_state",
    ] {
        let exists = sqlite(connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
            params![table],
            |row| row.get::<_, bool>(0),
        ))?;
        if !exists {
            return Err(import_error(format!(
                "sidecar is missing required table {table}"
            )));
        }
    }

    let (schema_version, needs_reindex) = sqlite(connection.query_row(
        "SELECT schema_version, needs_reindex FROM schema_state WHERE id = 1",
        [],
        |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
    ))?;
    if schema_version != 2 || needs_reindex != 0 {
        return Err(import_error(format!(
            "sidecar requires a full schema-v2 reindex \
             (schema={schema_version}, needs_reindex={needs_reindex})"
        )));
    }

    let (bakeoff_version, stored_strategy) = sqlite(connection.query_row(
        "SELECT schema_version, strategy FROM vertex_store_bakeoff_state WHERE id = 1",
        [],
        |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
    ))?;
    if bakeoff_version != super::BAKEOFF_SCHEMA_VERSION {
        return Err(import_error(format!(
            "sidecar bake-off schema {bakeoff_version} is stale"
        )));
    }
    if stored_strategy != strategy.identifier() {
        return Err(import_error(format!(
            "sidecar strategy {stored_strategy} does not match requested {}",
            strategy.identifier()
        )));
    }

    let non_cityjson = sqlite(connection.query_row(
        "SELECT COUNT(*) FROM packages WHERE package_type <> 'cityjson'",
        [],
        |row| row.get::<_, i64>(0),
    ))?;
    if non_cityjson != 0 {
        return Err(import_error(
            "vertex-store sidecars may contain only regular CityJSON packages",
        ));
    }

    let mut statement = sqlite(connection.prepare(
        "SELECT s.id, s.path, s.source_size, s.source_mtime_ns, \
                v.vertex_count, v.unit_count, v.payload_bytes \
         FROM sources AS s \
         LEFT JOIN vertex_store_source_state AS v ON v.source_id = s.id \
         WHERE s.vertices_offset IS NOT NULL ORDER BY s.id",
    ))?;
    let rows = sqlite(statement.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, Option<i64>>(4)?,
            row.get::<_, Option<i64>>(5)?,
            row.get::<_, Option<i64>>(6)?,
        ))
    }))?;
    for row in rows {
        let (source_id, path, expected_size, expected_mtime, vertices, units, payload) =
            sqlite(row)?;
        let (Some(vertices), Some(units), Some(payload)) = (vertices, units, payload) else {
            return Err(import_error(format!(
                "source {source_id} is missing vertex-store coverage state"
            )));
        };
        if vertices < 0 || units < 0 || payload < 0 || (vertices == 0 && units != 0) {
            return Err(import_error(format!(
                "source {source_id} has invalid vertex-store coverage state"
            )));
        }
        validate_freshness(Path::new(&path), expected_size, expected_mtime)?;
    }

    let orphan_states = sqlite(connection.query_row(
        "SELECT COUNT(*) FROM vertex_store_source_state AS v \
         LEFT JOIN sources AS s ON s.id = v.source_id WHERE s.id IS NULL",
        [],
        |row| row.get::<_, i64>(0),
    ))?;
    if orphan_states != 0 {
        return Err(import_error(
            "vertex-store source state contains orphan rows",
        ));
    }

    let foreign_key_error = sqlite(connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM pragma_foreign_key_check)",
        [],
        |row| row.get::<_, bool>(0),
    ))?;
    if foreign_key_error {
        return Err(import_error("sidecar foreign-key validation failed"));
    }
    Ok(())
}

fn validate_freshness(path: &Path, expected_size: i64, expected_mtime: i64) -> Result<()> {
    let metadata = fs::metadata(path).map_err(|error| {
        import_error(format!(
            "cannot stat indexed source {}: {error}",
            path.display()
        ))
    })?;
    let actual_size = i64::try_from(metadata.len())
        .map_err(|_| import_error("source file size exceeds SQLite range"))?;
    let actual_mtime = metadata
        .modified()?
        .duration_since(UNIX_EPOCH)
        .map_err(|_| import_error("source modification time predates Unix epoch"))?
        .as_nanos();
    let actual_mtime = i64::try_from(actual_mtime)
        .map_err(|_| import_error("source modification time exceeds SQLite range"))?;
    if actual_size != expected_size || actual_mtime != expected_mtime {
        return Err(import_error(format!(
            "indexed source {} is stale; run an explicit rebuild",
            path.display()
        )));
    }
    Ok(())
}

fn u64_to_i64(value: u64, label: &str) -> Result<i64> {
    i64::try_from(value).map_err(|_| import_error(format!("{label} exceeds SQLite range")))
}

fn sqlite<T>(result: rusqlite::Result<T>) -> Result<T> {
    result.map_err(|error| import_error(error.to_string()))
}

fn import_error(message: impl Into<String>) -> Error {
    Error::Import(message.into())
}
