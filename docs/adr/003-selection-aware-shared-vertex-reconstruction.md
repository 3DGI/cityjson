# ADR-010: Selection-Aware Shared-Vertex Reconstruction

## Status

Proposed. The semantic ownership and batching direction are selected, but the
random-access strategy for repeated scalar reads remains unresolved.

## Date

2026-07-29

## Context

Regular CityJSON sources store one top-level `vertices` array shared by many
CityObjects. `cityjson-index` indexes each root CityObject package and its
member fragments, then reconstructs an owned `CityJSONFeature` when a package
is read.

The current reconstruction path splits CityJSON semantics across two crates:

- `cityjson-index` parses CityObject fragments through `serde_json::Value`,
  loads the complete shared vertex array, discovers references in ordinary
  geometry boundaries, builds a local vertex array, rewrites indices, and
  serializes the fragments again;
- `cityjson-json` reparses those localized fragments and constructs the typed
  `OwnedCityModel`.

The index-local rewrite has an incomplete view of the `cityjson-types` model.
Root vertices are not referenced only by ordinary geometry boundaries. They
can also be referenced by GeometryInstance placement and by typed geometry
values such as `address.location`. Template vertices and texture UVs are
separate index spaces and must not be remapped as root vertices. Keeping this
knowledge in both crates makes it likely that a new or less common typed
geometry path is handled by the JSON importer but omitted by the index
localizer.

The current `CityJsonBackend` also keeps an unbounded, per-instance
`LruCache<PathBuf, Arc<Vec<[i64; 3]>>>`. It avoids reparsing a source's
vertices after the first package read, but memory grows with every source
touched by each live `CityIndex`. A process-wide bounded coordinate cache, as
proposed in [cityjson-rs PR #17], limits that growth but does not remove the
duplicated CityJSON semantics or the full-array representation.

[cityjson-rs PR #17]: https://github.com/3DGI/cityjson-rs/pull/17

## Proposed Decision

Move selection-aware reconstruction into `cityjson-json`.

`cityjson-index` should remain responsible for physical location and grouping:

- locate indexed CityObject fragments and source metadata;
- group requested packages by regular CityJSON source;
- preserve requested order and duplicate package references;
- provide access to the indexed source vertex array.

`cityjson-json` should own semantic reconstruction:

- inspect raw CityObject fragments without a JSON DOM round trip;
- collect every root-vertex reference represented by the typed CityJSON model,
  including ordinary boundaries, GeometryInstance placement, and
  `address.location`;
- keep root vertices, template vertices, and texture UVs as distinct index
  spaces;
- union selected root indices across a same-source batch;
- build deterministic dense local vertex pools ordered by source index;
- remap indices while constructing typed geometries;
- retain relationships within each selected package and discard relationships
  to CityObjects outside it;
- validate malformed coordinates and missing or out-of-range references through
  the normal JSON error path.

The staged JSON API should be replaced rather than extended with compatibility
aliases. It should expose one single-feature decoder for CityJSONSeq and
feature-file packages and one shared-vertex batch decoder for regular CityJSON.
The redundant direct, assumed-version, indexed-id, and assembly entry points
should be removed. All Rust and FFI callers should move directly to the new
surface.

For regular CityJSON, `read_packages` should decode all requested packages from
one source as one selection batch. `read_package` should use the same path with
a one-item batch. The index-local `serde_json::Value` localization helpers,
full coordinate cache, and `lru` dependency should then be removed.

## Discovered Limitation: A JSON Array Is Not Random-Access Storage

The proposed semantic split does not by itself provide efficient random vertex
access.

When the batch decoder receives only a bounded sequential reader for the
top-level `vertices` array, it must parse every coordinate before the highest
selected source index. On the pinned Basisvoorziening benchmark source, that
array contains 1,585,691 vertices. A package whose vertices occur near the end
of the array therefore requires an almost complete scan even if the resulting
feature uses only a few vertices.

Batch reads amortize this scan across all packages from the source. Repeated
scalar reads do not. Routing `read_package` through a one-item streaming batch
can repeatedly perform work proportional to the source vertex count:

```text
current cached path:
    first read       O(source vertices)
    later reads      O(selected vertices)

streaming-only path:
    every read       O(vertices preceding the highest selected index)
```

This is a time-versus-retained-memory tradeoff, not an implementation detail
that can be optimized away inside a sequential JSON parser. Removing the
coordinate cache without another access strategy can substantially regress
warm or random scalar reads even while greatly reducing peak memory.

The existing benchmark suite cannot reliably quantify that tradeoff:

- the JSON Criterion suite measures full-document `read_model`, not staged
  shared-vertex reconstruction;
- the index harness samples the first 256 package references rather than
  first, middle, and late source positions;
- scalar and batch reads run after materializing bbox queries on the same
  `CityIndex`, so cold and warm state are mixed;
- each operation is timed once per harness invocation;
- RSS is sampled after operations and `VmHWM` is process-lifetime, not an
  operation-local peak;
- neither `just ci` nor `just ffi ci` executes a performance comparison.

The refactor must therefore not claim performance acceptance from the current
suite alone.

## Random-Access Alternatives

### 1. Sparse persistent vertex checkpoints

During indexing, record a compact byte-offset checkpoint every fixed number of
vertices. Store the checkpoint directory in the sidecar and use it to seek near
the selected indices, parsing only the required chunks through
`cityjson-json`.

This keeps coordinates uncached and makes access proportional to selected
chunks rather than the complete source. It adds index-build work, sidecar
storage, a schema-version bump, and a required reindex. The checkpoint stride,
binary representation, and JSON chunk interface still need to be decided and
benchmarked.

### 2. Bounded coordinate cache

Keep complete decoded coordinates for a bounded set of sources, with a shared
memory budget and eviction policy.

This preserves fast repeated scalar access for cached sources and is simpler
than persistent checkpoints. It retains the full-array memory representation,
introduces global cache policy and configuration, and only bounds rather than
removes the original scaling problem. It also leaves performance sensitive to
cache warmth and eviction.

### 3. Streaming only

Accept sequential vertex scans and optimize for grouped package reads.

This is the simplest architecture and has bounded retained memory, but random
and repeated scalar latency scales with source position. Choosing it requires
an explicit API-performance tradeoff and an exception to the scalar benchmark
gate; it must not be presented as performance-neutral.

No random-access alternative is selected by this ADR. Implementation of cache
removal is blocked until this choice is made from measured results.

## Performance Validation Required Before Acceptance

Capture a release-mode baseline at the exact pre-refactor commit and compare it
with the candidate on the same machine, toolchain, and pinned corpus. Refactor
the existing index benchmark harness to measure:

- cold scalar reads at early, middle, and late source positions using a fresh
  `CityIndex`;
- repeated warm reads of the same scalar packages;
- same-source batches across multiple cardinalities, including overlap and
  duplicate requests;
- materializing bbox queries from small through full-source result sets;
- concurrent readers using independent `CityIndex` instances;
- operation-local peak RSS as source size and reader concurrency increase;
- CityJSONSeq and feature-file reads as controls for the replaced
  single-feature staged API.

Use repeated measurements and compare medians. The agreed balanced gate rejects
a core workload whose median elapsed time is more than 10% slower than its
baseline. Operation-local peak RSS must not regress beyond measurement noise,
and the selected design must demonstrate that retained reconstruction memory
does not scale with complete source vertices multiplied by live readers.

These focused performance scenarios are approved additions. They exist because
no current benchmark reaches the new shared-vertex contract with controlled
source position, batch size, warmth, and concurrency.

## Consequences

### Positive

- CityJSON geometry semantics have one owner: `cityjson-json`.
- Index reconstruction covers all root-vertex references represented by the
  typed model rather than only ordinary boundaries.
- Same-source batch reads can collect and decode the union of selected vertices
  once.
- The index no longer needs to parse, mutate, and reserialize CityObject JSON.
- Breaking the staged API removes redundant entry points and compatibility
  logic.

### Negative

- The refactor spans `cityjson-json`, `cityjson-lib`, `cityjson-index`, and
  their FFI callers.
- A streaming-only implementation cannot preserve current warm scalar-read
  complexity.
- Sparse checkpoints require a sidecar schema change and rebuild; a bounded
  cache retains policy and memory complexity.
- Performance acceptance now requires controlled external-corpus benchmarks in
  addition to correctness CI.

### Acceptance constraints

- Existing correctness tests should be refactored to the new behavior rather
  than preserving legacy staged APIs or index-local localization.
- Any additional correctness test requires separate explicit approval and an
  explanation of why an existing test cannot be refactored.
- No new Clippy allowances may be added.
- `just ci` and `just ffi ci` must pass.
- The dedicated reconstruction performance comparison must pass the balanced
  gate before this ADR can move to Accepted.
