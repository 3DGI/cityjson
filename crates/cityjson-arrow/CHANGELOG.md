# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and this project follows Semantic Versioning.

## [Unreleased]

### Changed
- Updated the core crate import path from `cityjson::...` to
  `cityjson_types::...` after the `cityjson-types` package rename.

## [0.6.0] - 2026-04-16

### Added
- Thin batch and stream codec surface rooted in relational CityJSON views.
- Relational-view based export path for Arrow batches and IPC streams.

### Changed
- Export and stream writing now consume `cityjson_types::relational::ModelRelationalView`.
- Reduced the public API to the vNext codec contract instead of compatibility-oriented builder surfaces.

## [0.6.2] - 2026-04-22

### Changed

- Bumped the package version to `0.6.2`.
- Aligned the direct `cityjson` dependency with `0.7.2` for the lockstep release train.
