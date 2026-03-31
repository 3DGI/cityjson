# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`cityarrow` is a Rust transport library for moving `cityjson-rs` models across
Arrow IPC and Parquet package boundaries.

The semantic unit remains `cityjson::v2_0::OwnedCityModel`.
`CityModelArrowParts` is the canonical transport decomposition used by package
readers and writers.

## Key Commands

- Build: `cargo build`
- Test: `cargo test`
- Check: `just check`
- Clippy: `just clippy`
- Format: `cargo fmt`
- Ignored real-data acceptance tests: `just acceptance`

## Architecture

Current source layout:

- `src/lib.rs`: public exports
- `src/convert/mod.rs`: `OwnedCityModel` <-> `CityModelArrowParts`
- `src/package/write.rs`: canonical package write for Parquet and Arrow IPC
- `src/package/read.rs`: canonical package read for Parquet and Arrow IPC
- `src/package/mod.rs`: shared package helpers and schema inference support
- `src/schema.rs`: canonical schema definitions, manifest types, and transport
  structs
- `src/error.rs`: crate error type

## Dependencies

- Local `cityjson-rs` checkout via `cityjson = { path = "../cityjson-rs" }`
- Local `serde_cityjson` checkout in dev-dependencies for acceptance tests
- Arrow and Parquet crates for canonical table I/O
- `serde` and `serde_json` for manifest and projected payload serialization

## Testing

Tests live in `tests/`.

Important coverage layers:

- conversion and canonical roundtrip tests over synthetic fixtures
- package I/O tests for both encodings
- schema and manifest surface checks
- ignored real-data acceptance tests in `tests/manifest_roundtrip.rs`

The ignored real-data tests are intentionally expensive. Run them only
explicitly.

## Documentation

- `README.md`: project overview and verification summary
- `docs/design.md`: transport design and invariants
- `docs/package-schema.md`: canonical package layout and manifest contract

## Development Notes

- The canonical package schema id is `cityarrow.package.v1alpha1`.
- Keep docs aligned with the code in `src/schema.rs` and `src/package/`.
- Avoid introducing claims about formats, modules, or views that are not
  implemented in this repository.
