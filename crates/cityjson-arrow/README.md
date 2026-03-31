# cityarrow

`cityarrow` is the Arrow and Parquet transport layer for the `cityjson-rs`
data model.

It should not define a second semantic model. The semantic unit remains
`cityjson::v2_0::CityModel`; Arrow IPC, Parquet, and stream framing are explicit
format boundaries built around that model.

## Status

As of 2026-03-31, the rewrite is implemented for the current canonical package
surface and the crate is working against the local `cityjson-rs` checkout.

Verified in this repository:

- `cargo test` passes
- `cargo test -- --ignored` passes
- the ignored acceptance gate round-trips the real `3DBAG` and
  `3D Basisvoorziening` cases through `serde_cityjson -> cityarrow -> cjval`

The crate should now be understood as a working transport boundary around
`cityjson::v2_0::OwnedCityModel`, not as a stale prototype.

## What Exists Today

The current codebase implements the canonical Parquet package path:

- `CityModelArrowParts` splits a `CityModel` into canonical component batches
- `convert::to_parts` exports `OwnedCityModel` into canonical batches
- `convert::from_parts` reconstructs `OwnedCityModel` from canonical batches
- `package::write_package_dir` writes canonical Parquet tables plus `manifest.json`
- `package::read_package_dir` reads the package back and validates schema shape
- metadata, transform, extensions, vertices, cityobjects, geometries, semantics,
  template geometries, geometry instances, materials, textures, and UV
  coordinates all round-trip for the supported surface
- the final acceptance path is wired through `serde_cityjson` and `cjval`

## Current Gap Summary

| Area | Current State | Gap |
| --- | --- | --- |
| Canonical Parquet package | Implemented and tested | Keep the schema stable while extending coverage |
| Attributes and extra | Implemented as Parquet-safe JSON fallback projections | Future work can widen selected fields to richer typed projections |
| Vertices and transform | Implemented against current `RealWorldCoordinate`-based APIs | None on the canonical path |
| Geometries | Implemented with normalized boundary flattening and reconstruction | Derived GeoArrow and GeoParquet views remain separate work |
| Semantics | Implemented for surface-based mappings | Point and linestring semantic mappings remain unsupported |
| Appearance | Materials, textures, UV coordinates, default themes, surface materials, and ring textures round-trip | Point and linestring material mappings remain unsupported; template-geometry appearance remains unsupported |
| Reading | Parquet package read is implemented and validated | Arrow IPC package read/write is still a separate future path |
| Tests | Unit, integration, and ignored acceptance gates pass | Keep expanding fixtures around unsupported edge cases |

## Design Direction

The redesign should follow four rules.

1. `CityModel` stays the semantic unit.
2. Arrow and Parquet stay explicit transport boundaries.
3. `CityModelArrowParts` is an internal transport decomposition, not a new data
   model family.
4. Parquet support must be designed around Parquet's real constraints, not
   around Arrow-only features such as unions.

The implementation stance is intentionally aggressive:

- prefer a clean rewrite over compatibility shims
- delete stale assumptions instead of adapting around them
- accept breaking changes if they produce a cleaner transport boundary
- only preserve code paths that fit the target architecture

Two design consequences follow from the current review.

### 1. Attributes must be projected, not pool-backed

`cityjson-rs v0.4.1` stores attributes inline again through
`AttributeValue::{Null, Bool, Unsigned, Integer, Float, String, Vec, Map, Geometry}`.

The old `cityarrow` approach tried to model attributes as:

- Arrow `Map<Utf8, Union>`
- then Parquet

That is the wrong seam. Parquet still cannot represent Arrow unions natively,
and the in-memory `AttributePool` path used by the current crate no longer
matches `cityjson-rs`.

The replacement direction is:

- discover attribute keys per owner scope
- resolve a Parquet-safe physical type per key
- write typed sibling columns
- encode nested `Vec` and `Map` values as JSON strings
- encode geometry-valued attributes as a handle or optional WKB shadow column

### 2. Geometry export should normalize at the boundary

Current `cityjson-rs` still stores boundaries in an offset-based flattened
structure:

- `vertices`
- `rings`
- `surfaces`
- `shells`
- `solids`

That is appropriate for in-memory semantics, but Arrow and Parquet benefit from
a more serialization-oriented layout. `cityarrow` should therefore normalize the
boundary representation at export time instead of forcing `cityjson-rs` to adopt
a transport-native shape internally.

The intended direction is:

- keep the current `cityjson-rs` boundary as the source of truth
- derive a GeoArrow-friendlier count-based layout at the Arrow/Parquet boundary
- keep CityJSON-specific shell and solid hierarchy as sidecar data

## Fit In The Wider Stack

`cityarrow` should align with the neighboring crate architecture documented in
`cjlib`.

- `cityjson-rs` owns semantic correctness and invariants
- `cityarrow` owns the Arrow transport boundary
- a future `cityparquet` can either be a sibling crate or a narrower facade on
  top of the Parquet parts of `cityarrow`
- `cjlib::arrow` and `cjlib::parquet` should expose explicit format modules that
  read and write `CityModel`, not transport-native fragments

## Supported Surface

The canonical roundtrip currently supports:

- metadata, transform, extensions, and model extra
- vertices and template vertices
- cityobjects plus parent/child relations
- projected cityobject and semantic attributes
- boundary-carrying geometries with normalized topology
- geometry instances and template geometries
- semantic surface assignments
- materials, textures, UV coordinates, default appearance themes,
  surface material assignments, and ring texture assignments

The canonical roundtrip still rejects these inputs explicitly:

- point and linestring semantic mappings
- point and linestring material mappings
- template geometry semantics
- template geometry materials
- template geometry textures

## Repository Map

- `src/lib.rs`
  - top-level `CityModelArrowParts` and conversion entry points
- `src/convert/mod.rs`
  - model-to-parts and parts-to-model conversion
- `src/package/`
  - canonical package manifest, Parquet write, and Parquet read
- `src/schema.rs`
  - canonical schema definitions and transport structs
- `docs/design.md`
  - current design and scope notes
- `docs/package-schema.md`
  - canonical package schema draft

## Design Doc

The current design document is [docs/design.md](docs/design.md).
The current schema draft is [docs/package-schema.md](docs/package-schema.md).
