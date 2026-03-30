# C++ Binding Layout

This directory will hold the public C++ wrapper over the shared low-level
`cjlib` FFI core.

Planned layout:

- `include/`: public headers
- `src/`: wrapper implementation
- `tests/`: C++ smoke and integration tests

The C++ layer should stay RAII-oriented and STL-friendly while compiling down
to the shared low-level core.
