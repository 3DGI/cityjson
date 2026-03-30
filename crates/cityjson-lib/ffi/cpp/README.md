# C++ Binding Layout

This directory holds the first public C++ wrapper over the shared low-level
`cjlib` FFI core.

Current layout:

- `include/`: public headers
- `tests/`: C++ smoke and integration tests

The current wrapper is intentionally small:

- RAII ownership for `cj_model_t`
- probe, parse, serialize, and create helpers
- model summary queries
- metadata, cityobject ID, geometry-type, and coordinate access
- a CMake smoke test linked against the shared FFI library

The C++ layer stays RAII-oriented and STL-friendly while compiling down to the
shared low-level core.

The shared C ABI header is generated into `../core/include/cjlib/cjlib.h` via
`just ffi-header`. The C++ wrapper should treat that header as its canonical
low-level contract rather than duplicating the declarations.
