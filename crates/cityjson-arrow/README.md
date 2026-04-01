# cityarrow

`cityarrow` is the Arrow and Arrow IPC transport layer for `cityjson-rs`.

`cityparquet` is the sibling crate for Parquet package I/O.

The semantic unit remains `cityjson::v2_0::OwnedCityModel`.
`CityModelArrowParts` is the canonical transport decomposition used at the
package boundary; it is not a second semantic model.

## Implemented Surface

The crate currently provides:

- `convert::to_parts` and `convert::from_parts`
- canonical package write/read for Arrow IPC file
- schema-locked canonical tables and manifest layout for
  `cityarrow.package.v1alpha1`
- exact package roundtrip coverage for Arrow IPC and Parquet via `cityparquet`

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
  both Parquet and Arrow IPC file and are covered by schema-lock, package, and
  shared-corpus correctness tests
- scope: the canonical package covers the current `OwnedCityModel` surface used
  by `cityjson-rs`, including templates, geometry instances, semantics,
  materials, textures, metadata, and projected attributes
- stability: the on-disk contract is currently `cityarrow.package.v1alpha1`,
  so compatibility should be treated as deliberate but not yet stable
- performance: the current conversion and package read paths are eager and
  fully in-memory; broad corpus roundtrips are therefore memory intensive

Known limitations in the current implementation:

- package helpers round-trip the canonical tables; manifest `views` are treated
  as optional non-canonical metadata
- template geometry pools cannot themselves contain geometry instances
- texture mappings are only supported on surface-backed geometry types
- the current implementation prioritizes exactness and schema clarity over
  streaming or low-memory operation

## Public API

The top-level crate exports:

- `to_parts` and `from_parts`
- `write_package_ipc_dir` and `read_package_ipc_dir` for Arrow IPC packages
- `cityparquet::write_package_dir` and `cityparquet::read_package_dir` for Parquet packages
- schema and manifest types from `src/schema.rs`

## Verification

The repository keeps four test layers around the canonical package path:

1. In-memory `to_parts`/`from_parts` roundtrip tests for synthetic fixtures.
2. Exact canonical table equality tests for Arrow IPC package roundtrips in
   `cityarrow` and Parquet package roundtrips in `cityparquet`.
3. Fast fixture tests that verify package I/O preserves canonical parts for
   both encodings and still reconstructs `cjval`-valid CityJSON.
4. Shared corpus conformance tests that roundtrip the same CityJSON 2.0
   correctness fixtures used by `serde_cityjson` through both Parquet and Arrow
   IPC packages.

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
- `src/package/`: package manifest plus Arrow IPC read/write
- `src/schema.rs`: canonical schema definitions and transport structs
- `tests/`: conversion, package, schema, and shared-corpus roundtrip coverage
