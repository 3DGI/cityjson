# cityarrow

`cityarrow` is the Arrow, Arrow IPC, and Parquet transport layer for the
`cityjson-rs` data model.

It does not define a second semantic model. The semantic unit remains
`cityjson::v2_0::OwnedCityModel`; Arrow IPC and Parquet are explicit transport
boundaries around that model.

## Status

The canonical package path is implemented for both Parquet and Arrow IPC file
storage.

The crate currently provides:

- `convert::to_parts` and `convert::from_parts`
- canonical package write/read for Parquet
- canonical package write/read for Arrow IPC
- schema-locked canonical tables and manifest layout
- package-level roundtrip coverage for exact canonical table equality

The current status and verification gates are documented in
[docs/status.md](docs/status.md).

## Canonical Surface

The supported canonical package surface includes:

- metadata, transform, extensions, and projected extra fields
- vertices and template vertices
- cityobjects plus parent/child relations
- projected cityobject and semantic attributes
- boundary-carrying geometries with normalized topology
- geometry instances and template geometries
- point, linestring, surface, and template semantic assignments
- materials, textures, texture vertices, default appearance themes, and UV
  mappings
- point, linestring, and surface material assignments
- geometry and template ring texture assignments

## Design Constraints

- `CityModel` remains the semantic unit
- `CityModelArrowParts` remains a transport decomposition
- canonical package schemas must remain Parquet-safe
- canonical topology stays normalized and reconstructible
- package schema changes must stay deliberate and schema-locked

## Current Documentation

- [docs/status.md](docs/status.md): implementation status and verification
- [docs/design.md](docs/design.md): transport design and invariants
- [docs/package-schema.md](docs/package-schema.md): canonical package schema

## Repository Map

- `src/lib.rs`: public API entry points
- `src/convert/mod.rs`: model-to-parts and parts-to-model conversion
- `src/package/`: package manifest plus Parquet and Arrow IPC read/write
- `src/schema.rs`: canonical schema definitions and transport structs
