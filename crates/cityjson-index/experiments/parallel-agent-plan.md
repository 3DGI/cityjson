# Parallel Vertex-Store Bake-Off

## Summary

Evaluate the three persistent sparse vertex-storage options from
[ADR 012](../docs/adr/012-evaluate-persistent-sparse-vertex-storage.md) using
one shared experimental harness and three parallel implementation branches.
Each candidate agent works in an isolated Git worktree created from the same
harness commit. Candidate implementations remain separate while they are
measured; only the selected strategy is later ported into production.

All experiment-only implementation, test, and orchestration code belongs under
this directory. The shared harness may expose narrowly scoped, feature-gated
hooks from the main crate where access to indexing or reconstruction internals
is unavoidable, but it must not change the default public API.

## Phase 1: Shared Harness

The coordinator creates `experiment/vertex-store-harness` from the agreed base
commit and completes this phase before starting the candidate agents.

- Add a feature-gated `vertex-store-bakeoff` binary with its implementation
  rooted under `experiments/`.
- Define the internal `VertexStore` contract for building persistent state and
  loading exact `(source_id, vertex_index)` coordinate requirements.
- Implement one shared package path that loads and validates indexed
  CityObjects, collects and deduplicates vertex requirements for the entire
  batch, invokes the candidate store, remaps geometry, and preserves package
  request order and duplicates.
- Make `read_package` use the same experimental path as a one-item
  `read_packages` call. Drop all decoded coordinates and encoded buffers at the
  end of each call.
- Define fixed strategy identifiers, distinct sidecar paths, a versioned JSON
  result schema, profiling provenance, and deterministic Groningen-182 sample
  generation.
- Make measured read processes require a prebuilt, matching sidecar. Missing,
  stale, schema-v2, or strategy-mismatched sidecars must fail without creating,
  deleting, migrating, or rebuilding persistent data.
- Provide common conformance tests and benchmark entry points for all four
  experiments in ADR 012. Candidate modules supply only storage-specific
  construction, validation, lookup, and telemetry.

Merge and validate this harness commit before creating the candidate
worktrees. After that point, candidate agents must not independently change the
shared contract or result schema. Necessary harness changes are reported to the
coordinator and applied once to the harness branch, after which all candidate
branches are rebased onto the new harness commit.

## Phase 2: Parallel Worktrees

Create the following branches and worktrees from the exact shared-harness
commit:

| Agent | Branch | Strategy ownership |
|---|---|---|
| Packed chunks | `experiment/vertex-store-packed` | Option A: packed coordinate chunks |
| JSON offsets | `experiment/vertex-store-offsets` | Option B: compact JSON offsets |
| Frame of reference | `experiment/vertex-store-for` | Option C: frame-of-reference bit packing |

Give every worktree its own Cargo target directory, experiment work root,
candidate sidecar path, and result directory. The agents may share the
Groningen-182 corpus read-only, but must not share writable sidecars, temporary
datasets, target directories, or raw result files.

Each agent owns one strategy subdirectory and its strategy-specific tests.
Shared harness files are coordinator-owned. Every agent must leave a focused
commit series, document reproducible build and run commands, and report its
commit SHA with every measurement artifact.

### Packed Coordinate Chunks Agent

- Implement the `source_vertex_chunks` schema exactly as specified by ADR 012.
- Stream vertices into chunks of at most 16,384 during reindexing, encoding
  each coordinate as three little-endian `i64` values.
- Resolve requested vertices using chunk arithmetic and SQLite incremental BLOB
  reads. Coalesce adjacent requested indices without reading unrelated chunks.
- Validate chunk bounds, row invariants, vertex counts, and payload lengths
  before returning any coordinates.

### Compact JSON Offsets Agent

- Implement the `source_vertex_offset_chunks` schema exactly as specified by
  ADR 012.
- Record chunk-relative little-endian `u32` offsets plus the final sentinel
  while scanning the authoritative source JSON. Reject spans exceeding
  `u32::MAX`.
- Coalesce source-file reads for consecutive requested indices, find each JSON
  value within its bounded slice, and parse exactly one `[i64; 3]` per requested
  vertex.
- Validate monotonic offsets, the zero origin, sentinel bounds, source
  freshness, and complete coordinate parsing before returning results.

### Frame-of-Reference Agent

- Implement the `source_vertex_superchunks` schema, 16,384-vertex
  superchunks, and independently encoded subblocks of at most 128 vertices.
- Encode each 27-byte descriptor and its X/Y/Z difference streams using the
  little-endian and least-significant-bit-first rules in ADR 012.
- Use checked `i128` subtraction and addition so the complete signed `i64`
  coordinate domain round-trips exactly.
- Read each touched subblock once per batch and decode only requested indices.
  Validate widths, computed payload length, padding, chunk bounds, and decoded
  conversions before returning results.

## Phase 3: Agent Validation

Each candidate agent runs the same validation sequence before handing results
back to the coordinator:

1. Run the shared unit and conformance suite, including empty sources,
   duplicate and out-of-order package requests, malformed and truncated
   persistent data, freshness failures, and schema-v2 reindex requirements.
2. Run regular-CityJSON reconstruction and normalized-schema tests against the
   candidate. Compare package counts, CityObject counts, relationships,
   memberships, deterministic package digests, and boundary/extrema vertices
   with the source data.
3. Run `just ci` from `crates/cityjson-index`. Do not claim the branch ready
   until it passes.
4. Confirm that singleton and batch reads retain no decoded source-sized vertex
   arrays after returning.
5. Submit the branch name, final commit SHA, commands used, validation summary,
   and paths to raw artifacts. Do not commit sidecars or large profiler output.

## Phase 4: Controlled Measurements

The coordinator runs the accepted candidate branches against the same immutable
Groningen-182 corpus and follows ADR 012 without strategy-specific deviations.

- Correctness and storage: build one fresh sidecar, run all conformance checks,
  and record complete and per-table sizes, vertex and payload counts, row
  counts, and observed bytes per vertex.
- Reindex cost: run three fresh-process, four-worker reindexes per candidate
  under the same 28 GiB cgroup with swap disabled; report median and range.
- Read latency and batching: use the same deterministic, source-stratified
  10,000-package sample; measure singleton reads and 2,048-package batches in
  first and immediately repeated passes.
- Tyler materialization: run the full 707,239-package workload at 1, 4, and 24
  workers, with three fresh-process native repetitions per cell and existing
  100 ms memory sampling.
- Rotate candidate execution order between repetitions. Do not describe either
  pass as a controlled cold or warm cache state, and do not drop the host page
  cache.

Every result must identify the strategy, candidate commit, harness commit,
corpus identity, sidecar path, worker count, repetition, and relevant runtime
configuration. Raw sidecars and profiling artifacts stay under isolated
`target/` paths. Commit only orchestration code, machine-readable summaries,
and concise result tables under `experiments/`.

## Phase 5: Review and Selection

The coordinator compares artifacts without merging the three candidate
implementations. Reject a strategy if it changes reconstructed output, loses
coordinate precision, returns partial models for malformed storage, reads a
complete source vertex array for a package, or retains an unbounded decoded
vertex cache.

Add the comparable result tables and the maintainers' selection rationale to
ADR 012. The selected representation then receives a separate production and
compatibility plan. Port that implementation deliberately from its experiment
branch; do not merge the entire experimental branch. Remove unselected
implementations after their relevant summaries and rationale have been
retained.

## Assumptions

- The shared harness is integrated before the candidate agents start.
- All three worktrees begin at the same harness commit and use the same Rust
  toolchain and corpus revision.
- The Groningen-182 corpus is immutable and available read-only to every
  worktree.
- Candidate selectors, schemas, and telemetry are experiment-only and do not
  become default or stable public API.
- Connection pooling and the concurrent `CityIndex` API remain outside this
  experiment, as required by ADR 012.
