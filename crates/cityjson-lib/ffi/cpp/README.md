# C++ Binding Layout

This directory holds the first public C++ wrapper over the shared low-level
`cityjson_lib` FFI core.

Current layout:

- `include/`: public headers
- `tests/`: C++ smoke and integration tests

The current wrapper is intentionally small:

- RAII ownership for `cj_model_t`
- probe, parse, serialize, and create helpers
- model summary queries
- metadata setters and getters, cityobject inspection, geometry-type, and coordinate access
- transform write control, cityobject mutation, geometry attachment, extraction, append, and cleanup
- boundary-backed geometry insertion and feature-stream serialization helpers
- a CMake smoke test linked against the shared FFI library

The C++ layer stays RAII-oriented and STL-friendly while compiling down to the
shared low-level core.

The shared C ABI header is generated into `../core/include/cityjson_lib/cityjson_lib.h` via
`just ffi-header`. The C++ wrapper should treat that header as its canonical
low-level contract rather than duplicating the declarations.
