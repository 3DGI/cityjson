//! Candidate-independent vertex counts derived from authoritative source JSON.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;

use cityjson_lib::{Error, Result};
use rusqlite::Connection;
use serde::de::{Deserializer as _, IgnoredAny, SeqAccess, Visitor};

use super::SourceVertexState;

/// Proves that candidate summaries cover every indexed `CityJSON` source and
/// match counts obtained independently from the authoritative vertices arrays.
///
/// # Errors
///
/// Returns an error for incomplete coverage, malformed JSON, or count mismatches.
pub fn validate_source_states_against_sources(
    connection: &Connection,
    states: &[SourceVertexState],
) -> Result<()> {
    let mut statement = sqlite(connection.prepare(
        "SELECT id, path, vertices_offset, vertices_length FROM sources \
         WHERE vertices_offset IS NOT NULL ORDER BY id",
    ))?;
    let rows = sqlite(statement.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            PathBuf::from(row.get::<_, String>(1)?),
            row.get::<_, i64>(2)?,
            row.get::<_, i64>(3)?,
        ))
    }))?;
    let sources = sqlite(rows.collect::<rusqlite::Result<Vec<_>>>())?;
    if states.len() != sources.len() {
        return Err(import_error(format!(
            "candidate reported {} source states for {} indexed CityJSON sources",
            states.len(),
            sources.len()
        )));
    }

    for ((source_id, path, offset, length), state) in sources.into_iter().zip(states) {
        if state.source_id != source_id {
            return Err(import_error(format!(
                "candidate source state {} does not match expected source {source_id}",
                state.source_id
            )));
        }
        let expected = count_vertices(&path, non_negative(offset)?, non_negative(length)?)?;
        if state.vertex_count != expected {
            return Err(import_error(format!(
                "candidate reported {} vertices for source {source_id}; authoritative JSON has {expected}",
                state.vertex_count
            )));
        }
        let expected_units = expected.div_ceil(16_384);
        if state.unit_count != expected_units {
            return Err(import_error(format!(
                "candidate reported {} units for source {source_id}; expected {expected_units}",
                state.unit_count
            )));
        }
        if expected == 0 && state.payload_bytes != 0 {
            return Err(import_error(format!(
                "empty source {source_id} has non-zero candidate payload"
            )));
        }
    }
    Ok(())
}

fn count_vertices(path: &PathBuf, offset: u64, length: u64) -> Result<u64> {
    let mut file = File::open(path)
        .map_err(|error| import_error(format!("cannot open source {}: {error}", path.display())))?;
    file.seek(SeekFrom::Start(offset))
        .map_err(|error| import_error(format!("cannot seek source {}: {error}", path.display())))?;
    let mut deserializer = serde_json::Deserializer::from_reader(file.take(length));
    deserializer.deserialize_seq(CountVisitor).map_err(|error| {
        import_error(format!(
            "cannot count vertices in {}: {error}",
            path.display()
        ))
    })
}

struct CountVisitor;

impl<'de> Visitor<'de> for CountVisitor {
    type Value = u64;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a CityJSON vertices array")
    }

    fn visit_seq<A>(self, mut sequence: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut count = 0_u64;
        while sequence.next_element::<IgnoredAny>()?.is_some() {
            count = count
                .checked_add(1)
                .ok_or_else(|| serde::de::Error::custom("vertex count overflow"))?;
        }
        Ok(count)
    }
}

fn non_negative(value: i64) -> Result<u64> {
    u64::try_from(value).map_err(|_| import_error("negative indexed vertices range"))
}

fn sqlite<T>(result: rusqlite::Result<T>) -> Result<T> {
    result.map_err(|error| import_error(error.to_string()))
}

fn import_error(message: impl Into<String>) -> Error {
    Error::Import(message.into())
}
