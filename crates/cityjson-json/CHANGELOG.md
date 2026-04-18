# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and this project follows Semantic Versioning.

## [0.7.1] - 2026-04-18

### Added
- Facade helpers for `cityjson-lib` integration, exposed as a new `facade` module.
- Serialization support for geometry-valued attributes.  

## [0.7.0] - 2026-04-16

### Added
- Explicit read and write entry points for owned CityJSON models and feature streams.
- Configuration objects that keep JSON I/O options at the module boundary instead of on wrapper types.

### Changed
- Removed the borrowed-model-first public shape in favor of an explicit owned-model JSON boundary.
- Aligned the crate with the relational vNext integration surface exposed by `cityjson`.
