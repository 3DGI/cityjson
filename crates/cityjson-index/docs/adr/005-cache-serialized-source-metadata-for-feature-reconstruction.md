# Cache Serialized Source Metadata for Feature Reconstruction

## Status

Accepted

## Date

2026-03-31

## Context

`cityjson-index` reconstructs feature packages by combining:

- indexed feature bytes or fragments from the storage backend
- source-level base metadata from the SQLite `sources.metadata` column

That metadata is needed on every read path:

- `get_with_metadata()`
- `query_iter_with_ids()`
- `query_iter_with_metadata()`
- `iter_all_with_ids()`
- `iter_all_with_metadata()`
- `read_feature()`

The existing metadata cache stored only the parsed `serde_json::Value`.
That avoided repeated SQLite reads and repeated JSON parsing, but the backend
read path still had to serialize that parsed value back into bytes for every
feature reconstruction.

That repeated `serde_json::to_vec(...)` work is not architecturally meaningful.
It is avoidable CPU and allocation churn on a hot path where the metadata bytes
are stable for the lifetime of the indexed source.

ADR 002 already established indexed byte-range reads as the correct I/O
baseline, and it kept a shared-vertices cache for regular `CityJSON`. This
change addresses a different remaining inefficiency: repeated metadata
serialization after the bytes have already been persisted and loaded once.

## Decision

`cityjson-index` will cache source metadata in two forms at the same time:

- parsed JSON as `Arc<Meta>` for APIs that return metadata to callers
- serialized bytes as `Arc<[u8]>` for backend feature reconstruction

The internal storage-backend contract will take cached metadata bytes directly
instead of a parsed metadata value.

Public API behavior remains the same:

- methods that expose metadata still return `Arc<Meta>`
- feature reconstruction still uses the same staged `cityjson-lib` helpers
- no on-disk schema changes are required

## Implementation

The implementation lives in
[/home/balazs/Development/cityjson-index/src/lib.rs](/home/balazs/Development/cityjson-index/src/lib.rs).

Key points:

- add an internal `CachedMetadata` struct with:
  - `value: Arc<Meta>`
  - `bytes: Arc<[u8]>`
- change `Index.metadata_cache` from `HashMap<i64, Arc<Meta>>` to
  `HashMap<i64, CachedMetadata>`
- add `Index::get_cached_metadata(source_id)` to populate both forms from the
  stored SQLite JSON string
- keep `Index::get_metadata(source_id)` as a compatibility wrapper that returns
  only the parsed value
- change `StorageBackend::read_one(...)` to accept `Arc<[u8]>`
- update `NdjsonBackend`, `CityJsonBackend`, and `FeatureFilesBackend` to pass
  cached metadata bytes straight into staged reconstruction
- update tests to construct metadata bytes explicitly where they call
  `read_one(...)` directly

## Consequences

### Positive

- repeated feature reads avoid reserializing source metadata on every hit
- the change keeps the public metadata-returning APIs stable
- the backend contract now matches what reconstruction actually consumes: bytes
- the improvement applies uniformly across `NDJSON`, regular `CityJSON`, and
  feature-files

### Negative

- the in-memory metadata cache is slightly larger because it keeps both parsed
  JSON and serialized bytes
- the internal backend API becomes a little less ergonomic for tests and direct
  callers because it now expects bytes instead of a parsed value

### Neutral tradeoff

We are deliberately spending a small amount of extra cache memory to remove
repeated serialization work from a hot steady-state read path.
