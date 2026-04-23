# ADR 2 And ADR 3 Optimization Plan

This document defines the next implementation slice after the first ADR 3
public-surface refactor benchmark run.

It complements:

- [ADR 2 and ADR 3 benchmark follow-up](adr-002-003-benchmark-follow-up.md)
- [ADR 2 and ADR 3 borrowed strings decision](adr-002-003-borrowed-strings-decision.md)
- [ADR 4: reduce conversion cost with ordinal canonical relations](adr/004-reduce-conversion-cost-with-ordinal-canonical-relations.md)

The purpose of this plan is narrower than the original ADR 3 implementation
plan:

- isolate `encode_parts` and `decode_parts` so they can be benchmarked and
  eventually bypassed in the steady-state public path
- remove eager `read_to_end` from the live stream path
- remove per-table payload buffering from live stream and package writes
- push package and stream decode toward incremental, lazy reconstruction

## Scope

This plan is about execution-model optimization, not architectural reversal.

Keep:

- `OwnedCityModel` as the semantic source and sink
- one live stream surface in `cityjson-arrow`
- one persistent package surface in `cityjson-parquet`
- the canonical table decomposition as an internal contract

Do not add in this slice:

- a borrowed semantic-model public API
- a restored public `CityModelArrowParts` surface
- dual old and new stream or package formats
- compatibility shims for deleted APIs

## Current Hotspots To Remove

The follow-up note identified these current bottlenecks:

- `ModelEncoder::encode` still does `OwnedCityModel -> encode_parts ->
  write_model_stream`
- `ModelDecoder::decode` still does `read_model_stream -> decode_parts ->
  OwnedCityModel`
- `write_model_stream` buffers all table payloads before writing the final
  stream
- `read_model_stream` reads the full source into memory before table decode
- `PackageWriter::write_file` serializes each table into a temporary `Vec<u8>`
  before writing the package file
- `PackageReader::read_file` still goes through full table decode,
  concatenation, canonical-parts reconstruction, and only then semantic import

Those are the first costs to attack because they block the intended ADR 3
stream-first and lazy-package behavior.

## Workstream 1: Isolate Conversion From Transport

Purpose:
separate conversion-only work from transport-only work and stop forcing the
steady-state public path through the same internal aggregate.

Work:

- keep `internal::encode_parts` and `internal::decode_parts` as doc-hidden
  benchmark and sibling-crate helpers
- move their implementation behind explicit internal conversion modules so they
  are no longer the mandatory path taken by `ModelEncoder`, `ModelDecoder`,
  `PackageWriter`, and `PackageReader`
- define an internal canonical-table producer interface that can emit canonical
  batches to a sink without first assembling a full `CityModelArrowParts`
- define a matching canonical-table consumer interface that can accept batches
  from a source without first rebuilding a full `CityModelArrowParts`
- add the split benchmarks from the follow-up note in the `cityjson-arrow`
  workspace itself:
  - `convert_encode_parts`
  - `convert_decode_parts`
  - `stream_write_parts`
  - `stream_read_parts`
  - `package_write_parts`
  - `package_read_parts`

Deliverables:

- one conversion-only benchmark target
- one transport-only benchmark target
- doc-hidden internal source and sink abstractions used by both stream and
  package code

Exit criteria:

- conversion-only and transport-only numbers are measurable without going
  through the public end-to-end API
- the public stream and package path no longer structurally requires a full
  `CityModelArrowParts` aggregate

## Workstream 2: Remove Eager Stream Buffering

Purpose:
make the live stream path behave like a real streaming boundary rather than a
manifest-prefixed buffered file.

### Stream Write

Current blocker:
the current live stream format writes a manifest before payloads, so the writer
must know every payload length up front and therefore buffers each table in
memory.

Work:

- replace the current manifest-first live framing with a sequential framed
  stream format
- define a stream prelude that carries only the model header and projection
  layout needed to validate later tables
- emit canonical tables in strict canonical order as independent frames
- let each frame carry:
  - table id
  - row count
  - payload length
  - one Arrow IPC payload
- write each table frame directly to the destination `Write`
- keep only frame metadata in memory, not full serialized payloads

### Stream Read

Current blocker:
the current reader calls `read_to_end` before it can inspect the leading
manifest and locate payloads.

Work:

- implement a framed stream reader that consumes one table frame at a time from
  `Read`
- validate the stream prelude once, then validate each table frame as it
  arrives
- decode the current Arrow IPC payload directly from the underlying reader or a
  bounded frame reader
- delete `read_to_end` from the live stream read path
- make the decoder hand each decoded batch to the importer as soon as the frame
  is available

Deliverables:

- a new live stream framing document
- stream writer and reader implementations that do not buffer the whole stream

Exit criteria:

- no `Vec<Vec<u8>>` payload staging remains in the live stream writer
- no eager `read_to_end` remains in the live stream reader
- live decode can begin before end-of-stream

## Workstream 3: Remove Package Payload Buffering And Eager Manifest Reads

Purpose:
make the persistent package path pay only for the table being written or read.

### Package Write

Current blocker:
the package writer serializes each table into a temporary `Vec<u8>` before
writing the package file.

Work:

- write each canonical table payload directly to the destination file
- collect only the manifest entries in memory:
  - table id
  - file offset
  - payload length
  - row count
- obtain payload lengths from file position deltas or a counting writer instead
  of retaining the serialized payload bytes
- keep the manifest-at-end package structure if it continues to serve lazy
  reads cleanly

### Package Read

Current blockers:
`read_package_manifest` reads the whole file to inspect the footer, and
`read_package_file` still decodes every table into a full canonical aggregate
before semantic import starts.

Work:

- change manifest reads to footer-first seek logic
- map the package once, then expose table slices and batch iterators lazily from
  the manifest index
- stop concatenating all batches into fresh `RecordBatch` values in the
  steady-state package read path
- make package read feed the same internal canonical-table source interface used
  by the live stream reader

Deliverables:

- footer-first manifest reader
- lazy table-slice package reader
- package-only benchmarks for:
  - `package_read_parts`
  - `package_read_manifest`

Exit criteria:

- package writing has no per-table payload byte buffering
- `read_manifest` does not load the whole package file
- package read no longer requires full-batch concatenation before import can
  begin

## Workstream 4: Push Reconstruction Toward Incremental And Lazy Decode

Purpose:
replace whole-parts reconstruction with ordered, binder-driven import from
canonical batches as they arrive.

Work:

- split import into explicit ordered stages:
  1. header and projection validation
  2. metadata, transform, and extensions
  3. shared pools such as vertices, template vertices, semantics, materials,
     and textures
  4. cityobject skeletons
  5. geometry and template-geometry reconstruction
  6. sidecar attachment for children, semantics, materials, and textures
- bind columns once per batch and keep typed binder structs on the hot path
- replace grouped row vectors and clone-heavy staging with span indexes keyed by
  canonical ids and ordinals
- move cityobject-to-geometry attachment away from
  `HashMap<String, Vec<(u32, u64)>>` staging toward id and ordinal-driven
  ordered attachment
- let the importer consume batches directly from the live stream or package
  table source instead of waiting for a fully materialized canonical aggregate
- treat missing required tables, duplicate ids, broken ordinals, or out-of-order
  frames as immediate decode failures

Recommended invariant:

- stream and package readers must emit tables in canonical order so the importer
  can stay single-pass

Deliverables:

- an importer state machine that accepts canonical batches incrementally
- per-table binder modules
- span-index helpers for geometry and appearance sidecars

Exit criteria:

- the steady-state read path does not rebuild a full `CityModelArrowParts`
- reconstruction starts before all tables are decoded
- geometry and appearance import do not depend on clone-and-sort grouped row
  rebuilds

## Workstream 5: Verification And Rollout

Purpose:
make sure the optimization slice produces measurable and defensible movement.

Work:

- keep the split benchmarks pinned to:
  - `io_3dbag_cityjson`
  - `io_3dbag_cityjson_cluster_4x`
- keep downstream `cjlib` headline and diagnostic suites separate so
  end-to-end and split numbers are not collapsed into one benchmark target
- require benchmark preparation code to validate native artifacts with the
  current decoders before reuse; file existence alone is not a valid
  compatibility check across stream/package revisions
- require native write benchmarks to pre-create their temp directory once and
  overwrite a fixed output path inside the timed loop
- compare each step against the current refactor campaign, not only against the
  JSON-normalized plot summary
- record allocation counters for:
  - `convert_decode_parts`
  - `stream_read_parts`
  - `package_read_parts`
- add negative tests for:
  - malformed stream prelude
  - duplicate or missing stream frames
  - malformed package footer or manifest
  - missing required canonical tables
  - broken canonical id or ordinal ordering

Exit criteria:

- split benchmarks can localize improvement to conversion, stream transport, or
  package transport
- stale native benchmark artifacts cannot silently survive a format change in
  the downstream harness
- write-path benchmark timings do not include `tempdir()` churn
- read-side allocation totals fall when the eager whole-parts path is removed
- public end-to-end read performance improves without reopening the semantic
  boundary decision

## Recommended Sequencing

Implement in this order:

1. Workstream 1: isolate conversion from transport and land split benchmarks
2. Workstream 2: replace live stream framing and delete `read_to_end`
3. Workstream 3: remove package payload buffering and eager manifest reads
4. Workstream 4: move import to incremental and lazy reconstruction
5. Workstream 5: lock in verification and measure the new baseline

The main rule is to avoid another half-cut architecture.

Do not:

- keep a manifest-first buffered live stream while claiming incremental decode
- keep package read based on eager batch concatenation while claiming lazy
  package access
- route the steady-state public API through `encode_parts` and `decode_parts`
  once the new source and sink interfaces exist
