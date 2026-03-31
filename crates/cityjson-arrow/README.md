# cityarrow

`cityarrow` is the Arrow and Parquet transport layer for the `cityjson-rs`
data model.

It should not define a second semantic model. The semantic unit remains
`cityjson::v2_0::CityModel`; Arrow IPC, Parquet, and stream framing are explicit
format boundaries built around that model.

## Status

As of 2026-03-31, the canonical package path is implemented for the current
supported surface and the crate is working against the local `cityjson-rs`
checkout.

Verified in this repository:

- `cargo test` passes
- `cargo test -- --ignored` passes
- the ignored acceptance gate round-trips the real `3DBAG` and
  `3D Basisvoorziening` cases through `serde_cityjson -> cityarrow -> cjval`

The crate should now be understood as a working transport boundary around
`cityjson::v2_0::OwnedCityModel`, not as a stale prototype.

## Implemented Surface

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

## Supported Surface

The canonical roundtrip currently supports:

- metadata, transform, extensions, and model extra
- vertices and template vertices
- cityobjects plus parent/child relations
- projected cityobject and semantic attributes
- boundary-carrying geometries with normalized topology
- geometry instances and template geometries
- semantic point, linestring, surface, and template assignments
- materials, textures, UV coordinates, default appearance themes,
  point/linestring/surface material assignments, and template ring texture
  assignments

## Current Priorities

- keep `cityarrow.package.v1alpha1` stable and schema-locked in tests
- keep projected attribute columns conservative and lossless
- add derived GeoArrow and GeoParquet views as secondary exports
- add Arrow IPC package read/write as a separate transport path

## Design Constraints

- `CityModel` remains the semantic unit
- Arrow and Parquet remain explicit format boundaries
- `CityModelArrowParts` remains a transport decomposition, not a second data model
- canonical Parquet schemas must avoid Arrow `Union` and `Map`
- attributes stay reconstructible even when projected conservatively
- normalized topology is the canonical package shape; GIS views remain derived

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
  - canonical package schema

## Documentation

The current design document is [docs/design.md](docs/design.md).
The current schema document is [docs/package-schema.md](docs/package-schema.md).
