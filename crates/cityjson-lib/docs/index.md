# cjlib

`cjlib` is the integration crate for the CityJSON ecosystem in this repository.

It is not a second CityJSON domain model. The intended layering is:

- `cityjson-rs`: normalized in-memory model and correctness-critical invariants
- `serde_cityjson`: CityJSON JSON and JSONL parsing/serialization
- `cjlib`: user-facing convenience API, version dispatch, sibling format integration, and future FFI boundary

## Public API Shape

The future public API is intentionally small.

### Primary entry points

- `cjlib::CityModel`
- `cjlib::CityJSONVersion`
- `cjlib::Error`

### Default JSON path

These stay as the ergonomic default for CityJSON JSON input:

- `CityModel::from_slice`
- `CityModel::from_file`
- `CityModel::from_stream`

### Explicit format modules

Formats beyond the default CityJSON JSON path should be explicit:

- `cjlib::json`
- `cjlib::arrow`
- `cjlib::parquet`

The design goal is:

- top-level methods for the common CityJSON path
- module-qualified methods for format-specific behavior

## Working Model

`cjlib::CityModel` should remain a thin owned wrapper over `cityjson::v2_0::OwnedCityModel`.
The facade should add only:

- constructor convenience
- version classification
- a small error surface
- feature-gated format integration

Everything else should come from `cityjson-rs`.

## User Experience

For most users, the expected workflow should be:

1. read a CityJSON document or stream with `CityModel::from_*`
2. work with the model through deref access to `cityjson-rs`
3. drop down to `cjlib::json` when explicit format-boundary control is needed
4. use feature-gated sibling modules for Arrow or Parquet transport

## Non-goals

The future `cjlib` API should not:

- reintroduce a second in-memory CityJSON model
- expose indexed JSON internals as the normal user-facing API
- duplicate parsing or conversion logic that belongs in `serde_cityjson`
- absorb storage invariants that belong in `cityjson-rs`
