# cityjson-arrow documentation

`cityjson-arrow` and `cityjson-parquet` define the Arrow batch, stream, and
package transport boundary for `cityjson-rs`.

This site documents the live stream surface, the persistent package surface,
the shared package schema, and the language-agnostic layouts.

## Start Here

- [cityjson-arrow](cityjson-arrow.md): live Arrow IPC stream transport and model conversion
- [cityjson-parquet](cityjson-parquet.md): persistent package I/O
- [Package schema](package-schema.md): shared canonical table contract
- [Arrow IPC spec](cityjson-arrow-ipc-spec.md): Arrow IPC layout specification
- [Parquet spec](cityjson-parquet-spec.md): Parquet layout specification
- [ADR 2 and ADR 3 benchmark follow-up](adr-002-003-benchmark-follow-up.md):
  first post-refactor benchmark reading and the exact split benchmark matrix
- [ADR 2 and ADR 3 borrowed strings decision](adr-002-003-borrowed-strings-decision.md):
  why the next optimization slice stays on the owned semantic boundary
- [ADR 2 and ADR 3 optimization plan](adr-002-003-optimization-plan.md):
  the focused execution plan for stream, package, and conversion optimization
- [ADR 4: reduce conversion cost with ordinal canonical relations](adr/004-reduce-conversion-cost-with-ordinal-canonical-relations.md):
  the `v2alpha2` schema cleanup, conversion rationale, and measured result
- [ADR 005: cut `v3alpha1` schema for Arrow-native projection and batch-native conversion](adr/005-cut-v3-schema-for-arrow-native-projection-and-batch-native-conversion.md):
  the hard schema break for recursive typed projections and batch-native conversion
- [ADR 006: cut the public surface to a thin batch and stream codec](adr/006-cut-public-surface-to-thin-batch-and-stream-codec.md):
  why the crate now exposes function-based stream APIs and ordered batch codecs
- [ADR 005 v3 implementation plan](adr-005-v3-implementation-plan.md):
  the execution sequence for the `v3alpha1` schema cut and encode/decode refactor

## Scope

The documentation focuses on the package boundary, not the CityJSON semantic
model itself.

- `cityjson-arrow` owns the live Arrow stream and ordered-batch codec surface.
- `cityjson-parquet` owns the persistent package I/O surface.
- Both crates share the same canonical schema and reconstruction rules.

## Implementation Notes

The package format is currently `cityjson-arrow.package.v3alpha2`.
It remains schema-locked and reconstructible from ids, ordinals, and typed
recursive projection layouts.
