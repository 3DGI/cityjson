# Plan for Getting `cjindex` to Runnable Integration Tests

## Goal

Take `cjindex` from its current scaffold state to the first point where the
real integration tests can run against `CityIndex` end to end.

That means:

- `reindex()` must populate a real index
- `get()` must resolve a feature by ID
- `query()` and `query_iter()` must resolve features by bbox
- `metadata()` must return cached per-source metadata
- each supported storage layout must be testable through the same public API

This plan is about reaching a runnable and defensible first end-to-end path,
not about optimizing every code path before correctness exists.

## Current State

The main blockers are all in [src/lib.rs](/home/balazs/Development/cjindex/src/lib.rs):

- the SQLite-backed `Index` is still a stub
- `CityIndex::{reindex,get,query,query_iter,metadata}` are still stubs
- `scan()` is still unimplemented for all three backends
- `read_one()` is implemented only for regular `CityJSON`
- the fixture prep pipeline already exists in
  [tests/common/data_prep.rs](/home/balazs/Development/cjindex/tests/common/data_prep.rs)

The important consequence is that the next work is no longer about cross-crate
JSON API design. It is about making the core `cjindex` index lifecycle real.

## Recommended Strategy

Do not try to finish every backend at once.

Use one vertical slice to prove the public API and index schema first, then
extend it backend by backend.

Recommended order:

1. feature-files vertical slice
2. regular `CityJSON`
3. `NDJSON` / `CityJSONSeq`

Why this order:

- feature-files is the simplest indexing shape
- regular `CityJSON` already has the hardest `read_one()` path implemented
- `NDJSON` is conceptually simple once the index core is real

## Phase 1: Make the Index Real

Before any backend-specific work, implement the SQLite core in
[src/lib.rs](/home/balazs/Development/cjindex/src/lib.rs).

### Required schema

Implement the tables already described in
[README.md](/home/balazs/Development/cjindex/README.md):

- `sources`
- `features`
- `feature_bbox`
- `bbox_map`

At minimum, the schema must support:

- mapping a feature ID to one `FeatureLocation`
- mapping a bbox hit back to one or more `FeatureLocation`s
- storing per-source metadata JSON
- storing regular-`CityJSON` shared-vertices byte ranges

### Required `Index` methods

Implement:

- `Index::open`
- `Index::insert_source`
- `Index::insert_features`
- `Index::lookup_id`
- `Index::lookup_bbox`
- `Index::get_metadata`
- `Index::clear`

Design goal:

- keep the first version small and explicit
- prioritize correctness and debuggability over clever abstractions

### Suggested simplifications

For the first pass:

- keep one transaction around bulk reindex writes
- use JSON text for cached metadata
- do not optimize metadata cache invalidation yet
- do not implement incremental updates

## Phase 2: Finish the `CityIndex` Public Lifecycle

Once `Index` is real, wire the top-level API in
[src/lib.rs](/home/balazs/Development/cjindex/src/lib.rs).

Implement:

- `CityIndex::reindex`
- `CityIndex::get`
- `CityIndex::query`
- `CityIndex::query_iter`
- `CityIndex::metadata`

### Expected behavior

`reindex()` should:

- clear existing index state
- call the selected backend `scan()`
- insert one `sources` row per source
- insert one `features` row per scanned feature
- populate bbox lookup tables

`get()` should:

- resolve one `FeatureLocation` via `lookup_id`
- dispatch to backend `read_one()`

`query()` should:

- resolve matching `FeatureLocation`s via `lookup_bbox`
- read all matching features eagerly

`query_iter()` should:

- resolve matching locations first
- lazily call backend `read_one()` during iteration

`metadata()` should:

- return cached metadata entries from indexed sources

## Phase 3: Feature-Files Vertical Slice

Implement the simplest backend first in
[src/lib.rs](/home/balazs/Development/cjindex/src/lib.rs).

### `FeatureFilesBackend::scan`

Implement:

- metadata file discovery using the configured glob
- feature file discovery using the configured glob
- nearest-ancestor metadata resolution
- one `SourceScan` per metadata root or resolved source grouping
- one `ScannedFeature` per feature file

### `FeatureFilesBackend::read_one`

Implement:

- read full feature file bytes
- deserialize through `cjlib::json::from_feature_slice_with_base` or the
  simpler feature-file path if no base merge is needed
- attach metadata from the indexed source if necessary

### Why this slice matters

This is the fastest way to prove:

- the SQLite schema is usable
- the top-level `CityIndex` API is viable
- the integration-test harness can exercise real indexing and querying

## Phase 4: First Runnable Integration-Test Milestone

As soon as the feature-files slice works, add or upgrade integration tests so
they exercise the real public API instead of only checking fixture shape.

Use the existing fixture prep in
[tests/common/data_prep.rs](/home/balazs/Development/cjindex/tests/common/data_prep.rs).

### Minimum new integration coverage

For feature-files:

- prepare fixtures into a temp output root
- build an index file in a temp directory
- call `reindex()`
- call `get()` for one known feature ID
- call `query()` for a bbox known to hit at least one feature
- call `query_iter()` and collect results
- call `metadata()` and verify at least one metadata object is returned

At this point, the first real integration test can run.

This is the first milestone worth landing before extending the other layouts.

## Phase 5: Regular `CityJSON`

After the feature-files path is proven, finish regular `CityJSON`.

### `CityJsonBackend::scan`

Implement:

- top-level metadata extraction
- shared `vertices` byte-range detection
- per-`CityObject` byte-range detection
- bbox computation from shared vertices plus transform

Important note:

- `CityJsonBackend::read_one` is already implemented
- the missing work is discovery and indexing, not subset materialization

### Integration target

Add the same public-API integration shape as feature-files:

- `reindex()`
- `get()`
- `query()`
- `query_iter()`
- `metadata()`

The semantic result should match the one-object contract already tested at unit
level.

## Phase 6: `NDJSON` / `CityJSONSeq`

Finish the remaining backend last.

### `NdjsonBackend::scan`

Implement:

- read first line as metadata
- record byte offsets and lengths of subsequent feature lines
- compute bbox from feature-local vertices plus metadata transform

### `NdjsonBackend::read_one`

Implement:

- seek to the feature byte range
- deserialize the feature line
- combine with cached metadata if needed

This backend should be straightforward once the index core and integration
pattern already exist.

## Testing Plan

## 1. Keep unit tests for narrow mechanics

Keep the current unit tests in
[src/lib.rs](/home/balazs/Development/cjindex/src/lib.rs) for:

- local vertex remap
- dangling relation filtering
- regular `CityJSON` one-object reconstruction

Those are fast and isolate tricky behavior.

## 2. Upgrade integration tests to public-API tests

The meaningful integration tests should target `CityIndex`, not backend helper
functions.

For each layout:

- build fixtures with the existing prep helper
- build a fresh SQLite index file
- run `reindex()`
- exercise `get()`
- exercise `query()`
- exercise `query_iter()`
- exercise `metadata()`

## 3. Delay benches until integration is real

Do not spend more time on benches until the public API path works.

Once the three layouts pass integration:

- rerun the existing benches
- measure indexing time separately from query time
- compare layout differences only after correctness is locked down

## Explicit Non-Goals

This plan does not require:

- incremental index updates
- mmap-based read optimization
- zero-copy JSON scanning
- grouped-subgraph extraction semantics
- cross-crate semantic refactors into `cityjson-rs`

Those may become worthwhile later, but they are not the blockers for runnable
integration tests.

## Acceptance Criteria

This plan is complete when all of the following are true:

- the SQLite-backed `Index` is implemented
- `CityIndex::{reindex,get,query,query_iter,metadata}` work end to end
- feature-files integration tests run through the public API
- regular `CityJSON` integration tests run through the public API
- `NDJSON` integration tests run through the public API
- the existing benches can run against those working code paths

## Recommended Immediate Next Step

Start with the feature-files vertical slice.

That is the shortest path to a real integration test because:

- it avoids shared-vertices indexing complexity
- it avoids `NDJSON` line-offset handling complexity
- it proves the public API and SQLite shape first

Once that slice is green, the remaining work becomes backend extension rather
than core architecture uncertainty.
