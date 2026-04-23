# Adopt a Canonical Transport Boundary Around `CityModelArrowParts`

## Status

Superseded by [ADR 006](006-cut-public-surface-to-thin-batch-and-stream-codec.md)

## Related Commits

- `77c8bfb` Implement cityjson-arrow package schema
- `f35a37b` Implement core cityjson-arrow conversion
- `6ba321d` Implement package I/O layer
- `e8057d6` Integrate package roundtrip and remove prototype tree
- `09e9b0a` Implement Arrow IPC package IO
- `55452b4` Refresh docs and lock canonical schema

## Context

`cityjson-arrow` exists to move full-fidelity `cityjson-rs` models across Arrow
storage boundaries.

The crate needed a transport architecture that was explicit enough to support
schema-locked package I/O, but narrow enough to avoid becoming a second domain
model beside `cityjson::v2_0::OwnedCityModel`.

The main pressure points were:

- CityJSON is structurally nested, while Parquet and Arrow IPC package I/O work
  better with explicit tables and join keys
- reconstruction must not depend on fragile row-order assumptions
- Parquet and Arrow IPC support should not diverge into separate logical
  schemas
- the public API should stay concrete and navigable instead of adding a large
  abstraction layer around transport concerns

## Decision

At the time of this decision, `cityjson-arrow` used a single canonical
transport decomposition: `CityModelArrowParts`.

The architecture is:

1. `cityjson::v2_0::OwnedCityModel` remains the semantic source and sink
2. `convert::to_parts` projects that model into canonical Arrow tables grouped
   as `CityModelArrowParts`
3. package readers and writers persist those canonical tables through either
   Parquet or Arrow IPC file encoding
4. `convert::from_parts` reconstructs the semantic `OwnedCityModel` from the
   same canonical parts shape

The package contract is shared across both supported encodings. Format choice
changes the file encoding on disk, not the logical schema.

The canonical transport rules are:

- semantic external ids remain explicit strings
- dense transport ids and ordinals drive joins and reconstruction
- shared resources stay pooled instead of being duplicated into nested payloads
- geometry topology is normalized into boundary sidecars
- projected attributes stay on the owning table
- unsupported nested or mixed attribute shapes fall back to lossless text
  encodings instead of Arrow union-based schemas

`cityjson-arrow` therefore remains a transport layer, not a semantic fork of
`cityjson-rs`.

## Consequences

Historical consequences:

- one logical package schema works for both Parquet and Arrow IPC
- the public API stays concrete around `OwnedCityModel`, package helpers, and
  `CityModelArrowParts`
- schema evolution is easier to reason about because there is one canonical
  decomposition to test and document
- reconstruction uses explicit ids and ordinals instead of implicit row order
- package roundtrip behavior can be locked down with exact table equality tests

Trade-offs:

- the current conversion and package read paths are eager and fully in-memory,
  which makes large-model roundtrips memory intensive
- the canonical schema is more verbose than the nested CityJSON source shape
  because topology and assignments are written through explicit sidecar tables
- some attribute shapes must fall back to text encodings to remain compatible
  with the shared Parquet and Arrow IPC contract
- the crate intentionally does not hide transport format choice behind a plugin
  or registry abstraction
