# C++ Binding Layout

This directory will hold the public C++ wrapper over the shared low-level
`cjlib` FFI core.

Planned layout:

- `include/`: public headers
- `src/`: wrapper implementation
- `tests/`: C++ smoke and integration tests

The C++ layer should stay RAII-oriented and STL-friendly while compiling down
to the shared low-level core.

The shared C ABI header is generated into `../core/include/cjlib/cjlib.h` via
`just ffi-header`. The C++ wrapper should treat that header as its canonical
low-level contract rather than duplicating the declarations.
