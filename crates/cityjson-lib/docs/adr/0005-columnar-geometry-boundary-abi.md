# ADR 0005: Columnar Geometry Boundary ABI

## Status

Accepted.

## Context

The shared FFI expansion plan includes a bulk geometry and boundary-access
slice. The main open design question for that slice was how boundary data
should cross the C ABI:

- rebuild nested JSON-like arrays
- expose raw internal references
- or copy an explicit flat payload

The upstream `cityjson-rs` storage already keeps geometry boundaries in flat
offset-encoded form. The FFI layer also needs a wasm-facing extraction story,
but `cityjson-rs` does not currently provide a general triangulation or mesh
derivation API that should be treated as stable shared ABI.

## Decision

Geometry boundary extraction in the shared ABI uses one owned, copied,
columnar payload:

- `vertex_indices`
- `ring_offsets`
- `surface_offsets`
- `shell_offsets`
- `solid_offsets`

The payload also carries:

- `geometry_type`
- `has_boundaries`

Boundary-ordered coordinates are exposed as a separate copied vertex buffer
instead of being folded into the topology payload.

The shared ABI does not add triangulation in this slice. The wasm adapter uses
boundary topology plus boundary-ordered coordinates as its flat extraction
primitive.

## Consequences

Positive:

- the ABI mirrors the upstream stored geometry layout instead of inventing a
  new transport shape
- C++, Python, and wasm can all consume one bulk boundary format with explicit
  ownership
- the slice stays additive and low-risk because it avoids derived mesh
  semantics that the upstream library does not yet freeze

Tradeoffs:

- callers that want nested JSON-style boundaries must reconstruct them
  themselves
- `GeometryInstance` stays limited to the stored geometry surface in this
  slice
- true triangle or mesh extraction remains a future derived-operation design,
  not part of the current ABI
