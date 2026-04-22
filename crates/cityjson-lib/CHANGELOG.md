# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and this project follows Semantic Versioning.

## [Unreleased]

### Added

- Rust-only `cityjson_lib::ops::subset`, `append`, and `merge` operations for native CityModel subsetting and model combination workflows.
- Rust-only `cityjson_lib::ops::filter` and `filter_with_options` APIs for predicate-based CityObject filtering, with optional recursive parent/child relative inclusion.

### Changed

- Shared CityObject result rebuilding between `ops::subset` and predicate filtering so retained parent/child references are remapped and references to removed CityObjects are stripped.

## [0.6.0] - 2026-04-20

### Added

- `arrow` module (`cityjson_lib::arrow`, feature `arrow`) for reading and writing
  the Arrow IPC streaming format via `cityjson-arrow`.
- `parquet` module (`cityjson_lib::parquet`, feature `parquet`) for reading and
  writing Parquet package files and dataset directories via `cityjson-parquet`.
- Arrow and Parquet I/O exposed through the C ABI (`ffi/core`), the C++ wrapper,
  and the Python bindings.
- Python sdist now bundles `cityjson-arrow` and `cityjson-parquet` sibling crates
  and rewrites their path references so the sdist is self-contained.

## [0.4.0] - 2026-04-16

### Added

- Arrow-first facade and binding flows that route columnar export through `cityjson-arrow`.
- Updated ADRs and public API docs describing the thinner vNext facade.

### Changed

- `CityModel` is now a direct re-export of `cityjson::v2_0::OwnedCityModel`.
- Removed the wrapper-model compatibility layer in favor of the minimal facade contract.
- Updated JSON, Arrow, FFI, examples, and tests to work directly against the re-exported model type.
