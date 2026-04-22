# Changelog

## 0.4.0

- Removed benchmark binaries, Criterion harnesses, and benchmark-only test corpus preparation from CI and the test harness.
- Replaced generated benchmark data with small tracked correctness fixtures for CityJSON, CityJSONSeq/NDJSON, and feature-file layouts.
- Upgraded `cityjson-lib` to 0.6.0 while keeping only the JSON feature enabled for `cityjson-index` and its FFI core.
- Scoped CI formatting and validation to correctness targets and removed Arrow/Parquet/Criterion from the `cityjson-index` dependency graph.
- Fixed the Python binding validation path to build a temporary JSON-only `cityjson-lib` wheel for tests.
- Replaced the GitHub Actions Rust toolchain action with direct `rustup` installation to avoid action archive download failures.

## 0.4.1

- Bumped the package version to `0.4.1`.
- Aligned `cityjson-lib` with `0.6.1` for the release train.

## 0.3.1

- Maintenance release for the initial public package metadata and release workflow.

## 0.3.0

- First public release of the `cityjson-index` crate.
- Ships the `cjindex` CLI for dataset inspection, indexing, and queries.
- Packages the public docs and release metadata for a first public GitHub/crates.io release.
