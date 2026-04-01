# cityarrow

`cityarrow` is the Arrow and Arrow IPC transport layer for `cityjson-rs`.

`cityparquet` is the sibling crate for persistent package I/O.

The semantic unit remains `cityjson::v2_0::OwnedCityModel`.
The canonical table decomposition is an internal transport contract, not the
public API boundary.

## Implemented Surface

The crate currently provides:

- `ModelEncoder` and `ModelDecoder` for live Arrow IPC stream transport
- schema-locked canonical tables and manifest layout for
  `cityarrow.package.v2alpha1`
- internal canonical table handling shared with `cityparquet`

The canonical package surface includes:

- metadata, transform, extensions, and projected extra fields
- vertices and template vertices
- cityobjects plus parent/child relations
- projected cityobject and semantic attributes
- boundary-carrying geometries with normalized topology
- geometry instances and template geometries
- point, linestring, surface, and template semantic assignments
- materials, textures, texture vertices, default appearance themes, and UV
  mappings
- surface and template material assignments
- geometry and template ring texture assignments

## Implementation Status

`cityarrow` is implemented enough to do exact end-to-end transport roundtrips,
but it is still an alpha transport surface.

- correctness: the canonical conversion and package paths are implemented for
  the live stream path in `cityarrow` and the persistent package path in
  `cityparquet`
- scope: the canonical package covers the current `OwnedCityModel` surface used
  by `cityjson-rs`, including templates, geometry instances, semantics,
  materials, textures, metadata, and projected attributes
- stability: the on-disk contract is currently `cityarrow.package.v2alpha1`,
  so compatibility should be treated as deliberate but not yet stable
- performance: the current conversion and package read paths are eager and
  fully in-memory; broad corpus roundtrips are therefore memory intensive

Known limitations in the current implementation:

- the single-file package container currently lives behind `cityparquet`, not
  the `cityarrow` top-level API
- template geometry pools cannot themselves contain geometry instances
- texture mappings are only supported on surface-backed geometry types
- the current implementation prioritizes exactness and schema clarity over
  streaming or low-memory operation

## Public API

The top-level crate exports:

- `ModelEncoder` and `ModelDecoder`
- schema and manifest types from `src/schema.rs`

`cityparquet` exports:

- `PackageWriter` and `PackageReader`
- the shared package manifest and schema types

## Verification

The repository keeps four test layers around the canonical package path:

1. Live Arrow stream roundtrip tests in `cityarrow`.
2. Persistent package roundtrip tests through `cityparquet`.

## Documentation

- [mkdocs.yml](mkdocs.yml): MkDocs Material site configuration
- [docs/index.md](docs/index.md): site landing page
- [docs/cityarrow.md](docs/cityarrow.md): user-facing `cityarrow` overview
- [docs/cityparquet.md](docs/cityparquet.md): user-facing `cityparquet` overview
- [docs/cityjson-arrow-ipc-spec.md](docs/cityjson-arrow-ipc-spec.md): Arrow
  IPC package layout specification
- [docs/cityjson-parquet-spec.md](docs/cityjson-parquet-spec.md): Parquet
  package layout specification
- [docs/package-schema.md](docs/package-schema.md): shared package overview and
  manifest contract
- [docs/design.md](docs/design.md): transport design and invariants
- [docs/adr/001-canonical-transport-boundary.md](docs/adr/001-canonical-transport-boundary.md):
  accepted ADR for the current transport architecture
- [STATUS.md](STATUS.md): current implementation status and readiness review

## Repository Map

- `src/lib.rs`: public API entry points
- `src/convert/mod.rs`: model-to-parts and parts-to-model conversion
- `src/transport.rs`: shared canonical-table transport helpers
- `cityparquet/src/package/`: persistent package/container implementation
- `src/schema.rs`: canonical schema definitions and transport structs
- `tests/`: conversion, package, schema, and shared-corpus roundtrip coverage
