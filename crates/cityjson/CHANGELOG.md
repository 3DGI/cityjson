# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and this project follows Semantic Versioning.

## [0.7.2] - 2026-04-22

### Added
- `CityModel::calculate_geographical_extent()` for calculating the union extent of all city objects from directly attached geometry.
- `CityModel::calculate_cityobject_geographical_extent(...)` for calculating a single city object's extent without reading or mutating stored `geographicalExtent` values.
- `GeometryInstance` extent calculation through template geometry resolution, row-major affine transformation, and reference point offsets.

## [0.7.0] - 2026-04-16

### Added
- Borrowed `ModelRelationalView` for zero-copy relational access over `OwnedCityModel`.
- Owned `OwnedRelationalSnapshot` materialization for import and table-oriented rebuild paths.
- Stable remap and raw-access seams that downstream Arrow and facade crates can consume directly.

### Changed
- `relational()` now returns the borrowed view instead of a materialized snapshot.
- Owned relational materialization moved behind `relational_snapshot()`.
- Query and downstream integration paths now align on the relational vNext contract.

## [0.6.0] - 2026-04-07

### Added
- Checked bulk vertex append APIs on `Vertices` and `CityModel`.
- Unsafe trusted-construction support for flat `Boundary` buffers.
- Serialization-oriented raw handle part access and trusted reconstruction helpers.
- Low-level tests covering raw boundary offset layers and trusted import paths.

## [0.4.0] - 2026-02-11

### Added
- Release-readiness hardening for public API visibility and semver safety.
- `Cargo.toml` package excludes for release artifacts.

### Changed
- Replaced `cityjson::core` wildcard re-exports with explicit curated exports.
- Moved low-level `resources::pool` module to crate-internal visibility.
- Marked additional public enums as `#[non_exhaustive]` (`CityJSON`, `BuilderMode`, `AttributeValue`).
- Updated `Error` formatting to a single exhaustive `match`.
- Eliminated per-call allocations in `SemanticMap`/`MaterialMap` read accessors.
- Switched `CityObjects::ids()` and `CityObjects::filter()` to iterator-returning APIs.
- Updated transform/extension documentation examples to public v2.0 API paths.
- Refreshed README content for release quality.
