# Cityarrow Implementation Plan

This file is the execution plan for the rewrite that implements the canonical
`cityarrow.package.v1alpha1` schema. Transitional compatibility with the old
prototype is not a goal.

## Goal

Implement a lossless canonical roundtrip:

`cityjson::v2_0::OwnedCityModel -> CityModelArrowParts -> Parquet package -> CityModelArrowParts -> OwnedCityModel`

The package remains the semantic/topological source of truth. GeoArrow and
GeoParquet feature tables remain derived views.

## Acceptance

The rewrite is accepted only when all of the following are true:

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

## Scope Order

### Phase 1: Canonical Core

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
- geometry-to-surface semantic assignments

### Phase 2: Deferred Modules

- materials
- textures
- template vertices
- template geometries
- geometry instances

Deferred modules must fail explicitly when encountered. They are not allowed to
silently disappear from the roundtrip.

## Implementation Tasks

### Task List

- [x] Replace the disconnected prototype layout with clean public modules:
  - `src/convert/`
  - `src/package/`
- [x] Remove stale prototype files that are no longer part of the rewrite.
- [x] Implement `convert::to_parts`.
- [x] Implement `convert::from_parts`.
- [x] Implement normalized boundary flattening and reconstruction for:
  - `MultiPoint`
  - `MultiLineString`
  - `MultiSurface`
  - `CompositeSurface`
  - `Solid`
  - `MultiSolid`
  - `CompositeSolid`
- [x] Implement canonical metadata, transform, and extension table conversion.
- [x] Implement canonical cityobject table conversion.
- [x] Implement canonical semantics table conversion.
- [x] Implement Parquet-safe attribute projection.
- [x] Preserve exact attribute values with per-key JSON fallback columns.
- [x] Preserve explicit null versus missing value distinction in fallback columns.
- [x] Reject unsupported modules with hard errors:
  - materials
  - textures
  - templates
  - geometry instances
- [x] Implement package manifest writing.
- [x] Implement Parquet table writing.
- [x] Implement package manifest reading.
- [x] Implement Parquet table reading.
- [x] Validate on package read that batch schemas match the canonical schema set.
- [x] Replace the identity placeholder in `tests/manifest_roundtrip.rs`.
- [x] Add focused unit tests for conversion helpers.
- [x] Add integration tests for package write/read roundtrip.
- [x] Run the full test stack, including ignored real-data acceptance.

## Design Constraints

- The canonical package must not use Arrow `Union`.
- The canonical package must not use Arrow `Map`.
- Boundary topology must stay in normalized flat arrays, not nested Arrow lists.
- Reconstruction must use explicit ids and ordinals, not implicit row order.
- Attributes must remain reconstructible even when the projected query columns are
  conservative JSON fallback columns.
- If a feature is not implemented, the code must return an explicit error instead
  of dropping data.

## Current Execution Split

- Main trunk: conversion rewrite, repo cleanup, integration, final verification.
- Parallel worktree A: package manifest and Parquet I/O.
- Parallel worktree B: harness and integration test updates around the new API.
