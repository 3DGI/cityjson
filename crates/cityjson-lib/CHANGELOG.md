# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and this project follows Semantic Versioning.

## [Unreleased]

### Changed

- `ops::append` and `ops::merge` now accept differing source transforms and
  reconcile them into the merged result instead of rejecting the merge.
  Identical transforms are preserved, mixed transforms clear the merged model
  transform, and transform-free inputs stay transform-free.

- Replace the Rust `ops::filter` and `filter_with_options` APIs with opaque `ModelSelection`, `select_cityobjects`, `select_geometries`, and `extract`.
- Keep `CityJSONFeature` roots valid after `subset` and selection-driven extraction shrink a model.
- Reroot surviving feature subsets to a parentless `CityObject` when the original root is removed.
- Return a model error when feature shrinking removes the root and no replacement root exists.
- Keep the JSON boundary strict for malformed feature packages.
- Replace float-exact assertions in the test suite with tolerant comparisons so strict clippy checks pass.
- Removed the local `cityjson-export` crate from the repository and dropped the benchmark helper path that depended on it.
- Restored `just ci` to a Rust-only validation path and kept the C++/Python/native validation under `just ffi *`.

## [0.6.1] - 2026-04-22

### Changed

- Bumped the direct `cityjson-rs` dependency to `0.7.2` and aligned the optional format crate pins to the lockstep release train.
- Bumped the package version to `0.6.1`.
- Aligned the Python package metadata with `0.6.1` so the PyPI release workflow publishes the same version as the Rust crate.
- Shared CityObject result rebuilding between `ops::subset` and predicate filtering so retained parent/child references are remapped and references to removed CityObjects are stripped.

### Added

- Rust-only `cityjson_lib::ops::subset`, `append`, and `merge` operations for native CityModel subsetting and model combination workflows.
- Rust-only `cityjson_lib::ops::ModelSelection`, `select_cityobjects`, `select_geometries`, and `extract` APIs for predicate-driven selection and reconstruction workflows.

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
