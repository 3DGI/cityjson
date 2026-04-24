# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and this project follows
Semantic Versioning at the workspace level: every release bumps every
crate in the workspace to the same version.

## [Unreleased]

### Added
- Added a `cityjson-index` aggregate feature-bounds summary API for callers that
  need whole-index bounds and feature counts without scanning feature pages.

### Changed
- Consolidated the seven `cityjson-*` Rust crates (`cityjson`,
  `cityjson-json`, `cityjson-arrow`, `cityjson-parquet`, `cityjson-lib`,
  `cityjson-fake`, `cityjson-index`) plus the two FFI core crates
  (`cityjson-lib-ffi-core`, `cityjson-index-ffi-core`) and the
  `cityjson-lib-wasm` shim into a single Cargo workspace at
  [`3DGI/cityjson`](https://github.com/3DGI/cityjson). All crates move
  to `shared-version = true` and bump together via
  `cargo release --workspace`.
- Unified the per-crate `CITYJSON_*_SHARED_CORPUS_ROOT` environment
  variables into a single `CITYJSON_SHARED_CORPUS_ROOT`.
- Bumped all workspace crates to `0.8.0` to mark the transition — this
  is a higher version than any pre-merge crate held.
- Crate names on crates.io and Python package names on PyPI are
  unchanged; only source repository URLs have moved.
- Optimized `cityjson-index` full-index page scans so later pages use a direct
  `features.id` range scan instead of a nullable paging predicate.

---

Per-crate history prior to the workspace merge is preserved in each
crate's former `CHANGELOG.md` inside `crates/<name>/CHANGELOG.md` and in
the full git history (line-level `git log` / `git blame` continues to
work across the move).
