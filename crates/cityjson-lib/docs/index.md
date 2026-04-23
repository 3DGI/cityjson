# cityjson-lib

`cityjson_lib` is the release-facing facade for the current CityJSON crate and
binding surface in this repository.

The docs set is intentionally small.
It only covers the currently shipped Rust, Python, and C++ APIs.

## Start Here

- [Guide](guide.md)
- [Reading Data](guide-reading.md)
- [Writing Data](guide-writing.md)
- [Public API](public-api.md)
- [Binding API](ffi/api.md)
- [Release Checklist](release-checklist.md)

## Scope

The current release-facing surface is:

- Rust JSON and CityJSONSeq IO
- Rust `ops` helpers: `cleanup`, `subset`, `select_cityobjects`, `select_geometries`, `extract`, `append`, `merge`
- Rust `query::summary`
- the shared FFI core in `ffi/core`
- Python bindings
- C++ bindings

Wasm, Arrow, and Parquet are not part of the current user docs surface.
