# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and this project follows Semantic Versioning.

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
