# ADR 3 Implementation Plan

This document defines the implementation plan for
[ADR 3](adr/003-separate-live-arrow-ipc-from-persistent-package-io.md).

It assumes the ADR is implemented as a hard break:

- no transition code
- no compatibility shims
- no dual readers or writers
- no support for `cityarrow.package.v1alpha1` once the new package format lands

## Goals

The implementation must deliver these outcomes together:

1. live process transport uses Arrow IPC stream format
2. persistent package IO uses one seekable, memory-mappable package container
3. conversion no longer depends on public `CityModelArrowParts`
4. import and export paths operate on bound columns and batch streams rather
   than eager `Vec<Row>` materialization
5. benchmark results can separate live IO, package IO, conversion only, and
   end-to-end cost

## Explicit Breaking Policy

The ADR 3 cut removes the current public transport and package surface in one
pass.

Remove:

- `CityModelArrowParts`
- `PackageTables`
- `read_package_ipc`
- `read_package_ipc_dir`
- `write_package_ipc`
- `write_package_ipc_dir`
- public `to_parts`
- public `from_parts`
- the directory-oriented Arrow IPC package layout
- the current Parquet package surface
- `cityarrow.package.v1alpha1`

Recommended repository-level cleanup:

- remove the `cityparquet` crate entirely once the new persistent package path
  is implemented
- replace the current Arrow IPC and Parquet package specs with a live-stream
  spec and a persistent package-container spec

## Target End State

The target public API is intentionally narrower than the current one.

- `ModelEncoder` encodes `OwnedCityModel` into either a live Arrow stream sink
  or a persistent package sink
- `ModelDecoder` reconstructs `OwnedCityModel` from either a live Arrow stream
  source or a persistent package source
- package reading and writing are defined in terms of a single package file,
  not a directory of per-table files
- the canonical table decomposition remains internal to the implementation and
  test suite

Recommended module layout:

```text
src/
  convert/
    export/
    import/
  transport/
    mod.rs
    schema.rs
    table_bind.rs
    stream/
      read.rs
      write.rs
    package/
      mod.rs
      manifest.rs
      index.rs
      mmap.rs
      read.rs
      write.rs
```

Modules and files to delete after the cut:

- `src/convert/mod.rs`
- `src/package/mod.rs`
- `src/package/pipeline.rs`
- `src/package/read.rs`
- `src/package/write.rs`
- `cityparquet/`

## Phase 1: Benchmarks And Cut Line

Purpose:
establish a hard baseline before deleting the current transport surface.

Work:

- add split benchmarks for:
  - live stream write
  - live stream read
  - package write
  - package read
  - encode only
  - decode only
  - end-to-end roundtrip
- pin the benchmark fixtures used by ADR 2 so the rewrite is measured against
  the same data
- add a small set of malformed-package fixtures for negative testing

Deliverables:

- `benches/transport.rs` or equivalent split benchmark targets
- fixture manifest describing base-tile and 4x-cluster inputs
- benchmark report committed or linked from `STATUS.md`

Exit criteria:

- benchmark harness runs independently for each layer named in ADR 2
- the current implementation baseline is captured before any API demolition

## Phase 2: Public API Demolition

Purpose:
remove the old public shape before the new implementation is introduced.

Work:

- stop exporting transport structs from [src/lib.rs](/home/balazs/Development/cityarrow/src/lib.rs)
- delete `CityModelArrowParts`, `PackageTables`, and the current package
  encoding enum from [src/schema.rs](/home/balazs/Development/cityarrow/src/schema.rs)
- replace `to_parts` / `from_parts` with encoder and decoder entry points
- remove public reexports that expose internal package-pipeline helpers
- remove the Parquet-first crate split if the new package format is no longer
  Parquet-based

Recommended public surface:

- `encode_to_stream`
- `decode_from_stream`
- `write_package`
- `read_package`

Exit criteria:

- the crate exposes operations, not batches
- downstream code cannot construct or depend on canonical transport structs

## Phase 3: Internal Transport Core

Purpose:
create the internal abstractions the new stream and package paths will share.

Work:

- define an internal canonical table enum and schema registry
- define a `TableBatchSource` abstraction for iterating canonical table batches
- define per-table binders that resolve column indexes once and expose typed
  array accessors
- define geometry-local span indexes for semantics, materials, and textures so
  the import path consumes contiguous row spans rather than grouped vectors
- define canonical ordering invariants and reject unsorted inputs instead of
  sorting them during import

Key implementation rule:

- `RecordBatch::column_by_name` is allowed only during binder construction, not
  inside row loops

Exit criteria:

- stream and package readers can both feed the same internal batch-source API
- import code can bind columns once per batch and walk them directly

## Phase 4: Live Arrow Stream Path

Purpose:
implement the live IPC half of ADR 3.

Work:

- add Arrow IPC stream writer support for canonical table batches
- add Arrow IPC stream reader support with incremental batch delivery
- define stream framing rules for:
  - schema emission
  - table identity
  - optional table presence
  - end-of-stream validation
- make the decoder start reconstruction as soon as the required leading tables
  arrive instead of waiting for full materialization

Recommended scope:

- target Unix pipes and generic `Read` / `Write` first
- keep networking and RPC concerns out of the initial implementation

Exit criteria:

- one process can encode a model to an Arrow stream while another reconstructs
  it incrementally
- stream roundtrip tests do not materialize a full `CityModelArrowParts` shape
  anywhere in the public or internal steady-state path

## Phase 5: Persistent Package Container

Purpose:
replace the current directory-of-files package layout with one seekable package
file.

Work:

- define `cityarrow.package.v2alpha1`
- define a single package file layout with:
  - package header
  - schema version
  - table directory or index
  - per-table batch offsets and lengths
  - payload sections aligned for efficient memory mapping
- implement package writer support for the new container
- implement `mmap`-backed package reads with lazy table and batch access
- ensure package validation can be done from the index and per-table schema
  metadata before full decode

Design rule:

- package reading must not concatenate all batches into fresh `RecordBatch`
  values before reconstruction

Exit criteria:

- persistent package read starts from one file open, not a table directory walk
- table batches are read lazily from offsets
- package-only benchmarks improve on the current fixed-cost file-count path

## Phase 6: Import Rewrite

Purpose:
replace the current eager row-based reconstruction path.

Current hotspots to remove:

- [src/convert/mod.rs:690](/home/balazs/Development/cityarrow/src/convert/mod.rs#L690)
- [src/convert/mod.rs:895](/home/balazs/Development/cityarrow/src/convert/mod.rs#L895)
- [src/convert/mod.rs:973](/home/balazs/Development/cityarrow/src/convert/mod.rs#L973)
- [src/convert/mod.rs:1030](/home/balazs/Development/cityarrow/src/convert/mod.rs#L1030)
- [src/convert/mod.rs:1137](/home/balazs/Development/cityarrow/src/convert/mod.rs#L1137)
- [src/convert/mod.rs:1172](/home/balazs/Development/cityarrow/src/convert/mod.rs#L1172)
- [src/convert/mod.rs:4678](/home/balazs/Development/cityarrow/src/convert/mod.rs#L4678)
- [src/convert/mod.rs:4766](/home/balazs/Development/cityarrow/src/convert/mod.rs#L4766)
- [src/convert/mod.rs:4853](/home/balazs/Development/cityarrow/src/convert/mod.rs#L4853)
- [src/convert/mod.rs:4939](/home/balazs/Development/cityarrow/src/convert/mod.rs#L4939)
- [src/convert/mod.rs:5030](/home/balazs/Development/cityarrow/src/convert/mod.rs#L5030)
- [src/convert/mod.rs:5117](/home/balazs/Development/cityarrow/src/convert/mod.rs#L5117)

Work:

- initialize the model from bound metadata, transform, extension, vertex, and
  shared-resource tables
- build per-geometry and per-template span indexes once
- reconstruct geometries and template geometries directly from bound arrays
- attach cityobject geometries during the main cityobject pass instead of
  rereading geometry tables later
- reconstruct semantic, material, and texture assignments from row spans rather
  than cloning and resorting grouped vectors
- reject invariant violations as soon as they are observed

Exit criteria:

- the import path has no eager `Vec<Row>` materialization in hot geometry and
  appearance reconstruction
- the import path does not reread already-decoded tables
- no clone-and-sort rebuild remains on the hot path

## Phase 7: Export Rewrite

Purpose:
replace row staging with direct batch construction and fused traversal.

Current hotspot:

- [src/convert/mod.rs:436](/home/balazs/Development/cityarrow/src/convert/mod.rs#L436)

Work:

- split export into core, geometry, semantic, and appearance builders backed by
  Arrow array builders rather than transport row structs
- fuse geometry export so boundaries, semantics, materials, and textures are
  derived in one coordinated traversal
- preserve canonical row ordering at write time so import never needs recovery
  sorts
- write batches directly to stream or package sinks instead of first building a
  public transport aggregate

Exit criteria:

- export derives each geometry-related sidecar once
- export emits canonical ordering by construction
- no public or long-lived internal `parts` aggregate is required

## Phase 8: Crate And Documentation Cleanup

Purpose:
delete the old architecture completely and make the new one the only documented
surface.

Work:

- remove `cityparquet/` if it no longer owns any live package surface
- replace [docs/cityjson-arrow-ipc-spec.md](/home/balazs/Development/cityarrow/docs/cityjson-arrow-ipc-spec.md)
  with a live Arrow stream transport spec
- replace [docs/cityjson-parquet-spec.md](/home/balazs/Development/cityarrow/docs/cityjson-parquet-spec.md)
  with the persistent package container spec or delete it if obsolete
- update [docs/design.md](/home/balazs/Development/cityarrow/docs/design.md) to describe the
  new public API and internal transport role
- update examples, tests, and `README.md`
- remove docs that describe the deleted package directory layout

Exit criteria:

- no user-facing documentation describes the deleted transport surface
- the workspace and docs reflect one architecture rather than current plus next

## Verification Gates

Correctness gates:

- exact semantic roundtrip for the current conversion fixture corpus
- negative tests for malformed stream frames, malformed package indexes,
  missing required tables, duplicate ids, broken ordinals, and unsupported
  combinations
- package validation tests that fail before full decode when the package index
  is inconsistent

Performance gates:

- split benchmarks from ADR 2 must continue to run in CI or a documented local
  benchmark workflow
- package read avoids eager concatenation
- live stream decode can start before full payload completion

Architecture gates:

- no public transport struct equivalent to `CityModelArrowParts`
- no public API that exposes canonical table batches directly
- no old package reader or writer remains in the tree

## Sequencing Recommendation

Implement in this order:

1. Phase 1 benchmarks
2. Phase 2 public API demolition
3. Phase 3 internal transport core
4. Phase 4 live Arrow stream path
5. Phase 5 persistent package container
6. Phase 6 import rewrite
7. Phase 7 export rewrite
8. Phase 8 cleanup and documentation

The critical rule is to avoid hybrid architecture.

Do not:

- keep old and new readers side by side
- keep a public transport struct while also introducing stream-first APIs
- preserve `v1alpha1` package compatibility
- preserve `cityparquet` for legacy naming reasons if it no longer matches the
  actual architecture
