# Python Binding Layout

This directory will hold the Python package built on top of the shared
low-level `cjlib` FFI core.

Planned layout:

- `pyproject.toml`: Python package metadata
- `src/cjlib/`: Python package
- `tests/`: Python-facing smoke and integration tests

The Python layer should expose object-oriented wrappers and views, not raw
handles as the normal public API.
