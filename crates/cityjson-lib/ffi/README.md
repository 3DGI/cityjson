# FFI Layout

This directory holds the non-Rust binding work for `cjlib`.

The intended layering is:

- `core/`: shared low-level Rust FFI surface
- `cpp/`: C++ wrapper over the shared core
- `python/`: Python binding over the shared core
- `wasm/`: wasm adapter over the shared core

Only `core/` and `wasm/` are Rust crates. The C++ and Python directories are
host-language projects that will consume the shared core.
