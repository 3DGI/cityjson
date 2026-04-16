# Cut Public Surface To Thin Batch And Stream Codec

## Status

Accepted

## Context

The rewrite plan for the `cityjson-rs` / `cityjson-arrow` split is explicit
about the intended role of this crate:

- `cityjson-rs` is the native model
- `cityjson-arrow` is a thin Arrow codec over low-level relational views
- public APIs should expose batches and stream transport, not a parts-centric
  intermediate model

The current `cityjson-arrow` implementation had already removed full
`CityModelArrowParts` staging from the main stream hot path, but the repo still
described and exercised the crate through `ModelEncoder` / `ModelDecoder`.

That was the wrong abstraction boundary for the crate as it exists today:

- it hides the ordered-batch codec shape that sibling crates actually depend on
- it makes the stream API look object-centric instead of transport-centric
- it leaves the documentation anchored on historical wrapper types even though
  the implementation is already batch-native internally

There is also an upstream constraint:

- `cityjson-rs` currently exposes raw pool views, raw boundary views, and dense
  export remaps
- it does not yet expose the full proposed relational snapshot/import API

So `cityjson-arrow` can become thinner today, but it cannot yet delegate every
projection and import concern into `cityjson-rs`.

## Decision

`cityjson-arrow` will expose a thin public codec surface built around ordered
batches and live streams.

The public surface is now:

1. `write_stream(writer, model, &ExportOptions)`
2. `read_stream(reader, &ImportOptions)`
3. `export_reader(model, &ExportOptions)`
4. `ModelBatchDecoder`
5. `import_batches(header, projection, batches, &ImportOptions)`

The implementation rules are:

- the public API does not center `ModelEncoder` / `ModelDecoder`
- canonical tables remain an internal transport contract
- doc-hidden parts bridges remain only where currently unavoidable for the
  sibling `cityjson-parquet` crate and local diagnostics
- the stream write path remains direct-to-sink rather than routing through a
  public parts aggregate
- documentation must describe the upstream gap explicitly instead of pretending
  that the full relational-core rewrite has already landed

## Consequences

Good:

- the crate surface matches its real responsibility: Arrow batches and Arrow
  streams
- repo-local tests and benches exercise the thin codec API directly
- the docs stop presenting the crate as if `CityModelArrowParts` or wrapper
  encoder/decoder types were the intended user boundary
- the remaining upstream dependency on `cityjson-rs` is explicit and reviewable

Trade-offs:

- the current batch reader still materializes canonical Arrow batches in this
  crate rather than walking a future `cityjson-rs::relational` snapshot
- `ProjectionLayout` is still discovered and carried here because the upstream
  relational string/attribute contract does not exist yet
- doc-hidden compatibility hooks remain until `cityjson-parquet` can move off
  them
