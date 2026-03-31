# cityarrow Status

This file records the current implementation state of the canonical
`cityarrow.package.v1alpha1` transport.

## Current State

The canonical package path is implemented for both supported table encodings:

- Parquet
- Arrow IPC file

The implemented path is:

- `convert::to_parts`
- `package::{write_package_dir, write_package_ipc_dir}`
- `package::{read_package_dir, read_package_ipc_dir}`
- `convert::from_parts`

The semantic unit remains `cityjson::v2_0::OwnedCityModel`. `CityModelArrowParts`
is the canonical transport decomposition, not a second semantic model.

## Canonical Surface

The current canonical package supports:

- metadata, transform, extensions, and projected extra fields
- vertices and template vertices
- cityobjects plus parent/child relations
- projected cityobject and semantic attributes
- boundary-carrying geometries with normalized topology
- geometry instances and template geometries
- point, linestring, surface, and template semantic assignments
- materials, textures, texture vertices, and default appearance themes
- point, linestring, and surface material assignments
- geometry and template ring texture assignments

## Verification

The repository carries four layers of test coverage around the package path:

1. In-memory `to_parts`/`from_parts` roundtrip tests for synthetic fixtures.
2. Exact canonical table equality tests for Parquet and Arrow IPC package
   roundtrips.
3. Fast fixture tests that verify package I/O preserves canonical parts for both
   encodings and still reconstructs `cjval`-valid CityJSON.
4. Opt-in real-data acceptance tests for `3DBAG` and `3D Basisvoorziening`
   covering exact package roundtrip equality and exact normalized model equality
   for both encodings.

The real-data tests are intentionally `#[ignore]` because they are expensive and
large enough to require explicit execution.

## Stability Rules

- `cityarrow.package.v1alpha1` is schema-locked in tests.
- Package-surface changes must remain deliberate and documented.
- Reconstruction must use explicit ids and ordinals rather than implicit row
  order.
- If a feature is not implemented, conversion must return an explicit error
  instead of silently dropping data.

## Next Work

The main work that remains outside the canonical package path is:

- derived GeoArrow and GeoParquet views
- selected wider typed projections for query-oriented attribute access
