# Python Binding Layout

This directory holds the Python package built on top of the shared low-level
`cjlib` FFI core.

Current layout:

- `pyproject.toml`: Python package metadata
- `src/cjlib/`: Python package
- `tests/`: Python-facing smoke and integration tests

The current binding is intentionally small and explicit:

- pure-`ctypes` loading of the shared C ABI
- `CityModel` object wrapper over native handles
- probe, parse, serialize, create, and summary helpers
- metadata, cityobject ID, geometry-type, and coordinate access
- a Python smoke test that exercises the built shared library

The Python layer exposes object-oriented wrappers and views, not raw handles
as the normal public API.
