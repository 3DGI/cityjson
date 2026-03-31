# Cityarrow Implementation Status

This file records the current implementation state of the canonical
`cityarrow.package.v1alpha1` package.

## Status

The canonical Parquet package path is implemented and verified.

- `convert::to_parts` is implemented
- `convert::from_parts` is implemented
- package manifest write/read is implemented
- Parquet table write/read is implemented
- the real acceptance path is `serde_cityjson -> cityarrow -> serde_cityjson -> cjval`

## Verified Gates

1. Canonical in-memory roundtrip is lossless for the supported package surface.
2. Canonical package write/read roundtrip is lossless for the supported package surface.
3. The acceptance harness uses the same manifest-driven case setup as
   `~/Development/serde_cityjson`.
4. The final acceptance path is:
   - parse with `serde_cityjson`
   - convert through `cityarrow`
   - serialize the reconstructed model with `serde_cityjson`
   - validate the emitted JSON with `cjval`
5. The real-world datasets `3DBAG` and `3D Basisvoorziening` pass that final gate.

## Implemented Canonical Surface

- metadata
- transform
- extensions
- vertices
- cityobjects
- cityobject children/parents
- projected attributes and extra fields
- geometries
- normalized geometry boundaries
- semantics
- geometry-to-primitive semantic assignments
- template-geometry semantic assignments
- materials
- textures
- texture vertices
- geometry point, linestring, and surface materials
- geometry ring textures
- template geometry materials
- template geometry ring textures
- default appearance themes

## Design Constraints

- The canonical package must not use Arrow `Union`.
- The canonical package must not use Arrow `Map`.
- Boundary topology must stay in normalized flat arrays, not nested Arrow lists.
- Reconstruction must use explicit ids and ordinals, not implicit row order.
- Attributes must remain reconstructible even when the projected query columns are
  conservative JSON fallback columns.
- If a feature is not implemented, the code must return an explicit error instead
  of dropping data.

## Separate Work

- derived GeoArrow and GeoParquet feature views
- Arrow IPC package read/write
- richer typed projections for selected high-value attribute columns
