# Python Binding Layout

This directory holds the Python package built on top of the shared low-level
`cityjson_lib` FFI core.

Current layout:

- `pyproject.toml`: Python package metadata
- `src/cityjson_lib/`: Python package
- `tests/`: Python-facing smoke and integration tests

The current binding is intentionally small and explicit:

- pure-`ctypes` loading of the shared C ABI
- `CityModel` object wrapper over native handles
- Arrow-byte parse and serialize as the primary bulk transport path
- probe, parse, serialize, create, and summary helpers
- metadata, cityobject ID, geometry-type, and single-geometry access
- a Python smoke test that exercises the built shared library

The Python layer keeps bulk interchange Arrow-first and avoids materializing
wrapper-wide projected cityobject or vertex collections.
