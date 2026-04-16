# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and this project follows Semantic Versioning.

## [0.4.0] - 2026-04-16

### Added
- Arrow-first facade and binding flows that route columnar export through `cityjson-arrow`.
- Updated ADRs and public API docs describing the thinner vNext facade.

### Changed
- `CityModel` is now a direct re-export of `cityjson::v2_0::OwnedCityModel`.
- Removed the wrapper-model compatibility layer in favor of the minimal facade contract.
- Updated JSON, Arrow, FFI, examples, and tests to work directly against the re-exported model type.
