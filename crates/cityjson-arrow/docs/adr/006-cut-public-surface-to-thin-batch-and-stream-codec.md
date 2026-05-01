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

There is still an upstream split to respect:

- `cityjson-rs` now exposes a borrowed relational export view and an owned
  relational snapshot/import builder
- `cityjson-arrow` still owns projection discovery, Arrow schema assembly, and
  Arrow-specific reconstruction glue

So the crate can now root export in the relational surface, but it is still the
transport layer rather than a thin forwarding shim over all import/export work.

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
- export and stream writing start from `cityjson_types::relational::ModelRelationalView`
  rather than walking the owned semantic model directly
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
- the export hot path now consumes the same borrowed relational contract that
  downstream sibling crates are expected to target

Trade-offs:

- `ProjectionLayout` is still discovered and carried here because the upstream
  relational string/attribute contract does not exist yet
- doc-hidden compatibility hooks remain until `cityjson-parquet` can move off
  them
