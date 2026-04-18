# Python Binding

This directory contains the `cityjson-lib` Python package built on top of the
shared low-level C ABI.

The binding stays intentionally small:

- `ctypes` loading of the shared library
- `CityModel` wrappers over native handles
- typed authoring classes for `Value`, `Contact`, `CityObjectDraft`, `RingDraft`, `SurfaceDraft`, `ShellDraft`, and `GeometryDraft`
- typed resource IDs for semantics, materials, textures, geometries, templates, and cityobjects
- probe, parse, serialize, create, summary, and stream helpers
- smoke tests that exercise the published surface

The package is publishable to PyPI as `cityjson-lib`. The build step compiles
the shared Rust FFI core and bundles the native library into the wheel and
source distribution.

The main end-to-end Python reference is
[`examples/fake_complete.py`](examples/fake_complete.py), which builds the full
`cityjson_fake_complete.city.json` fixture through the public Python API.
