# ADR: Full Integration Benchmark Harness Across All Storage Layouts

## Status

Accepted

## Date

2026-03-29

## Context

`cjindex` supports three storage layouts:

- feature-files
- regular `CityJSON`
- `NDJSON` / `CityJSONSeq`

Before this change, the benchmark story was inconsistent:

- [benches/ndjson.rs](/home/balazs/Development/cjindex/benches/ndjson.rs)
  benchmarked the real `CityIndex` API
- [benches/cityjson.rs](/home/balazs/Development/cjindex/benches/cityjson.rs)
  only benchmarked raw JSON parsing
- [benches/feature_files.rs](/home/balazs/Development/cjindex/benches/feature_files.rs)
  only benchmarked raw JSON parsing

That meant the project did not have one answer to the question "how fast is
`cjindex`?" Different layouts were measuring different things.

This also made performance interpretation weak:

- NDJSON numbers included indexing and lookup behavior
- CityJSON and feature-files numbers mostly reflected `serde_json` costs
- fixture shapes were not aligned, so the results were not directly comparable

We needed one integration benchmark suite that exercised the same public API
operations across all three layouts in release mode.

## Decision

We added a shared integration benchmark harness in
[benches/support.rs](/home/balazs/Development/cjindex/benches/support.rs)
and converted all three benchmark entry points to thin wrappers around that
shared harness.

The benchmark suite now measures the same five public operations for each
layout:

- `reindex`
- `get`
- `query`
- `query_iter`
- `metadata`

The benchmark harness derives all three storage layouts from the same small
raw-input subset, so the layouts are measured against comparable data rather
than unrelated fixture shapes.

## Implementation

### 1. Shared harness

The main implementation lives in
[benches/support.rs](/home/balazs/Development/cjindex/benches/support.rs).

It introduces:

- `LayoutKind`
- `bench_layout(c: &mut Criterion, kind: LayoutKind)`

That helper owns the common Criterion logic for all five operations. The
per-layout bench files only select the storage layout.

### 2. Fixture preparation

The harness materializes one benchmark subset from the raw input dataset under:

- `/home/balazs/Data/3DBAG_3dtiles_test/input`

It then derives all three benchmark layouts from that same subset by reusing
the existing fixture-prep logic in
[tests/common/data_prep.rs](/home/balazs/Development/cjindex/tests/common/data_prep.rs).

The final fixture contract is:

- 3 tiles
- 3 feature files per tile
- 9 features total

Per storage layout, that materializes to:

- feature-files: 9 files, 9 features total
- CityJSON: 3 files, 9 features total
- NDJSON: 3 files, 9 features total

This was an important correction during implementation. The first harness
version selected only one feature per tile, which made the derived CityJSON and
NDJSON files degenerate single-feature tiles. That would have under-measured
the real cost of tiled one-object extraction. The harness was corrected to keep
multiple features per tile.

### 3. Shared operation timing

For each layout, the harness:

1. prepares the derived layout root
2. builds a fully indexed `CityIndex` for warm steady-state benches
3. chooses one stable feature id from the subset
4. computes one stable bbox that covers the selected features
5. benchmarks the five operations using identical timing structure

The `reindex` bench uses a fresh empty SQLite index per iteration so it
measures actual rebuild cost. The read-path benches reuse a populated index so
they measure steady-state lookup behavior rather than setup.

### 4. Thin per-layout entry points

The three bench files are now intentionally small:

- [benches/feature_files.rs](/home/balazs/Development/cjindex/benches/feature_files.rs)
- [benches/cityjson.rs](/home/balazs/Development/cjindex/benches/cityjson.rs)
- [benches/ndjson.rs](/home/balazs/Development/cjindex/benches/ndjson.rs)

Each one just imports `mod support;` and calls `bench_layout(...)` with the
appropriate `LayoutKind`.

This removes duplicated timing code and avoids benchmark drift between layouts.

## Consequences

### Positive

- `cjindex` now has one consistent release-mode integration benchmark suite.
- The three layouts are measured through the same public API surface.
- Benchmark output is directly comparable across layouts.
- The harness is small enough to run routinely during development.
- The results are much more useful for design decisions than the old parse-only
  CityJSON and feature-files benches.

### Negative

- The benchmark harness now depends on an external local dataset path.
- Benchmarks do more setup work than the previous micro-benches.
- The shared support module duplicates a small amount of test-helper logic,
  especially around model-to-bbox derivation and fixture selection.

### Neutral tradeoff

We chose realistic, comparable integration benchmarks over pure parsing
micro-benchmarks. If raw parse benchmarks are still useful later, they should
be added back explicitly as a separate benchmark class, not mixed into the main
integration suite.

## Results and Interpretation

The benchmark run and exact numbers are recorded in
[docs/cjindex-full-integration-benches-results.md](/home/balazs/Development/cjindex/docs/cjindex-full-integration-benches-results.md).

In simple terms, the implementation showed:

- indexing is in the same range across all three layouts
- feature-files and NDJSON are very close on warm reads
- CityJSON is the clear steady-state read-path outlier

Observed release-mode timings on the shared subset:

- `feature_files_reindex`: `4.3400 ms` to `4.4566 ms`
- `cityjson_reindex`: `4.5582 ms` to `4.6971 ms`
- `ndjson_reindex`: `4.2319 ms` to `4.3517 ms`
- `feature_files_get`: `72.957 us` to `73.153 us`
- `cityjson_get`: `95.515 us` to `95.737 us`
- `ndjson_get`: `72.189 us` to `72.293 us`
- `feature_files_query`: `915.04 us` to `916.08 us`
- `cityjson_query`: `2.8397 ms` to `2.8438 ms`
- `ndjson_query`: `904.48 us` to `906.02 us`

The likely explanation is architectural:

- feature-files and NDJSON start from feature-shaped payloads
- CityJSON has to extract one object out of a multi-feature tile and rebuild a
  one-object feature package on reads

That makes CityJSON the next obvious optimization target.

## Follow-up

The next performance work should focus on the CityJSON read path, not on
benchmark coverage. The benchmark harness now provides a reliable baseline for
measuring those future improvements.
