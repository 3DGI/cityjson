use std::fs;
use std::path::PathBuf;

use rusqlite::Connection;

use super::*;

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
fn marker_only_database_is_rejected() {
    let path = temporary_sidecar("marker-only");
    let connection = Connection::open(&path).expect("sidecar opens");
    write_sidecar_marker(&connection, VertexStoreStrategy::PackedChunks).expect("marker writes");
    drop(connection);

    let error = open_matching_read_sidecar(&path, VertexStoreStrategy::PackedChunks)
        .expect_err("normalized tables are mandatory");
    assert!(error.to_string().contains("required table"));
    fs::remove_file(path).expect("temporary sidecar removes");
}

#[test]
fn source_state_rejects_units_for_empty_source() {
    let connection = Connection::open_in_memory().expect("memory database opens");
    connection
        .execute_batch(
            "PRAGMA foreign_keys = ON; \
             CREATE TABLE sources (id INTEGER PRIMARY KEY);",
        )
        .expect("source table creates");
    connection
        .execute("INSERT INTO sources (id) VALUES (1)", [])
        .expect("source inserts");
    let error = write_source_vertex_states(
        &connection,
        &[SourceVertexState {
            source_id: 1,
            vertex_count: 0,
            unit_count: 1,
            payload_bytes: 0,
        }],
    )
    .expect_err("empty source cannot have units");
    assert!(error.to_string().contains("empty vertex sources"));
}
