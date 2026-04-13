# FFI Layout

This directory holds the non-Rust binding work for `cityjson_lib`.

The intended layering is:

- `core/`: shared low-level Rust FFI surface
- `cpp/`: C++ wrapper over the shared core
- `python/`: Python binding over the shared core
- `wasm/`: wasm adapter over the shared core

Only `core/` and `wasm/` are Rust crates. The C++ and Python directories are
host-language projects that consume the shared core.

Current implementation status:

- `core/`: shared C ABI foundation plus parse and serialize entry points,
  read/write options, read-only inspection, copied coordinate buffers,
  geometry-boundary extraction, targeted mutation, and
  model-authoritative workflows
- `cpp/`: first RAII wrapper with parse, inspect, serialize, cleanup,
  append/extract, geometry-boundary helpers, feature-stream helpers, and smoke
  tests
- `python/`: first `ctypes` binding with object wrapper, cleanup,
  append/extract, geometry-boundary helpers, feature-stream helpers, and smoke
  tests
- `wasm/`: task-oriented adapter over the shared core, exercised through the
  real `wasm32-unknown-unknown` benchmark path, with probe/summary, coordinate
  and boundary extraction, write options, feature-stream merge, and a small
  authoring/cleanup slice
