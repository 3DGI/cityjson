# cityjson-lib

`cityjson_lib` is the stable facade crate for the current CityJSON release line.

It does three jobs:

- exposes a small Rust API over the shared CityJSON model
- delegates JSON and CityJSONSeq boundary work to `cityjson-json`
- hosts the shared FFI core used by the Python and C++ bindings

This crate is not a second CityJSON model.

## Current Split

- `cityjson-rs`
  Semantic model, invariants, and correctness-sensitive mutation.
- `cityjson-json`
  CityJSON and CityJSONSeq parsing, probing, staged reconstruction, and
  serialization.
- `cityjson-lib`
  Rust facade, `query`, `ops`, and the shared low-level FFI core.

## Start Here

- [Guide](guide.md)
  How to use the Rust facade on the current release line.
- [Reading Data](guide-reading.md)
  Parallel read examples for Rust, Python, and C++.
- [Writing Data](guide-writing.md)
  Parallel write examples for Rust, Python, and C++.
- [Public API](public-api.md)
  The stable Rust-facing surface.
- [Binding API](ffi/api.md)
  The published Python and C++ surfaces, with matching Rust examples.
- [Release Checklist](release-checklist.md)
  Concrete release steps for crates.io, PyPI, and the C++ install path.

## Scope

The publishable surface on this branch is:

- Rust JSON and CityJSONSeq support
- Rust `ops` helpers for `cleanup`, `extract`, `append`, and `merge`
- Python bindings over the shared C ABI
- C++ bindings over the same shared C ABI

The wasm adapter is still work in progress.
Arrow and Parquet are intentionally out of the current publishable crate.

## Archive

Older planning pages remain in the docs tree as short archived notes.
They are no longer the source of truth for the release surface.
