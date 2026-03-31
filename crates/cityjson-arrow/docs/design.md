# cityarrow Design

This document defines the transport design and invariants for `cityarrow`.

The concrete on-disk contract lives in
[docs/package-schema.md](package-schema.md).

## Purpose

`cityarrow` moves `cityjson-rs` models across Arrow IPC and Parquet boundaries.

It does not define a second semantic model family. The semantic core remains
`cityjson::v2_0::OwnedCityModel`, and the transport decomposition is
`CityModelArrowParts`.

## Source Model

The transport design follows the ownership structure already present in
`cityjson-rs`.

- attributes remain attached to semantic owners such as metadata, cityobjects,
  semantics, materials, and textures
- shared resources remain pooled for geometries, semantics, materials,
  textures, and template geometries
- geometry boundaries remain offset-based and flattened for reconstruction

`cityarrow` preserves that semantic structure while normalizing it into a
package shape that is explicit, schema-checkable, and reconstructible across
both supported storage encodings.

## Design Rules

1. `cityarrow` is a transport boundary, not a semantic fork.
2. The public API trades in `OwnedCityModel`, package helpers, and explicit
   transport structs.
3. `CityModelArrowParts` is the canonical transport decomposition.
4. Canonical package schemas must be valid for both Parquet and Arrow IPC file.
5. Package changes must be deliberate, documented, and schema-locked in tests.
6. Reconstruction must use explicit ids and ordinals instead of row order.

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
- textures, texture vertices, geometry ring textures, and template ring
  textures

This keeps shared resources explicit and keeps topology and appearance
assignments reconstructible through joins and ordinals instead of opaque nested
payloads.

## Attribute Encoding

Attributes are projected into flat sibling columns on the owning table.

- primitive values map to native Arrow scalar columns
- nested arrays and maps fall back to JSON text columns
- mixed-type logical keys fall back to lossless text instead of Arrow unions
- geometry references may be carried as explicit geometry ids

This keeps the canonical schema Parquet-safe while remaining losslessly
reconstructible.

## Geometry Encoding

Geometry remains offset-based in memory and is normalized at the transport
boundary.

The canonical package writes boundary topology through explicit sidecars such as
`vertex_indices`, `line_lengths`, `ring_lengths`, `surface_lengths`,
`shell_lengths`, and `solid_lengths`.

Semantics, materials, and textures remain parallel sidecars over that topology:

- point assignments align to point order
- linestring assignments align to linestring order
- surface assignments align to surface order
- ring texture assignments align to ring order
- template assignments use explicit primitive type plus primitive ordinal

## Coordinates And Transform

`cityarrow` serializes the world-coordinate vertex buffers stored by
`cityjson-rs` and round-trips `transform` as explicit metadata when present.

The canonical package does not assume quantized integer vertices.

## Reader And Writer Scope

The implemented package path is symmetric:

- `convert::to_parts`
- package write to Parquet or Arrow IPC file
- package read from Parquet or Arrow IPC file
- `convert::from_parts`

All supported readers reconstruct the same `CityModelArrowParts` shape and then
the same semantic `OwnedCityModel`.

## Non-goals

The canonical package does not try to:

- create a second semantic model beside `OwnedCityModel`
- force `cityjson-rs` to adopt a transport-native in-memory representation
- make Arrow union types part of the canonical contract
- collapse the full CityJSON model into one generic GIS table
- hide format choice behind a registry or plugin abstraction
