# Implementation Plan

This document now tracks the post-`tyler` dogfood phase of the rewrite.
The facade boundary work and JSON boundary work are complete, and the
`tyler` migration has been committed and released as `tyler` 0.4.0.

## Completed

### Facade Boundary

- `CityModel` is a thin owned wrapper over `cityjson::v2_0::OwnedCityModel`
- `as_inner`, `as_inner_mut`, `into_inner`, `AsRef`, and `AsMut` are the
  explicit boundary helpers
- `Deref` and `DerefMut` are not used to blur the facade boundary
- the crate root stays small and explicit

### JSON Boundary

- `cjlib::json` owns document parsing, feature parsing, feature-stream reading,
  feature-stream writing, and serialization
- `json::from_file` stays document-oriented
- `.jsonl` files are not silently treated as documents
- unsupported or unfinished branches fail loudly

### Tyler Dogfood

- `tyler` now reads CityJSON through `cjlib`
- feature loading uses the base-aware `json::from_feature_file_with_base`
- Tyler no longer owns its own CityJSON serde structs
- ownership scoring parity with the legacy implementation has been restored
- the migration was released as `tyler` 0.4.0

## Remaining `cjlib` Work

The remaining work is about turning the current façade into a stable production
surface, not about resurrecting legacy CityJSON parsing.

### 1. Replace Placeholder Workflow Modules

`ops`, `arrow`, and `parquet` still contain obvious placeholders.
Those modules should either grow real implementations or remain explicit
`todo!()` markers until they do.

### 2. Add Workload Benchmarks

The Tyler dogfood run showed that the release build is usable and memory
efficient. The next step is to make those workloads measurable inside the
`cjlib` repository itself.

Benchmark targets:

- metadata load
- single feature parse
- feature scan used for cell ownership
- directory-tree feature walk

### 3. Stabilize The FFI Boundary

Once the Rust surface stops moving, add the narrow FFI layer on top of the
proven API instead of speculating about a broader foreign-language surface.

## Current Guidance

- do not reintroduce a second CityJSON model
- do not add Tyler-specific view layers
- improve the main `cjlib` API when dogfooding exposes real friction
- keep the implementation honest with tests and benchmarks
