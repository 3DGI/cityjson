# cityarrow Design

This document defines the current transport design for `cityarrow`.

The concrete package schema is documented in
[docs/package-schema.md](package-schema.md). Current implementation status and
verification gates are documented in [docs/status.md](status.md).

## Purpose

`cityarrow` moves `cityjson-rs` models across Arrow IPC and Parquet boundaries.

It does not define a second semantic model family. The semantic core remains
`cityjson::v2_0::CityModel`, and the transport decomposition is
`CityModelArrowParts`.

## Source Model

The current `cityjson-rs` model drives the transport design.

- Attributes are stored inline on semantic owners such as model extra,
  metadata extra, cityobject attributes, cityobject extra, and semantic
  attributes.
- Shared resources remain pooled for geometries, semantics, materials,
  textures, and template geometries.
- Geometry boundaries are stored in offset-based flattened structures in memory.

`cityarrow` preserves that semantic structure while normalizing it into a
package shape that is easier to write, read, and validate across Arrow IPC and
Parquet.

## Design Rules

1. `cityarrow` is a format boundary, not a semantic fork.
2. The public API trades in `CityModel` and explicit package helpers, not a
   transport-native semantic replacement.
3. `CityModelArrowParts` is the canonical transport decomposition.
4. Canonical package schemas must be valid for both Arrow IPC and Parquet.
5. Package changes must be deliberate, documented, and schema-locked in tests.
6. Reconstruction must use explicit ids and ordinals instead of depending on
   row order.

## Transport Decomposition

The canonical package decomposes a model into component tables that mirror the
main ownership boundaries in `cityjson-rs`:

- metadata and transform
- extensions
- vertices and template vertices
- cityobjects and parent/child relations
- geometries, geometry boundaries, and geometry instances
- semantics plus geometry/template assignment tables
- materials plus geometry/template assignment tables
- textures, texture vertices, geometry ring textures, and template ring textures

This keeps shared resources explicit and keeps topology and appearance
assignments reconstructible through relational joins and ordinals.

## Attribute Encoding

Attributes are projected into flat sibling columns on the owning table.

- primitive values map to native Arrow scalar columns
- nested arrays and maps fall back to JSON text columns
- geometry-valued attributes may be carried as geometry handles and optional
  derived binary views
- mixed-type values for the same logical key are coerced to a lossless text
  representation instead of relying on Arrow unions

This keeps the canonical schema Parquet-safe while remaining reconstructible.

## Geometry Encoding

Geometry remains offset-based in memory and normalized at the transport
boundary.

The canonical package writes:

- `vertex_indices`
- `line_lengths`
- `ring_lengths`
- `surface_lengths`
- `shell_lengths`
- `solid_lengths`

This normalization keeps topology explicit, keeps Parquet nesting shallow, and
supports later derived GIS views without forcing a GIS-native in-memory model
onto `cityjson-rs`.

Semantics, materials, and textures remain parallel sidecars over that normalized
topology:

- point assignments align to point order
- linestring assignments align to linestring order
- surface assignments align to surface order
- ring texture assignments align to ring order
- template assignments use explicit primitive type plus primitive ordinal

## Coordinates And Transform

`cityarrow` serializes the world-coordinate vertex buffers that `cityjson-rs`
stores today and round-trips `transform` as explicit metadata when present.

The canonical package does not assume quantized integer vertices. Any future
quantized export would be a separate transport policy, not the canonical model.

## Reader And Writer Scope

The implemented canonical package path is symmetric:

- `convert::to_parts`
- package write to Parquet or Arrow IPC
- package read from Parquet or Arrow IPC
- `convert::from_parts`

All supported readers reconstruct the same `CityModelArrowParts` shape and then
the same semantic `CityModel`.

## Non-goals

The canonical package does not try to:

- create a second semantic model beside `CityModel`
- force `cityjson-rs` to adopt a transport-native in-memory representation
- make Arrow union types part of the canonical design
- collapse the full CityJSON model into one generic GIS table
- hide format choice behind a generic registry abstraction

## Current Follow-on Work

The canonical package path is implemented. The remaining work is downstream of
that package:

- derived GeoArrow and GeoParquet views
- selected wider typed projections for query-oriented workloads
