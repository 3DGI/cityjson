# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
