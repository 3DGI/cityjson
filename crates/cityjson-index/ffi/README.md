# FFI Layout

This directory holds the release-facing non-Rust binding work for
`cityjson_index`.

## Layout

- `core/`: publishable shared C ABI crate for the wrapper packages
- `python/`: publishable Python package over the shared core

## Status

- `core/`: shared C ABI and native entry points for the index API
- `python/`: PyPI-ready `ctypes` package with wheel and sdist build support

See `ffi/python/README.md` for the Python package surface.
