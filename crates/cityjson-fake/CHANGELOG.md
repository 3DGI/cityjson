# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0] - 2026-04-18

### Changed

- **Breaking**: `attributes_random_values: bool` in `AttributeConfig` is replaced by two new fields:
  - `attributes_value_mode: AttributeValueMode` — controls the value-type strategy per attribute key. Values: `heterogenous` (default, previously `true`) or `homogenous` (previously `false`, with new semantics).
  - `attributes_allow_null: bool` (default `true`) — controls whether `null` is a possible attribute value in either mode.
- In the manifest schema, the `attributes_random_values` field is replaced by `attributes_value_mode` and `attributes_allow_null`.
- In the CLI, `--attributes-random-values` is replaced by `--attributes-value-mode heterogenous|homogenous` and `--attributes-allow-null`.
- Attribute generation is now **per `CityObject`** (was: one shared set cloned to all objects). This makes heterogenous mode truly heterogenous: the same attribute key can have different value types across different `CityObject`s.
- `AttributeValueMode::Homogenous` now pre-generates an attribute type table (`AttributeSchema`) once per model. Every `CityObject` receives the same attribute keys, each with a fixed scalar type (`Bool`, `Integer`, `Unsigned`, `Float`, or `String`), but freshly generated values. When `allow_null` is enabled, each value has a 1-in-7 chance of being `null` instead of its designated scalar type.
- Attribute generation is applied consistently to `CityObject` attributes, semantic surface attributes, and metadata extra attributes using the same faker configuration.

### Added

- `AttributeValueMode` enum (`Heterogenous` | `Homogenous`) in `cityjson_fake::attribute`.
- `AttributeSchema` struct (pre-generated type table for homogenous mode) in `cityjson_fake::attribute`.
- `ScalarType` enum (`Bool` | `Integer` | `Unsigned` | `Float` | `String`) in `cityjson_fake::attribute`.
- `AttributesFaker::generate_schema()` — generates the attribute type table for homogenous mode.
- `AttributesFaker::generate_from_schema()` — generates per-object attributes using a pre-generated type table.
- Semantic surface attributes: `Semantic` objects now receive generated attributes via the same `AttributesFaker` configuration.
- Metadata extra attributes: `Metadata.extra` is now populated when attributes are enabled and metadata is enabled.
- `AttributeValueMode` is re-exported from `cityjson_fake::prelude`.

## [0.4.0] - 2025-04-07

### Changed

- Updated to `rand` 0.10 and `jsonschema` 0.46 APIs.
- Expanded fuzz and CLI test coverage.

## [0.3.1] - 2024-11-12

### Fixed

- Minor bug fixes and cleanup.

## [0.3.0] - 2024-10-01

### Added

- Manifest-driven generation with JSON schema validation.
- CLI manifest support (`--manifest`, `--case`, `--check-manifest`).

## [0.2.0] - 2024-08-01

### Added

- Initial public release with `CityModelBuilder` API.
- CLI binary `cjfake`.

[Unreleased]: https://github.com/3DGI/cityjson-fake/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/3DGI/cityjson-fake/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/3DGI/cityjson-fake/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/3DGI/cityjson-fake/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/3DGI/cityjson-fake/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/3DGI/cityjson-fake/releases/tag/v0.2.0
