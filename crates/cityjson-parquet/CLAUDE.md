# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with
code in this repository.

## Project Overview

`cityjson-parquet` is the persistent package crate in the `cityjson-rs`
ecosystem. It writes `cityjson::v2_0::OwnedCityModel` values into seekable
single-file packages and reads them back.

The semantic unit remains `OwnedCityModel`. The package format uses canonical
Arrow IPC table payloads arranged with a footer-located manifest so the writer
never needs a backwards seek pass.

## Key Commands

- Build: `cargo build`
- Test: `cargo test`   (requires `../cityjson-arrow` as a sibling directory)
- Check: `just check`
- Clippy / lint: `just lint`
- Format: `cargo fmt`

## Architecture

Current source layout:

- `src/lib.rs`: public re-exports — schema and manifest types forwarded from
  `cityjson_arrow::schema`, plus `PackageReader` and `PackageWriter` from the
  `package` module
- `src/package/mod.rs`: `PackageWriter`, `PackageReader`, `PackageSink`,
  `write_package_model_file`, `write_package_parts_file`,
  `read_package_model_file`, `read_package_parts_file`,
  `read_package_manifest`, and all internal serialization helpers
- `src/spatial.rs`: `SpatialIndex`, `SpatialEntry`, `BBox2D`, Hilbert curve
  index and query utilities

## Dependencies

- `cityjson-arrow` via `path = "../cityjson-arrow"` — provides the canonical
  Arrow schema, manifest types, `IncrementalDecoder`, `CanonicalTableSink`,
  and doc-hidden build/emit helpers
- `cityjson` via `path = "../cityjson-rs"` — the semantic model
- `arrow`, `arrow-array`, `arrow-ipc`, `arrow-schema` — Arrow IPC file reader
  and writer
- `memmap2` — memory-mapped file access for zero-copy payload slicing
- `serde_json` — manifest serialisation

Dev only:

- `cityjson-json` — for test fixture parsing
- `tempfile` — temporary directories in roundtrip tests
- `serde` — derive macros used in test helpers

## Testing

All integration tests live in `tests/`.

`tests/package_shared_corpus_roundtrip.rs` includes
`../../tests/support/shared_corpus.rs` from `../cityjson-arrow`. Both repos
must be checked out as siblings for the tests to compile. The crate does not
declare a `[workspace]`, so `--workspace` is not meaningful here; use
`cargo test` directly.

`src/spatial.rs` contains unit tests for the Hilbert curve and bounding box
geometry.

## Documentation

- `README.md`: project overview, architecture summary, and verification commands
- `docs/`: user-facing documentation (served via `properdocs` / `just site-serve`)

## Development Notes

- the package magic bytes are `CITYJSON_ARROW_PKG_V3\0` and the footer magic
  is `CITYJSON_ARROW_PKG_V3IDX\0`; changing either breaks all existing files
- the package schema id is `cityjson-arrow.package.v3alpha3`; it is embedded in
  the manifest by `cityjson_arrow::schema` and must stay in sync with
  `cityjson-arrow`
- `SpatialIndex` lives in `pub mod spatial` and is not re-exported at the crate
  root; reference it as `cityjson_parquet::spatial::SpatialIndex`
- avoid introducing any public API that duplicates schema or manifest types
  already exported by `cityjson-arrow`
