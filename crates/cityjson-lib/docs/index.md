# cjlib

`cjlib` is the ergonomic entry point for the CityJSON crates in this repository.

The design is intentionally small:

- `cityjson-rs` is the only in-memory model
- `cjlib::CityModel` is a thin owned wrapper over `cityjson::v2_0::OwnedCityModel`
- `cjlib` keeps version classification and constructor convenience methods
- future FFI crates should bind to `cjlib`, not reimplement the model boundary

## Current scope

Supported today:

- constructing a new owned model with `CityModel::new`
- importing a full `CityJSON` v2.0 document with `from_slice` or `from_file`
- importing a strict `CityJSONFeature` stream with `from_stream`
- working directly with re-exported `cityjson` types when finer control is needed

Recognized but intentionally unfinished:

- `CityJSON` v1.0 import: `todo!()`
- `CityJSON` v1.1 import: `todo!()`

## Positioning

`cjlib` is not a second CityJSON domain model anymore.

If a type already exists in `cityjson-rs`, `cjlib` should re-export it or let callers access it
through the wrapped model. The crate’s job is to be the stable, user-friendly facade at the format
boundary and the future FFI boundary.
