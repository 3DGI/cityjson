# Selection-Aware Shared-Vertex Reconstruction Implementation Plan

## Status

Proposed. Benchmark work may begin immediately. Production cache removal is
blocked until the random-access decision in
[ADR-010](../../crates/cityjson-index/docs/adr/010-selection-aware-shared-vertex-reconstruction.md)
is resolved.

## Goal

Make `cityjson-json` the sole owner of CityJSON selection, vertex-reference
discovery, index remapping, validation, and typed model construction.

`cityjson-index` should locate and group source bytes without interpreting
geometry JSON. Same-source package batches should share semantic preparation
and vertex resolution, while scalar reads should retain acceptable cold and
warm latency.

Breaking changes are intentional. Do not preserve the existing staged API,
index-local localization logic, or tests whose only purpose is legacy
compatibility.

## 1. Establish the performance baseline first

Refactor the existing `cityjson-index` benchmark harness before changing the
read path. Do not add a separate Criterion target or new correctness test
functions.

Add reconstruction-only operation variants:

- cold scalar reads for packages at the first, middle, and last source
  positions, opening a fresh `CityIndex` for each measurement;
- repeated warm reads of those same packages on one `CityIndex`;
- same-source batch sizes `1`, `16`, `256`, and `4096`, selecting references
  across the complete source rather than taking only the first records;
- a 256-request batch with 50% duplicate references;
- small, medium, large, and full bbox lookup-plus-materialization;
- one and four concurrent readers, each using an independent `CityIndex`;
- existing CityJSONSeq and feature-file scalar and 256-package batch reads as
  control cases.

Run every timing variant seven times after one unmeasured process and filesystem
warm-up. Record every observation plus median and median absolute deviation.
Cold means an empty decoder/cache state; it does not mean forcing the operating
system page cache cold.

Run memory variants in isolated child processes. Sample RSS during the measured
operation rather than reading RSS only after it, and report the operation-local
increment above the pre-operation process baseline. Measure:

- one fixed early package against the 1k, 5k, 25k, and full regular-CityJSON
  sources;
- the 256-package same-source batch;
- one and four independent concurrent readers of the full source.

Capture the baseline from the exact pre-refactor commit using the pinned
Basisvoorziening artifact, release mode, one machine, and the pinned Rust
toolchain. Store the raw baseline and a human-readable summary under
`crates/cityjson-index/docs/benchmarks/`, including the commit SHA, CPU, memory,
kernel, toolchain, corpus artifact, and command.

Add a comparison recipe to the crate justfile that accepts baseline and
candidate JSON files. It must:

- match rows by layout, dataset, source position, operation, variant, batch
  size, and concurrency;
- fail if any core candidate median is more than `1.10x` its baseline;
- fail if operation-local peak RSS exceeds the baseline by more than 5%, which
  is the allowed measurement tolerance;
- print unmatched or missing rows as errors rather than silently ignoring
  them;
- emit both machine-readable results and a Markdown summary.

The new performance scenarios are approved because the existing suite does not
control source position, cache warmth, batch size, concurrency, or
operation-local memory. This approval does not extend to new correctness test
functions.

## 2. Replace the staged JSON API

Replace the existing staged API family with a two-phase shared-vertex contract.
The first phase discovers semantic requirements; the caller resolves physical
coordinates; the second phase constructs typed models.

Expose concepts equivalent to:

```rust
pub struct CityObjectFragment<'a> {
    pub id: &'a str,
    pub object: &'a serde_json::value::RawValue,
}

pub struct SharedVertexFeature<'a> {
    pub id: &'a str,
    pub cityobjects: &'a [CityObjectFragment<'a>],
}

pub struct PreparedSharedVertexFeatures {
    // Private parsed fragments, prepared base context, and remap plans.
}

pub fn prepare_shared_vertex_features(
    features: &[SharedVertexFeature<'_>],
    base: &[u8],
) -> Result<PreparedSharedVertexFeatures>;

impl PreparedSharedVertexFeatures {
    pub fn required_vertex_indices(&self) -> &[usize];

    pub fn finish(
        self,
        coordinates: &[[f64; 3]],
    ) -> Result<Vec<OwnedCityModel>>;
}
```

`required_vertex_indices` must return a sorted, deduplicated list.
`coordinates[n]` must correspond to `required_vertex_indices()[n]`.
`finish` must reject a coordinate-count mismatch. Callers must not supply a
map whose iteration order can affect local vertex numbering.

Also expose one single-feature decoder and writer:

```rust
pub fn read_feature(
    feature: &[u8],
    base: &[u8],
    id_override: Option<&str>,
) -> Result<OwnedCityModel>;

pub fn write_feature(
    writer: impl std::io::Write,
    model: &OwnedCityModel,
) -> Result<()>;
```

Remove, rather than deprecate:

- `FeatureAssembly` and `FeatureObjectFragment`;
- assembly-based entry points;
- redundant `*_direct` and assumed-version aliases;
- the indexed-id staged alias;
- staged file-reading conveniences that only wrap byte reading;
- corresponding `cityjson-lib` re-exports and legacy examples.

Update all Rust, C, Python, C++, and WASM callers directly. Do not add
compatibility shims.

## 3. Implement semantic preparation in `cityjson-json`

Parse base metadata and resources once for the batch and create one reusable
prepared feature-model shell. Keep complete feature outputs owned and
independent.

Parse CityObject fragments without `serde_json::Value` materialization or
reserialization. During preparation:

- collect root vertex indices from every ordinary geometry boundary;
- collect the root reference point used by GeometryInstance placement;
- collect geometry-valued attributes such as `address.location`;
- keep root vertices, geometry-template vertices, material indices, texture
  indices, and texture UV indices as separate domains;
- prepare deterministic remap plans ordered by source vertex index;
- retain parent/child relationships whose endpoints are both in the package
  and omit external relationships;
- validate geometry shape and reference types through the same errors as the
  normal full-document reader.

During `finish`:

- apply the source transform exactly once;
- construct typed flat boundaries and geometry instances directly with local
  handles;
- construct geometry-valued attributes through the same geometry importer;
- reject missing, malformed, non-finite, or out-of-range coordinates;
- build one dense local root-vertex pool per returned feature;
- preserve input feature order.

Do not hardcode integer coordinates in the staged contract. The normal JSON
reader accepts numeric CityJSON coordinates, and selected reconstruction must
use the same numeric rules.

## 4. Rework regular-CityJSON reads in `cityjson-index`

Change `read_packages` to:

1. Deduplicate requested record IDs for physical decoding while retaining the
   original request sequence.
2. Fetch locations and package members once.
3. Group regular-CityJSON packages by source ID and shared vertex-array range.
4. Open each source once and read raw CityObject fragments by indexed range.
5. Call `prepare_shared_vertex_features` once for each source group.
6. Resolve the sorted requested vertex indices through the selected physical
   access strategy.
7. Call `finish` once and associate each model with its package record.
8. Restore original order and duplicate references by cloning complete output
   models only after unique decoding.

Route `read_package` through this implementation with a one-item input.

Keep CityJSONSeq and feature-file reads on `read_feature`. Remove the local
feature JSON mutation and wrapper-reserialization path; `id_override` owns the
indexed package-ID behavior.

Delete the index-local components once equivalent typed reconstruction is
passing:

- `LocalizedFeatureParts` and `LocalizedFeatureObject`;
- `build_feature_parts` and recursive JSON boundary rewriting;
- full shared-coordinate parsing from the package read path;
- the `LruCache` dependency if the selected access strategy does not use it.

Do not change package ordering, duplicate alignment, or package membership
semantics.

## 5. Resolve physical vertex access

Stop after the baseline and shared semantic API are available. Update ADR-010
with one accepted strategy before deleting the current coordinate cache.

Evaluate the alternatives against every reconstruction benchmark, with special
attention to repeated middle/late scalar reads:

### Candidate A: sparse sidecar checkpoints

This is the preferred candidate.

- During regular-CityJSON indexing, record byte offsets relative to the
  top-level vertex-array range at a fixed checkpoint stride.
- Store count, stride, and offsets in the source record using a compact
  versioned binary representation.
- Group requested indices by checkpoint interval, merge adjacent intervals,
  seek to those source ranges, and let `cityjson-json` parse only the selected
  coordinate chunks.
- Bump the schema version and mark older sidecars as requiring a transactional
  reindex. Do not implement an in-place legacy migration.
- Refactor the existing schema-version and rebuild tests to the new version.

The prototype stride must be selected from `64`, `256`, and `1024` using
sidecar size, cold scalar latency, 256-package batch throughput, and reindex
time. Choose the smallest stride whose sidecar increase is at most 2% of source
bytes and whose reindex time passes the 10% gate. Break remaining ties by lower
late-scalar median. Record the measurements and chosen representation in
ADR-010.

### Candidate B: bounded coordinate cache

Evaluate only if sparse checkpoints cannot pass the reindex or read gates.

- Use one process-wide byte budget, deterministic eviction, and no per-instance
  unbounded cache.
- Count decoded coordinate allocation size against the budget.
- Define concurrent miss behavior so one source is decoded once rather than
  once per waiter.
- Keep cache policy outside CityJSON semantic parsing.

This candidate is rejected if peak memory scales with the number of live
`CityIndex` instances or if required configuration changes public API behavior.

### Candidate C: streaming only

Select only with explicit follow-up approval to waive the warm/random scalar
10% gate. Without that approval, a streaming-only candidate that fails the
balanced gate is not acceptable.

The selected strategy must be used by both scalar and batch reads. Do not keep
multiple production paths selected by hidden heuristics.

## 6. Refactor existing correctness tests

No new correctness test functions are approved by this plan.

Refactor the existing staged-assembly test to exercise
`prepare_shared_vertex_features` and `finish`. Its fixture must continue to
cover overlapping selections and deterministic local indices. This test is
necessary to verify the replacement public contract, not legacy assembly
behavior.

Extend the existing regular-CityJSON localization fixture with:

- an `address.location` root vertex unused by ordinary geometry; and
- a GeometryInstance whose placement vertex is also unused by ordinary
  geometry.

This verifies that semantic reference discovery moved completely into
`cityjson-json`.

Refactor the existing same-source batch test to assert one semantic preparation
and one vertex-resolution request per source group while preserving request
order and duplicates.

Refactor existing malformed-input tests to cover missing and out-of-range
selected vertices. Delete assertions whose only purpose is the removed aliases,
synthetic wrapper implementation, JSON-DOM localizer, or coordinate cache.

If a required correctness case cannot fit an existing test, stop and request
explicit approval before adding a test. Explain the uncovered invariant and why
none of the existing tests can be refactored to contain it.

## 7. Documentation and acceptance

Update ADR-009 to mark it superseded by ADR-010 once the new API and physical
access decision are implemented. Move ADR-010 from Proposed to Accepted and
record:

- the chosen random-access strategy;
- schema/reindex consequences;
- final API names;
- baseline and candidate benchmark artifacts;
- any approved performance exception.

Update crate documentation and changelogs for the deliberate breaking API and,
if applicable, required sidecar rebuild. Do not document removed entry points as
deprecated alternatives.

Implementation is accepted only when:

- the dedicated reconstruction benchmark comparison passes the balanced gate;
- `just ci` passes;
- `just ffi ci` passes;
- no new `#[allow(clippy::...)]` attributes were added;
- no unapproved correctness test functions were added;
- retained reconstruction memory no longer scales with complete decoded source
  coordinates multiplied by live readers.
