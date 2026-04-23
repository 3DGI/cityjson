# FFI Layout

This directory holds the release-facing non-Rust binding work for `cityjson_lib`.

## Layout

- `core/`: publishable shared C ABI crate for the wrapper packages
- `cpp/`: installable C++ wrapper over the shared core
- `python/`: publishable Python package over the shared core
- `wasm/`: work-in-progress wasm adapter over the shared core

## Status

- `core/`: publishable shared C ABI, parse and serialize entry points, read/write options, inspection, geometry boundaries, and model workflows
- `cpp/`: installable CMake package with smoke tests and generated config
- `python/`: PyPI-ready `ctypes` package with wheel and sdist build support
- `wasm/`: intentionally out of the public release surface for now

See [Binding API](../docs/ffi/api.md) for tabbed Rust, Python, and C++ examples of the publishable surface.
