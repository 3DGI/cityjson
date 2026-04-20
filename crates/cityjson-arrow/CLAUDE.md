# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`cityjson-arrow` is the Arrow stream and batch codec for `cityjson-rs`.

The data model is `cityjson::v2_0::OwnedCityModel`. The canonical Arrow tables
used for transport are an internal detail shared with the sibling `cityjson-parquet` crate.

## Key Commands

- Build: `cargo build`
- Test: `cargo test`
- Check: `just check`
- Lint: `just lint`
- Format: `cargo fmt`
- Doc site: `just site-build`
- Rust API docs: `just rustdoc`

## Architecture

Current source layout:

- `src/lib.rs`: public exports
- `src/codec.rs`: `write_stream`, `read_stream`, `export_reader`, `import_batches`, `ModelBatchDecoder`, `ModelBatchReader`
- `src/stream.rs`: live Arrow IPC framing (prelude, frames, end marker)
- `src/schema.rs`: `CityArrowHeader`, `ProjectionLayout`, `PackageManifest`, and related schema types
- `src/transport.rs`: `CanonicalTable` enum and `CanonicalTableSink` trait (internal)
- `src/convert/`: export and import implementation
  - `export.rs`: model → canonical batches via `ModelRelationalView`
  - `import.rs`: canonical batches → model reconstruction
  - `projection.rs`: attribute projection layout discovery
  - `arrow.rs`, `geometry.rs`: Arrow type and geometry encoding helpers
- `src/internal.rs`: bridges kept for `cityjson-parquet` and benchmarks
- `src/error.rs`: crate error type

## Dependencies

- Local `cityjson-rs` checkout via `cityjson = { path = "../cityjson-rs" }`
- Local `serde_cityjson` checkout in dev-dependencies for acceptance tests
- Arrow crates for table I/O
- `serde` and `serde_json` for manifest and prelude serialization

## Testing

Tests live in `tests/`.

Coverage layers:

- conversion and canonical roundtrip tests over synthetic fixtures
- stream and batch codec tests
- schema and manifest surface checks
- shared-corpus conformance roundtrip tests

## Documentation

- `README.md`: project overview and benchmarks
- `docs/design.md`: transport design
- `docs/package-schema.md`: canonical table contract
- `docs/cityjson-arrow.md`: public API reference
- `docs/cityjson-arrow-ipc-spec.md`: binary stream layout
- `docs/cityjson-parquet-spec.md`: persistent package binary layout
- `docs/adr/`: architecture decision records

## Development Notes

- The canonical package schema id is `cityjson-arrow.package.v3alpha2`.
- Keep docs aligned with the code in `src/schema.rs` and `src/codec.rs`.
- Avoid introducing claims about formats, modules, or APIs that are not
  implemented in this repository.
