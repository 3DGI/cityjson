# cityjson-lib-ffi-core

`cityjson-lib-ffi-core` is the shared low-level C ABI for the release-facing
bindings in this repository.

It intentionally stays small and explicit:

- opaque model handles with owned transfer helpers
- stable status and error categories
- parse, serialize, summary, and workflow entry points
- generated C headers under `include/cityjson_lib/cityjson_lib.h`

The C++ and Python bindings build on this crate. The wasm adapter uses the same
substrate internally, but it is not part of the public release surface yet.

This crate is dual-licensed under MIT or Apache-2.0. See the package metadata
and the license files in this repository.
