# FFI Layout

This directory holds the non-Rust binding work for `cjlib`.

The intended layering is:

- `core/`: shared low-level Rust FFI surface
- `cpp/`: C++ wrapper over the shared core
- `python/`: Python binding over the shared core
- `wasm/`: wasm adapter over the shared core

Only `core/` and `wasm/` are Rust crates. The C++ and Python directories are
host-language projects that consume the shared core.

Current implementation status:

- `core/`: shared C ABI foundation plus read-only inspection, copied coordinate
  buffers, columnar geometry boundary extraction, and minimal creation or
  add-vertex paths
- `cpp/`: first RAII wrapper with parse, inspect, serialize, geometry-boundary
  helpers, and smoke tests
- `python/`: first `ctypes` binding with object wrapper, geometry-boundary
  helpers, and smoke tests
- `wasm/`: narrow task-oriented adapter for probe, summary, coordinate
  extraction, and geometry-boundary extraction
