# Python Binding

This directory contains the `cityjson-lib` Python package built on top of the
shared low-level C ABI.

The binding stays intentionally small:

- `ctypes` loading of the shared library
- `CityModel` wrappers over native handles
- probe, parse, serialize, create, and summary helpers
- metadata, cityobject ID, geometry-type, and geometry access
- smoke tests that exercise the published surface

The package is publishable to PyPI as `cityjson-lib`. The build step compiles
the shared Rust FFI core and bundles the native library into the wheel and
source distribution.
