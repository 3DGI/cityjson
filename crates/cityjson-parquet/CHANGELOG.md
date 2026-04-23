# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- native Parquet dataset format with `ParquetDatasetWriter` and
  `ParquetDatasetReader`
- docs for the two durable format surfaces: `.cityjson-parquet` package and
  native Parquet dataset
- shared-corpus roundtrip tests for native Parquet datasets

### Changed

- nullable canonical `FixedSizeList` columns are written to native Parquet
  datasets as nullable Parquet lists with reader-side fixed-length validation
  for PyArrow, DuckDB, and Polars interoperability

## [0.5.4] - 2026-04-22

### Changed

- Bumped the package version to `0.5.4`.
- Aligned `cityjson-arrow` to `0.6.2` and the direct `cityjson` dependency to `0.7.2` for the lockstep release train.

## [0.5.2] - 2026-04-17

### Changed

- fixed `readme` field in `Cargo.toml` to point to this crate's own `README.md`
  instead of `../cityjson-arrow/README.md`

### Added

- README.md, CLAUDE.md, STATUS.md, CHANGELOG.md
- justfile with `check`, `build`, `lint`, `fmt`, `test`, `coverage`,
  `rustdoc`, `site-build`, `site-serve`, `ci` recipes
- .gitignore
- pyproject.toml and properdocs.yml for the documentation site
- docs/ with `index.md`, `cityjson-parquet.md`, `cityjson-parquet-spec.md`,
  `package-schema.md`, and `design.md`
