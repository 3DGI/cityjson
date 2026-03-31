# cityarrow Design

This document rewrites the design for `cityarrow` around the current
`cityjson-rs` model and the actual state of this repository as of 2026-03-31.

It replaces the older implicit assumptions in the codebase with an explicit
transport design.

The concrete package schema draft that follows from this design is documented in
[docs/package-schema.md](package-schema.md).

## Purpose

`cityarrow` exists to move `cityjson-rs` models across Arrow and Parquet
boundaries.

It does not exist to define a second semantic model family.

The design target is therefore:

- semantic core: `cityjson::v2_0::CityModel`
- transport boundary: Arrow IPC, Parquet, and framed streams
- advanced internal decomposition: `CityModelArrowParts`

## Current Reality

The canonical rewrite is now implemented on the Parquet package path.

### Confirmed state from the code review

- `CityModelArrowParts` already expresses the right component split.
- canonical Parquet package write exists.
- canonical Parquet package read exists.
- the crate contains component schemas for metadata, transform, vertices,
  cityobjects, geometries, semantics, materials, textures, and UV coordinates.
- `cargo test` passes.
- `cargo test -- --ignored` passes, including the real-data
  `serde_cityjson -> cityarrow -> cjval` gate.

### Current unsupported scope

The rewrite is not universal over every possible `cityjson-rs` shape.
The current conversion code still rejects:

- point and linestring semantic mappings
- point and linestring material mappings
- template geometry semantics
- template geometry materials
- template geometry textures

## Source Model In `cityjson-rs` Today

The current source model should drive the transport design.

### Semantic unit

The semantic unit is:

```rust
cityjson::v2_0::CityModel<VR, SS>
```

At the transport boundary, `cityarrow` should accept and return `CityModel`,
not a transport-native semantic replacement.

### Attributes are inline again

Current `cityjson-rs` stores attributes inline as:

- `AttributeValue::Null`
- `AttributeValue::Bool`
- `AttributeValue::Unsigned`
- `AttributeValue::Integer`
- `AttributeValue::Float`
- `AttributeValue::String`
- `AttributeValue::Vec`
- `AttributeValue::Map`
- `AttributeValue::Geometry`

These attributes appear directly on semantic owners such as:

- `CityModel::extra`
- `Metadata::extra`
- `CityObject::attributes`
- `CityObject::extra`
- `Semantic::attributes`

This is important: the old `cityarrow` assumption that attribute export is
blocked on a global `AttributePool` is no longer true.

### Shared resources remain pooled

Current `cityjson-rs` still uses shared resource collections for:

- geometries
- semantics
- materials
- textures
- template geometries

That means `cityarrow` still benefits from component batches that mirror these
pools.

### Boundaries are still offset-based in-memory

Current geometry boundaries remain flattened and offset-based:

- `vertices`
- `rings`
- `surfaces`
- `shells`
- `solids`

This is a good in-memory shape for `cityjson-rs`. It should not force the wire
format to use the same exact layout if a normalized Arrow/Parquet layout is
better.

### Raw access exists

Current `cityjson-rs` also exposes a raw accessor for zero-copy reads of the
internal pools.

That gives `cityarrow` two viable implementation paths:

- stable high-level iterators for normal conversion code
- raw accessors for high-throughput batch builders

## Design Rules

The redesign should follow these rules.

1. `cityarrow` is a format boundary, not a semantic fork.
2. The public API should trade in `CityModel` or streams of `CityModel`.
3. `CityModelArrowParts` is an advanced transport detail.
4. Arrow-only features must not become the canonical Parquet design if Parquet
   cannot encode them.
5. Normalization from the semantic model into a transport-friendly layout is
   acceptable and expected at serialization time.
6. The rewrite may break existing `cityarrow` APIs and internal schemas freely
   if that yields a cleaner architecture. Transitional compatibility is not a
   design goal.

## Transport Model

`CityModelArrowParts` remains a good internal decomposition, but it should be
treated as a transport package.

### Suggested component split

| Component | Rows | Role |
| --- | --- | --- |
| `metadata` | 0 or 1 | model metadata |
| `transform` | 0 or 1 | transform metadata |
| `extensions` | 0 or 1 | CityJSON extensions metadata |
| `extra` | 0 or 1 | model-level extra attributes |
| `vertices` | N | world coordinates |
| `cityobjects` | N | cityobject graph plus attribute projections |
| `geometries` | N | geometry topology and references |
| `semantics` | N | semantic resource pool |
| `materials` | N | material resource pool |
| `textures` | N | texture resource pool |
| `vertices_texture` | N | UV coordinate buffer |
| `template_vertices` | N | geometry template vertices |
| `template_geometries` | N | geometry template pool |

This split keeps the transport aligned with the current `cityjson-rs` ownership
model and with the intended explicit-module architecture in `cjlib`.

## Attribute Encoding

Attribute encoding is the main design gap.

### What should not be the canonical solution

The old code path models attributes as:

- Arrow `Map<Utf8, DenseUnion>`

That is not a viable canonical Parquet representation because Parquet still does
not support Arrow union types natively.

Even if Arrow IPC keeps a richer in-memory form for debugging or experimentation,
the crate design should not center on unions.

### Canonical Parquet-safe approach

Project attributes into typed sibling columns per owner scope.

For each owner scope:

- discover the union of attribute keys present in the batch
- resolve a physical Arrow type per key
- emit one nullable column per discovered key
- attach field metadata that records the original CityJSON key and encoding

Examples of owner scopes:

- model extra
- metadata extra
- cityobject attributes
- cityobject extra
- semantic attributes

### Type resolution rules

Primitive values map directly:

- `Null` -> null
- `Bool` -> `Boolean`
- `Unsigned` -> `UInt64`
- `Integer` -> `Int64`
- `Float` -> `Float64`
- `String` -> `LargeUtf8`

Nested values map to JSON strings:

- `Vec` -> `LargeUtf8` containing JSON
- `Map` -> `LargeUtf8` containing JSON

Geometry-valued attributes need an explicit policy:

- default lossless mode: `UInt32` geometry handle column with field metadata
- optional interoperability mode: additional `Binary` WKB shadow column

If a key appears with mixed primitive types across rows, coerce it to
`LargeUtf8` rather than trying to preserve a union in Parquet.

That preserves data, keeps the schema Parquet-safe, and matches the direction
used by GeoParquet-style table layouts.

### Column naming

Use namespaced columns inside the owning component.

Examples:

- `attributes.height`
- `attributes.name`
- `attributes.address_json`
- `extra.source`
- `extra.status`

The exact field naming can be flattened or nested, but the contract should make
the owner scope obvious and avoid collisions between `attributes` and `extra`.

## Geometry Encoding

Geometry is the second major design area.

### Source model

Current `cityjson-rs` exposes offset-based flattened boundaries. That remains
the correct in-memory source representation.

### Transport normalization

At the Arrow/Parquet boundary, normalize geometry topology into a more
serialization-friendly layout.

Recommended exported fields:

- `vertex_indices`
- `ring_lengths`
- `surface_lengths`
- `shell_lengths`
- `solid_lengths`

This normalization is a simple linear conversion from the current offset-based
boundary:

- offsets in `cityjson-rs`
- counts in Arrow and Parquet

Why this normalization is useful:

- it aligns better with WKB polygon traversal order
- it is easier to map into GeoArrow-style polygon hierarchies
- it keeps Parquet nesting shallower than a fully nested list-of-list-of-list
  encoding

### GeoArrow compatibility

For non-volumetric geometries, the normalized boundary can be transformed into a
standard GeoArrow geometry column by:

- prefix-summing lengths back into offsets
- dereferencing vertex indices into coordinates
- emitting GeoArrow-native coordinate buffers

For `Solid` and `MultiSolid`, the shell and solid hierarchy remains a
CityJSON-specific sidecar rather than pretending that the shape is already a
standard GeoArrow primitive.

### Semantics, materials, and textures

Surface-aligned and ring-aligned sidecar arrays should stay parallel to the
normalized topology.

That means:

- semantic references align to surface order
- material assignments align to surface order
- texture assignments align to ring order

That UV layout is now implemented canonically through:

- `texture_vertices`
- `geometry_ring_textures.uv_indices`

Texture image references alone are not enough; the UV index mapping remains a
first-class part of the canonical package.

## Coordinates And Transform

Current `cityjson-rs` stores world coordinates directly and carries `transform`
as model metadata when present.

`cityarrow` should therefore:

- serialize vertex buffers from current coordinate storage
- round-trip the transform as explicit metadata
- stop assuming that vertex buffers are always quantized integers

If a future specialized export wants quantized integer coordinate columns for
size or interoperability reasons, that should be an explicit transport policy,
not an assumption baked into the core conversion path.

## Reader And Writer Scope

The canonical Parquet package path is symmetrical today.

### Implemented writer behavior

- Parquet directory output

### Implemented reader behavior

- Parquet directory input
- reconstruction into `CityModel`

Arrow IPC packaging remains separate work. The important design point is that
all supported readers reconstruct the same transport decomposition and then the
same semantic `CityModel`.

## Public API Alignment With `cjlib`

The neighboring `cjlib` docs establish the right boundary rule:

- explicit format modules
- one semantic unit across formats
- no generic codec registry

`cityarrow` should therefore optimize for APIs that can sit naturally behind:

- `cjlib::arrow`
- `cjlib::parquet`

That implies:

- file helpers should read and write `CityModel`
- streams should yield `CityModel` or explicit streams of `CityModel`
- `CityModelArrowParts` should remain available for advanced use, but it should
  not become the primary semantic facade

## Rewrite Strategy

The next implementation should be a direct rewrite in the target architecture,
not a staged migration from the current prototype.

### Rewrite constraints

- update the entire crate to `cityjson-rs v0.4.1` consistently instead of
  carrying dual API assumptions
- remove every `AttributePool` and Arrow-union-centered path rather than trying
  to preserve them
- rebuild vertices, transform, geometry, and semantic conversion around the
  current source model
- define the target Arrow and Parquet schemas once, then make both readers and
  writers conform to them
- delete obsolete tests and disabled code that encode the wrong data model
- build the acceptance suite around the same manifest-driven case setup used by
  `serde_cityjson`, including its generated profile catalog and downloaded
  real-world datasets

### Rewrite acceptance criteria

The rewrite is done when all of the following are true:

- the crate compiles against current `cityjson-rs`
- `CityModel -> CityModelArrowParts -> CityModel` works for the supported
  component set
- the roundtrip suite uses the same setup as `serde_cityjson`, centered on the
  manifest-driven case catalog and real dataset layout rather than ad hoc local
  fixtures
- Arrow IPC and Parquet use the same target transport decomposition
- attributes and extras use the projected Parquet-safe encoding defined above
- geometry export uses the normalized transport layout defined above
- the accepted roundtrip path ends by serializing the reconstructed model to
  JSON with `serde_cityjson`
- the real-world `3DBAG` and `3D Basisvoorziening` cases are serialized to JSON
  with `serde_cityjson` at the very end and those produced files validate with
  `cjval`
- the test suite validates the new architecture instead of preserving the old
  one

## Non-goals

The redesign should not:

- create a second semantic model beside `CityModel`
- force `cityjson-rs` to adopt a transport-native in-memory representation
- make Arrow union types the center of the Parquet design
- hide format choice behind a generic `read` or `write` registry

## Decision Summary

The main conclusions from the review are straightforward.

- `cityarrow` should be rebased to the current inline-attribute `cityjson-rs`
  model, not the removed pool-based one.
- the Parquet attribute gap should be solved with schema discovery and projected
  columns, not with Arrow union types.
- geometry should be normalized at the serialization boundary instead of pushing
  a GeoArrow-friendly layout into `cityjson-rs` internals.
- the crate should align with `cjlib` by keeping `CityModel` as the public
  semantic unit across Arrow and Parquet.
