# cityarrow

`cityarrow` is the Arrow, Arrow IPC, and Parquet transport layer for
`cityjson-rs`.

The semantic unit remains `cityjson::v2_0::OwnedCityModel`.
`CityModelArrowParts` is the canonical transport decomposition used at the
package boundary; it is not a second semantic model.

## Implemented Surface

The crate currently provides:

- `convert::to_parts` and `convert::from_parts`
- canonical package write/read for Parquet
- canonical package write/read for Arrow IPC file
- schema-locked canonical tables and manifest layout for
  `cityarrow.package.v1alpha1`
- exact package roundtrip coverage for both supported encodings

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
- point, linestring, and surface material assignments
- geometry and template ring texture assignments

## Public API

The top-level crate exports:

- `to_parts` and `from_parts`
- `write_package_dir` and `read_package_dir` for Parquet packages
- `write_package_ipc_dir` and `read_package_ipc_dir` for Arrow IPC packages
- schema and manifest types from `src/schema.rs`

## Verification

The repository keeps four test layers around the canonical package path:

1. In-memory `to_parts`/`from_parts` roundtrip tests for synthetic fixtures.
2. Exact canonical table equality tests for Parquet and Arrow IPC package
   roundtrips.
3. Fast fixture tests that verify package I/O preserves canonical parts for
   both encodings and still reconstructs `cjval`-valid CityJSON.
4. Opt-in real-data acceptance tests for `3DBAG` and `3D Basisvoorziening`
   covering exact package roundtrip equality and exact normalized model
   equality for both encodings.

The real-data acceptance tests stay `#[ignore]` because they are materially
more expensive than the regular suite.

## Documentation

- [docs/design.md](docs/design.md): transport design and invariants
- [docs/adr/001-canonical-transport-boundary.md](docs/adr/001-canonical-transport-boundary.md):
  accepted ADR for the current transport architecture
- [docs/package-schema.md](docs/package-schema.md): canonical package layout and
  manifest contract

## Repository Map

- `src/lib.rs`: public API entry points
- `src/convert/mod.rs`: model-to-parts and parts-to-model conversion
- `src/package/`: package manifest plus Parquet and Arrow IPC read/write
- `src/schema.rs`: canonical schema definitions and transport structs
- `tests/`: conversion, package, schema, and real-data roundtrip coverage
